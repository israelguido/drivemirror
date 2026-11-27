use drivemirror::i18n::{Language, Message, tr};
use ksni::{
    Tray,
    blocking::{Handle, TrayMethods},
    menu::StandardItem,
};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Show,
    Sync,
    Quit,
}

pub(crate) struct DriveMirrorTray {
    sender: std::sync::mpsc::Sender<Action>,
    pub(crate) language: Language,
}

impl Tray for DriveMirrorTray {
    fn id(&self) -> String {
        "drivemirror".into()
    }
    fn title(&self) -> String {
        "DriveMirror".into()
    }
    fn icon_name(&self) -> String {
        "drivemirror".into()
    }
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(Action::Show);
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        vec![
            StandardItem {
                label: tr(self.language, Message::OpenApp).into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.sender.send(Action::Show);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: tr(self.language, Message::SyncNow).into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.sender.send(Action::Sync);
                }),
                ..Default::default()
            }
            .into(),
            ksni::MenuItem::Separator,
            StandardItem {
                label: tr(self.language, Message::Quit).into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.sender.send(Action::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn start(
    sender: std::sync::mpsc::Sender<Action>,
    language: Language,
) -> Result<Handle<DriveMirrorTray>, ksni::Error> {
    DriveMirrorTray { sender, language }.spawn()
}
