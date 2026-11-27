use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use drivemirror::{
    Config, SyncCommand, SyncRunner,
    accounts::{RemoteProblem, remote_problem},
    default_config_path,
    i18n::{Language, Message, tr},
    panel::PanelState,
};
use gtk::{gio, glib, prelude::*};

use crate::{
    theme,
    tray::{Action, DriveMirrorTray},
};

type SyncBlock = (Option<String>, Option<Vec<String>>, Vec<String>);

pub(crate) struct Ui {
    local: gtk::Entry,
    pub(crate) remote: gtk::Entry,
    /// Drive remotes without a token, found by the accounts page; sync refuses them.
    pub(crate) incomplete_remotes: RefCell<Vec<String>>,
    /// Remote names from the last successful listing; `None` while unknown.
    pub(crate) known_remotes: RefCell<Option<Vec<String>>>,
    remote_folder: gtk::Entry,
    interval: gtk::SpinButton,
    excludes: gtk::TextView,
    excluded_folders: gtk::TextView,
    status: gtk::Label,
    sync: gtk::Button,
    language: gtk::DropDown,
    translated: RefCell<Vec<(glib::WeakRef<gtk::Widget>, Message)>>,
    refreshing: Cell<bool>,
    tray: Option<ksni::blocking::Handle<DriveMirrorTray>>,
}

