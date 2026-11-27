mod app;
mod theme;
mod tray;

use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    let start_hidden = std::env::args().any(|arg| arg == "--tray");
    let application = gtk::Application::builder()
        .application_id("com.github.israelguido.drivemirror")
        .build();
    let (sender, receiver) = std::sync::mpsc::channel();
    let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));
    let _tray =
        tray::start(sender.clone()).map_err(|error| eprintln!("Bandeja indisponível: {error}"));
    application.connect_activate(move |app| app::activate(app, receiver.clone(), start_hidden));
    application.run()
}
