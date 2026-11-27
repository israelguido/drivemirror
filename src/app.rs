use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use drivemirror::{Config, SyncCommand, SyncRunner, default_config_path};
use gtk::{gio, glib, prelude::*};

use crate::{theme, tray::Action};

struct Ui {
    local: gtk::Entry,
    remote: gtk::Entry,
    remote_folder: gtk::Entry,
    interval: gtk::SpinButton,
    excludes: gtk::TextView,
    excluded_folders: gtk::TextView,
    status: gtk::Label,
    sync: gtk::Button,
}

pub fn activate(
    app: &gtk::Application,
    receiver: Arc<Mutex<std::sync::mpsc::Receiver<Action>>>,
    start_hidden: bool,
) {
    if !app.windows().is_empty() {
        app.windows()[0].present();
        return;
    }
    let hold = app.hold();
    theme::apply();
    let config_path = default_config_path();
    let initial = Config::load(&config_path).unwrap_or_default();
    let config = Arc::new(Mutex::new(initial.clone()));
    let runner = SyncRunner::default();
    let last_sync = Rc::new(RefCell::new(Instant::now()));
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("DriveMirror")
        .default_width(720)
        .default_height(720)
        .build();
    window.connect_destroy(move |_| {
        let _ = &hold;
    });
    window.connect_close_request(|window| {
        window.set_visible(false);
        glib::Propagation::Stop
    });

    let ui = Rc::new(build_ui(&initial));
    let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
    root.set_margin_top(22);
    root.set_margin_bottom(22);
    root.set_margin_start(24);
    root.set_margin_end(24);
    let title = gtk::Label::new(Some("DriveMirror"));
    title.add_css_class("title-1");
    title.set_halign(gtk::Align::Start);
    let subtitle = gtk::Label::new(Some(
        "Espelhe uma pasta local para um destino privado do rclone.",
    ));
    subtitle.set_halign(gtk::Align::Start);
    subtitle.add_css_class("dim-label");
    root.append(&title);
    root.append(&subtitle);
    let grid = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
        .build();
    add_row(&grid, 0, "Pasta local", &ui.local);
    let choose = gtk::Button::with_label("Escolher…");
    grid.attach(&choose, 2, 0, 1, 1);
    add_row(&grid, 1, "Remoto rclone", &ui.remote);
    add_row(&grid, 2, "Pasta no Drive", &ui.remote_folder);
    add_row(&grid, 3, "Intervalo (min)", &ui.interval);
    root.append(&grid);
    root.append(&section(
        "Arquivos ignorados (um padrão por linha)",
        &ui.excludes,
    ));
    root.append(&section(
        "Pastas ignoradas (um padrão por linha)",
        &ui.excluded_folders,
    ));
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let save = gtk::Button::with_label("Salvar configurações");
    let open = gtk::Button::with_label("Abrir arquivo de configuração");
    ui.sync.set_label("Sincronizar agora");
    ui.sync.add_css_class("sync-button");
    actions.append(&save);
    actions.append(&open);
    actions.append(&ui.sync);
    root.append(&actions);
    ui.status.set_halign(gtk::Align::Start);
    ui.status.set_wrap(true);
    root.append(&ui.status);
    window.set_child(Some(&root));

    let local = ui.local.clone();
    choose.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            let dialog = gtk::FileChooserNative::builder()
                .title("Escolha a pasta local")
                .transient_for(&window)
                .action(gtk::FileChooserAction::SelectFolder)
                .accept_label("Escolher")
                .cancel_label("Cancelar")
                .build();
            dialog.connect_response(glib::clone!(
                #[weak]
                local,
                move |dialog, response| {
                    if response == gtk::ResponseType::Accept
                        && let Some(path) = dialog.file().and_then(|file| file.path())
                    {
                        local.set_text(&path.to_string_lossy());
                    }
                    dialog.destroy();
                }
            ));
            dialog.show();
        }
    ));

    save.connect_clicked(glib::clone!(
        #[strong]
        ui,
        #[strong]
        config,
        #[strong]
        config_path,
        #[strong]
        last_sync,
        move |_| {
            match read_ui(&ui).and_then(|value| {
                value.save(&config_path)?;
                *config.lock().map_err(|_| "Configuração indisponível")? = value;
                Ok(())
            }) {
                Ok(()) => {
                    *last_sync.borrow_mut() = Instant::now();
                    ui.status
                        .set_text("Configurações salvas. O próximo ciclo usa o novo intervalo.");
                }
                Err(error) => ui.status.set_text(&error),
            }
        }
    ));
    let status = ui.status.clone();
    open.connect_clicked(glib::clone!(
        #[strong]
        config_path,
        #[weak]
        status,
        move |_| {
            if !config_path.exists()
                && let Err(error) = Config::default().save(&config_path)
            {
                status.set_text(&error);
                return;
            }
            let uri = glib::filename_to_uri(&config_path, None).unwrap_or_default();
            if let Err(error) =
                gio::AppInfo::launch_default_for_uri(&uri, gio::AppLaunchContext::NONE)
            {
                status.set_text(&format!("Não foi possível abrir o editor: {error}"));
            }
        }
    ));

    let run_sync: Rc<dyn Fn()> = Rc::new(glib::clone!(
        #[strong]
        ui,
        #[strong]
        config,
        #[strong]
        runner,
        #[strong]
        config_path,
        #[strong]
        last_sync,
        move || {
            *last_sync.borrow_mut() = Instant::now();
            let loaded = Config::load(&config_path).and_then(|value| {
                value.validate()?;
                Ok(value)
            });
            match loaded
                .and_then(|value| {
                    *config.lock().map_err(|_| "Configuração indisponível")? = value.clone();
                    SyncCommand::from_config(&value)
                })
                .and_then(|command| runner.start(command))
            {
                Ok(()) => {
                    ui.status.set_text("Sincronizando…");
                    ui.sync.set_sensitive(false);
                }
                Err(error) => ui.status.set_text(&error),
            }
        }
    ));
    ui.sync.connect_clicked(glib::clone!(
        #[strong]
        run_sync,
        move |_| run_sync()
    ));

    glib::timeout_add_local(
        Duration::from_millis(400),
        glib::clone!(
            #[strong]
            ui,
            #[strong]
            runner,
            #[strong]
            config,
            #[strong]
            last_sync,
            #[strong]
            run_sync,
            #[strong]
            window,
            #[strong]
            app,
            move || {
                while let Ok(action) = receiver.lock().expect("tray receiver").try_recv() {
                    match action {
                        Action::Show => window.present(),
                        Action::Sync => run_sync(),
                        Action::Quit => app.quit(),
                    }
                }
                if !runner.is_running() && !ui.sync.is_sensitive() {
                    ui.sync.set_sensitive(true);
                    if let Some(result) = runner.last_result() {
                        let message = if result.success {
                            if result.message.is_empty() {
                                "Sincronização concluída.".into()
                            } else {
                                result.message
                            }
                        } else {
                            format!("Falha: {}", result.message)
                        };
                        ui.status.set_text(&message);
                        *last_sync.borrow_mut() = Instant::now();
                    }
                }
                let minutes = config.lock().map(|c| c.interval_minutes).unwrap_or(10);
                if !runner.is_running()
                    && last_sync.borrow().elapsed()
                        >= Duration::from_secs(minutes.saturating_mul(60))
                {
                    run_sync();
                }
                glib::ControlFlow::Continue
            }
        ),
    );
    let config_status = ui.status.clone();
    glib::timeout_add_local(
        Duration::from_secs(2),
        glib::clone!(
            #[strong]
            config,
            #[strong]
            config_path,
            #[strong]
            config_status,
            move || {
                theme::apply();
                match Config::load(&config_path).and_then(|value| {
                    value.validate()?;
                    Ok(value)
                }) {
                    Ok(value) => {
                        if let Ok(mut current) = config.lock() {
                            *current = value;
                        }
                    }
                    Err(error) if config_path.exists() => config_status.set_text(&error),
                    Err(_) => {}
                }
                glib::ControlFlow::Continue
            }
        ),
    );
    if !start_hidden {
        window.present();
    }
}

