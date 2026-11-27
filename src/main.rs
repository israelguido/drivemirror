mod accounts_page;
mod app;
mod theme;
mod tray;
mod tray_panel;

use drivemirror::{Config, default_config_path};
use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    let arguments: Vec<String> = std::env::args().collect();
    let start_hidden = arguments.iter().any(|arg| arg == "--tray");
    let gtk_arguments: Vec<String> = arguments
        .into_iter()
        .filter(|argument| argument != "--tray")
        .collect();
    let application = gtk::Application::builder()
        .application_id("com.github.israelguido.drivemirror")
        .build();
    let (sender, receiver) = std::sync::mpsc::channel();
    let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));
    let language = Config::load(&default_config_path())
        .unwrap_or_default()
        .language
        .current();
    let tray = tray::start(sender.clone(), language);
    let tray_error = tray.as_ref().err().map(|error| error.to_string());
    let tray = tray.ok();
    application.connect_activate(move |app| {
        app::activate(
            app,
            receiver.clone(),
            start_hidden,
            tray_error.clone(),
            tray.clone(),
        )
    });
    application.run_with_args(&gtk_arguments)
}