pub fn activate(
    app: &gtk::Application,
    receiver: Arc<Mutex<std::sync::mpsc::Receiver<Action>>>,
    start_hidden: bool,
    tray_error: Option<String>,
    tray: Option<ksni::blocking::Handle<DriveMirrorTray>>,
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
    let quit_requested = Rc::new(Cell::new(false));
    // What the automatic sync last refused (config text, known and incomplete remotes):
    // it stays quiet until one of them changes instead of repeating every interval.
    let blocked = Rc::new(RefCell::new(None::<SyncBlock>));
    // Outcome and local (hour, minute) of the latest sync, shown by the tray panel.
    let last_success = Rc::new(Cell::new(None::<bool>));
    let last_success_at = Rc::new(Cell::new(None::<(u32, u32)>));
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

    let ui = Rc::new(build_ui(&initial, tray));
    if let Some(error) = &tray_error {
        ui.status
            .set_text(&t(&ui, Message::TrayUnavailable).replace("{error}", error));
    }
    let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
    root.set_margin_top(22);
    root.set_margin_bottom(22);
    root.set_margin_start(24);
    root.set_margin_end(24);
    let title = gtk::Label::new(Some("DriveMirror"));
    title.add_css_class("title-1");
    title.set_halign(gtk::Align::Start);
    let subtitle = translated_label(&ui, Message::Subtitle);
    subtitle.set_halign(gtk::Align::Start);
    subtitle.add_css_class("dim-label");
    root.append(&title);
    root.append(&subtitle);
    let sync_page = gtk::Box::new(gtk::Orientation::Vertical, 14);
    sync_page.set_margin_top(12);
    let grid = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
        .build();
    add_row(&grid, 0, &ui, Message::LocalFolder, &ui.local);
    let choose = translated_button(&ui, Message::Choose);
    grid.attach(&choose, 2, 0, 1, 1);
    add_row(&grid, 1, &ui, Message::RemoteLabel, &ui.remote);
    add_row(&grid, 2, &ui, Message::RemoteFolderLabel, &ui.remote_folder);
    add_row(&grid, 3, &ui, Message::IntervalLabel, &ui.interval);
    add_row(&grid, 4, &ui, Message::LanguageLabel, &ui.language);
    sync_page.append(&grid);
    sync_page.append(&section(&ui, Message::ExcludedFiles, &ui.excludes));
    sync_page.append(&section(
        &ui,
        Message::ExcludedFolders,
        &ui.excluded_folders,
    ));
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let save = translated_button(&ui, Message::SaveSettings);
    let open = translated_button(&ui, Message::OpenConfigFile);
    register(&ui, &ui.sync, Message::SyncNow);
    ui.sync.add_css_class("sync-button");
    actions.append(&save);
    actions.append(&open);
    actions.append(&ui.sync);
    sync_page.append(&actions);
    ui.status.set_halign(gtk::Align::Start);
    ui.status.set_wrap(true);
    sync_page.append(&ui.status);
    let notebook = gtk::Notebook::new();
    notebook.set_vexpand(true);
    notebook.append_page(&sync_page, Some(&translated_label(&ui, Message::SyncTab)));
    notebook.append_page(
        &crate::accounts_page::build(&ui, window.upcast_ref()),
        Some(&translated_label(&ui, Message::AccountsTab)),
    );
    root.append(&notebook);
    window.set_child(Some(&root));
    apply_language(&ui);

    let local = ui.local.clone();
    let chooser_ui = ui.clone();
    let chooser = Rc::new(RefCell::new(None::<gtk::FileChooserNative>));
    choose.connect_clicked(glib::clone!(
        #[weak]
        window,
        #[strong]
        chooser,
        move |_| {
            let dialog = gtk::FileChooserNative::builder()
                .title(t(&chooser_ui, Message::ChooseLocalFolder))
                .transient_for(&window)
                .action(gtk::FileChooserAction::SelectFolder)
                .accept_label(t(&chooser_ui, Message::Accept))
                .cancel_label(t(&chooser_ui, Message::Cancel))
                .build();
            dialog.connect_response(glib::clone!(
                #[weak]
                local,
                #[strong]
                chooser,
                move |dialog, response| {
                    if response == gtk::ResponseType::Accept
                        && let Some(path) = dialog.file().and_then(|file| file.path())
                    {
                        local.set_text(&path.to_string_lossy());
                    }
                    dialog.destroy();
                    chooser.borrow_mut().take();
                }
            ));
            *chooser.borrow_mut() = Some(dialog.clone());
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
                let baseline = config
                    .lock()
                    .map_err(|_| t(&ui, Message::ConfigUnavailable))?
                    .clone();
                value.save_if_unchanged(&config_path, &baseline)?;
                *config
                    .lock()
                    .map_err(|_| t(&ui, Message::ConfigUnavailable))? = value;
                Ok(())
            }) {
                Ok(()) => {
                    *last_sync.borrow_mut() = Instant::now();
                    ui.status.set_text(t(&ui, Message::SettingsSaved));
                }
                Err(error) => ui.status.set_text(&error),
            }
        }
    ));
    open.connect_clicked(glib::clone!(
        #[strong]
        config_path,
        #[strong]
        ui,
        move |_| {
            let status = &ui.status;
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
                status.set_text(
                    &t(&ui, Message::OpenEditorFailed).replace("{error}", &error.to_string()),
                );
            }
        }
    ));

    let run_sync: Rc<dyn Fn(bool)> = Rc::new(glib::clone!(
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
        #[strong]
        blocked,
        #[strong]
        last_success,
        move |manual| {
            *last_sync.borrow_mut() = Instant::now();
            let block: SyncBlock = (
                std::fs::read_to_string(&config_path).ok(),
                ui.known_remotes.borrow().clone(),
                ui.incomplete_remotes.borrow().clone(),
            );
            if !manual && blocked.borrow().as_ref() == Some(&block) {
                return;
            }
            blocked.replace(None);
            // Set when the failure only goes away after the config or remote list changes.
            let lasting = Cell::new(false);
            let prepared = if manual {
                read_ui(&ui).and_then(|selected| {
                    let baseline = config
                        .lock()
                        .map_err(|_| t(&ui, Message::ConfigUnavailable))?
                        .clone();
                    selected.save_for_sync(&config_path, &baseline)?;
                    Ok(selected)
                })
            } else {
                Config::load(&config_path)
                    .and_then(|saved| {
                        saved.validate()?;
                        Ok(saved)
                    })
                    .inspect_err(|_| lasting.set(true))
                    .and_then(|saved| {
                        if read_ui_raw(&ui) != saved {
                            populate_ui(&ui, &saved);
                            if let Ok(mut current) = config.lock() {
                                *current = saved;
                            }
                            return Err(t(&ui, Message::SavedConfigChanged).into());
                        }
                        Ok(saved)
                    })
            };
            match prepared
                .and_then(|value| {
                    let remote = value.remote.trim();
                    let problem = remote_problem(
                        remote,
                        ui.known_remotes.borrow().as_deref(),
                        &ui.incomplete_remotes.borrow(),
                    );
                    if let Some(problem) = problem {
                        lasting.set(true);
                        let message = match problem {
                            RemoteProblem::Missing => Message::RemoteMissing,
                            RemoteProblem::Incomplete => Message::RemoteNotAuthorized,
                        };
                        return Err(t(&ui, message).replace("{name}", remote));
                    }
                    *config
                        .lock()
                        .map_err(|_| t(&ui, Message::ConfigUnavailable))? = value.clone();
                    SyncCommand::from_config(&value)
                })
                .and_then(|command| runner.start(command, selected_language(&ui)))
            {
                Ok(()) => {
                    ui.status.set_text(t(&ui, Message::Syncing));
                    ui.sync.set_sensitive(false);
                }
                Err(error) => {
                    if lasting.get() {
                        last_success.set(Some(false));
                    }
                    if !manual && lasting.get() {
                        blocked.replace(Some(block));
                    }
                    ui.status.set_text(&error);
                }
            }
        }
    ));
    ui.sync.connect_clicked(glib::clone!(
        #[strong]
        run_sync,
        move |_| run_sync(true)
    ));

    let request_quit: Rc<dyn Fn()> = Rc::new(glib::clone!(
        #[strong]
        ui,
        #[strong]
        runner,
        #[strong]
        app,
        #[strong]
        quit_requested,
        move || {
            if runner.is_running() {
                quit_requested.set(true);
                ui.status.set_text(t(&ui, Message::WaitingToQuit));
            } else {
                app.quit();
            }
        }
    ));
    let panel = crate::tray_panel::build(
        &ui,
        Rc::new(glib::clone!(
            #[strong]
            run_sync,
            move || run_sync(true)
        )),
        Rc::new(glib::clone!(
            #[weak]
            window,
            move || window.present()
        )),
        request_quit.clone(),
    );
    let refresh_panel = glib::clone!(
        #[strong]
        panel,
        #[strong]
        ui,
        #[strong]
        runner,
        #[strong]
        config,
        #[strong]
        last_success,
        #[strong]
        last_success_at,
        move || {
            let Ok(config) = config.lock().map(|config| config.clone()) else {
                return;
            };
            let state = PanelState::new(
                config.validate().is_ok(),
                runner.is_running(),
                last_success.get(),
            );
            panel.refresh(
                selected_language(&ui).current(),
                state,
                last_success_at.get(),
                &config,
            );
        }
    );
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
            #[strong]
            quit_requested,
            #[strong]
            last_success,
            #[strong]
            last_success_at,
            move || {
                while let Ok(action) = receiver.lock().expect("tray receiver").try_recv() {
                    match action {
                        Action::Show => window.present(),
                        Action::TogglePanel => {
                            if panel.toggle() {
                                refresh_panel();
                            }
                        }
                        Action::Sync => run_sync(true),
                        Action::Quit => request_quit(),
                    }
                }
                if !runner.is_running() && !ui.sync.is_sensitive() {
                    ui.sync.set_sensitive(true);
                    if let Some(result) = runner.last_result() {
                        last_success.set(Some(result.success));
                        if result.success
                            && let Ok(now) = glib::DateTime::now_local()
                        {
                            last_success_at.set(Some((now.hour() as u32, now.minute() as u32)));
                        }
                        let message = if result.success {
                            if result.message.is_empty() {
                                t(&ui, Message::SyncDone).into()
                            } else {
                                result.message
                            }
                        } else {
                            t(&ui, Message::SyncFailed).replace("{error}", &result.message)
                        };
                        ui.status.set_text(&message);
                        *last_sync.borrow_mut() = Instant::now();
                        if quit_requested.get() {
                            app.quit();
                            return glib::ControlFlow::Break;
                        }
                    }
                }
                let minutes = config.lock().map(|c| c.interval_minutes).unwrap_or(10);
                if !runner.is_running()
                    && last_sync.borrow().elapsed()
                        >= Duration::from_secs(minutes.saturating_mul(60))
                {
                    run_sync(false);
                }
                if panel.is_visible() {
                    refresh_panel();
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
            #[strong]
            ui,
            move || {
                theme::apply();
                match Config::load(&config_path).and_then(|value| {
                    value.validate()?;
                    Ok(value)
                }) {
                    Ok(value) => {
                        if let Ok(mut current) = config.lock() {
                            if read_ui_raw(&ui) == *current {
                                populate_ui(&ui, &value);
                                *current = value;
                            } else if *current != value {
                                config_status.set_text(t(&ui, Message::UnsavedExternalChange));
                            }
                        }
                    }
                    Err(error) if config_path.exists() => config_status.set_text(&error),
                    Err(_) => {}
                }
                glib::ControlFlow::Continue
            }
        ),
    );
    ui.language.connect_selected_notify(glib::clone!(
        #[strong]
        ui,
        #[strong]
        config,
        #[strong]
        config_path,
        move |_| {
            if ui.refreshing.get() {
                return;
            }
            apply_language(&ui);
            // Persist only the language; other edited fields stay unsaved until Save.
            let language = selected_language(&ui);
            let result = config
                .lock()
                .map_err(|_| t(&ui, Message::ConfigUnavailable).to_string())
                .and_then(|mut current| {
                    let value = Config {
                        language,
                        ..current.clone()
                    };
                    value.save_if_unchanged(&config_path, &current)?;
                    *current = value;
                    Ok(())
                });
            match result {
                Ok(()) => ui.status.set_text(t(&ui, Message::LanguageSaved)),
                Err(error) => ui.status.set_text(&error),
            }
        }
    ));
    if !start_hidden || tray_error.is_some() {
        window.present();
    }
}