fn build_ui(config: &Config) -> Ui {
    let local = gtk::Entry::new();
    local.set_hexpand(true);
    local.set_text(&config.local_folder.to_string_lossy());
    let remote = gtk::Entry::new();
    remote.set_text(&config.remote);
    remote.set_placeholder_text(Some("gdrive"));
    let remote_folder = gtk::Entry::new();
    remote_folder.set_text(&config.remote_folder);
    remote_folder.set_placeholder_text(Some("Backup/Documentos"));
    let interval = gtk::SpinButton::with_range(1.0, 1440.0, 1.0);
    interval.set_value(config.interval_minutes as f64);
    let excludes = text_view(&config.exclude);
    let excluded_folders = text_view(&config.exclude_folders);
    Ui {
        local,
        remote,
        remote_folder,
        interval,
        excludes,
        excluded_folders,
        status: gtk::Label::new(Some(
            "Configure o destino e salve antes da primeira sincronização.",
        )),
        sync: gtk::Button::new(),
    }
}

fn add_row<W: IsA<gtk::Widget>>(grid: &gtk::Grid, row: i32, text: &str, widget: &W) {
    let label = gtk::Label::new(Some(text));
    label.set_halign(gtk::Align::Start);
    grid.attach(&label, 0, row, 1, 1);
    grid.attach(widget, 1, row, 1, 1);
}
fn text_view(lines: &[String]) -> gtk::TextView {
    let view = gtk::TextView::new();
    view.set_monospace(true);
    view.set_vexpand(true);
    view.buffer().set_text(&lines.join("\n"));
    view
}
fn section(title: &str, view: &gtk::TextView) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 5);
    let label = gtk::Label::new(Some(title));
    label.set_halign(gtk::Align::Start);
    let scroll = gtk::ScrolledWindow::builder()
        .min_content_height(90)
        .child(view)
        .build();
    box_.append(&label);
    box_.append(&scroll);
    box_
}
fn lines(view: &gtk::TextView) -> Vec<String> {
    let buffer = view.buffer();
    buffer
        .text(&buffer.start_iter(), &buffer.end_iter(), false)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect()
}
fn read_ui(ui: &Ui) -> Result<Config, String> {
    let value = Config {
        local_folder: PathBuf::from(ui.local.text().as_str()),
        remote: ui.remote.text().to_string(),
        remote_folder: ui.remote_folder.text().to_string(),
        interval_minutes: ui.interval.value_as_int() as u64,
        exclude: lines(&ui.excludes),
        exclude_folders: lines(&ui.excluded_folders),
    };
    value.validate()?;
    Ok(value)
}
