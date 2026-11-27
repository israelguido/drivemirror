//! Small summary panel opened by a left click on the tray icon. On wlroots/KDE
//! Wayland it is a layer surface under the top bar; elsewhere a small window.

use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

use drivemirror::{
    Config,
    i18n::{Language, Message},
    panel::{PanelState, last_sync_text, route_text},
};
use gtk::{gdk, glib, prelude::*};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::app::{Ui, register, translated_button};

pub(crate) struct Panel {
    window: gtk::Window,
    state: gtk::Label,
    last: gtk::Label,
    route: gtk::Label,
    /// When focus loss closed the panel; the tray click causing it must not reopen it.
    auto_hidden_at: Cell<Option<Instant>>,
}

pub(crate) fn build(
    ui: &Ui,
    on_sync: Rc<dyn Fn()>,
    on_settings: Rc<dyn Fn()>,
    on_quit: Rc<dyn Fn()>,
) -> Rc<Panel> {
    let window = gtk::Window::builder()
        .title("DriveMirror")
        .resizable(false)
        .default_width(320)
        .build();
    let layer = gtk4_layer_shell::is_supported();
    if layer {
        // Must happen before the window is realized (first present).
        window.init_layer_shell();
        window.set_namespace(Some("drivemirror"));
        window.set_layer(Layer::Top);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 8);
        window.set_margin(Edge::Right, 8);
        window.set_keyboard_mode(KeyboardMode::OnDemand);
    }
    let panel = Rc::new(Panel {
        window: window.clone(),
        state: gtk::Label::builder().xalign(1.0).hexpand(true).build(),
        last: gtk::Label::builder().xalign(0.0).build(),
        route: gtk::Label::builder()
            .xalign(0.0)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .build(),
        auto_hidden_at: Cell::new(None),
    });

    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.set_width_request(320);
    root.set_margin_top(14);
    root.set_margin_bottom(14);
    root.set_margin_start(14);
    root.set_margin_end(14);
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let title = gtk::Label::new(Some("DriveMirror"));
    title.add_css_class("heading");
    header.append(&title);
    header.append(&panel.state);
    root.append(&header);
    panel.last.add_css_class("dim-label");
    root.append(&panel.last);
    root.append(&panel.route);

    let sync = translated_button(ui, Message::SyncNow);
    sync.add_css_class("sync-button");
    sync.add_css_class("suggested-action");
    root.append(&sync);
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actions.set_homogeneous(true);
    let settings = gtk::Button::new();
    register(ui, &settings, Message::Settings);
    let quit = translated_button(ui, Message::Quit);
    actions.append(&settings);
    actions.append(&quit);
    root.append(&actions);
    window.set_child(Some(&root));

    sync.connect_clicked(move |_| on_sync());
    settings.connect_clicked(glib::clone!(
        #[weak]
        window,
        move |_| {
            window.set_visible(false);
            on_settings();
        }
    ));
    quit.connect_clicked(move |_| on_quit());

    let keys = gtk::EventControllerKey::new();
    keys.connect_key_pressed(glib::clone!(
        #[weak]
        window,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_, key, _, _| {
            if key == gdk::Key::Escape {
                window.set_visible(false);
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        }
    ));
    window.add_controller(keys);
    window.connect_close_request(|window| {
        window.set_visible(false);
        glib::Propagation::Stop
    });
    if layer {
        // A click outside moves keyboard focus away from the layer surface.
        window.connect_is_active_notify(glib::clone!(
            #[weak]
            panel,
            move |window| {
                if !window.is_active() && window.is_visible() {
                    panel.auto_hidden_at.set(Some(Instant::now()));
                    window.set_visible(false);
                }
            }
        ));
    }
    panel
}

impl Panel {
    pub(crate) fn is_visible(&self) -> bool {
        self.window.is_visible()
    }

    /// Opens or closes the panel; returns true when it was opened.
    pub(crate) fn toggle(&self) -> bool {
        if self.window.is_visible() {
            self.window.set_visible(false);
            return false;
        }
        if self
            .auto_hidden_at
            .take()
            .is_some_and(|at| at.elapsed() < Duration::from_millis(800))
        {
            return false;
        }
        self.window.present();
        true
    }

    pub(crate) fn refresh(
        &self,
        language: Language,
        state: PanelState,
        last_success_at: Option<(u32, u32)>,
        config: &Config,
    ) {
        self.state.set_text(&format!("● {}", state.label(language)));
        for (class, active) in [
            ("success", state == PanelState::UpToDate),
            ("error", state == PanelState::Failed),
            ("warning", state == PanelState::NotConfigured),
        ] {
            if active {
                self.state.add_css_class(class);
            } else {
                self.state.remove_css_class(class);
            }
        }
        self.last
            .set_text(&last_sync_text(language, last_success_at));
        self.route.set_text(&route_text(
            &config.local_folder,
            &config.remote,
            &config.remote_folder,
            Some(glib::home_dir().as_path()),
        ));
    }
}