fn build_ui(config: &Config, tray: Option<ksni::blocking::Handle<DriveMirrorTray>>) -> Ui {
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
    // Index order matches Language::ALL; item 0 is retranslated by apply_language.
    let language = gtk::DropDown::from_strings(&["", "Português (Brasil)", "English", "Español"]);
    language.set_selected(language_index(config.language));
    let ui = Ui {
        local,
        remote,
        remote_folder,
        interval,
        excludes,
        excluded_folders,
        status: gtk::Label::new(None),
        sync: gtk::Button::new(),
        language,
        translated: RefCell::default(),
        incomplete_remotes: RefCell::default(),
        known_remotes: RefCell::default(),
        refreshing: Cell::new(false),
        tray,
    };
    ui.status.set_text(t(&ui, Message::InitialStatus));
    ui
}

fn language_index(language: Language) -> u32 {
    Language::ALL
        .iter()
        .position(|candidate| *candidate == language)
        .unwrap_or(0) as u32
}
pub(crate) fn selected_language(ui: &Ui) -> Language {
    Language::ALL
        .get(ui.language.selected() as usize)
        .copied()
        .unwrap_or_default()
}
pub(crate) fn t(ui: &Ui, message: Message) -> &'static str {
    tr(selected_language(ui).current(), message)
}
/// Keeps a label or button text in the selected language; dropped widgets are forgotten.
pub(crate) fn register(ui: &Ui, widget: &impl IsA<gtk::Widget>, message: Message) {
    let widget = widget.upcast_ref::<gtk::Widget>();
    set_text(widget, tr(selected_language(ui).current(), message));
    ui.translated
        .borrow_mut()
        .push((widget.downgrade(), message));
}
pub(crate) fn translated_label(ui: &Ui, message: Message) -> gtk::Label {
    let label = gtk::Label::new(None);
    register(ui, &label, message);
    label
}
pub(crate) fn translated_button(ui: &Ui, message: Message) -> gtk::Button {
    let button = gtk::Button::new();
    register(ui, &button, message);
    button
}
fn set_text(widget: &gtk::Widget, text: &str) {
    if let Some(label) = widget.downcast_ref::<gtk::Label>() {
        label.set_text(text);
    } else if let Some(button) = widget.downcast_ref::<gtk::Button>() {
        button.set_label(text);
    }
}
fn apply_language(ui: &Ui) {
    let language = selected_language(ui).current();
    ui.translated.borrow_mut().retain(|(widget, message)| {
        widget
            .upgrade()
            .map(|widget| set_text(&widget, tr(language, *message)))
            .is_some()
    });
    if let Some(model) = ui.language.model().and_downcast::<gtk::StringList>() {
        let selected = ui.language.selected();
        ui.refreshing.set(true);
        model.splice(0, 1, &[tr(language, Message::SystemLanguage)]);
        ui.language.set_selected(selected);
        ui.refreshing.set(false);
    }
    if let Some(tray) = &ui.tray {
        tray.update(|tray| tray.language = language);
    }
}

fn add_row<W: IsA<gtk::Widget>>(grid: &gtk::Grid, row: i32, ui: &Ui, message: Message, widget: &W) {
    let label = translated_label(ui, message);
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
fn section(ui: &Ui, message: Message, view: &gtk::TextView) -> gtk::Box {
    let box_ = gtk::Box::new(gtk::Orientation::Vertical, 5);
    let label = translated_label(ui, message);
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
    let value = read_ui_raw(ui);
    value.validate()?;
    Ok(value)
}
fn read_ui_raw(ui: &Ui) -> Config {
    Config {
        local_folder: PathBuf::from(ui.local.text().as_str()),
        remote: ui.remote.text().to_string(),
        remote_folder: ui.remote_folder.text().to_string(),
        interval_minutes: ui.interval.value_as_int() as u64,
        exclude: lines(&ui.excludes),
        exclude_folders: lines(&ui.excluded_folders),
        language: selected_language(ui),
    }
}
fn populate_ui(ui: &Ui, config: &Config) {
    ui.local.set_text(&config.local_folder.to_string_lossy());
    ui.remote.set_text(&config.remote);
    ui.remote_folder.set_text(&config.remote_folder);
    ui.interval.set_value(config.interval_minutes as f64);
    ui.excludes.buffer().set_text(&config.exclude.join("\n"));
    ui.excluded_folders
        .buffer()
        .set_text(&config.exclude_folders.join("\n"));
    if selected_language(ui) != config.language {
        ui.refreshing.set(true);
        ui.language.set_selected(language_index(config.language));
        ui.refreshing.set(false);
        apply_language(ui);
    }
}
