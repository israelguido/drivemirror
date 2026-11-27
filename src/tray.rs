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
                label: "Abrir DriveMirror".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.sender.send(Action::Show);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "Sincronizar agora".into(),
                activate: Box::new(|tray: &mut Self| {
                    let _ = tray.sender.send(Action::Sync);
                }),
                ..Default::default()
            }
            .into(),
            ksni::MenuItem::Separator,
            StandardItem {
                label: "Sair".into(),
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
) -> Result<Handle<DriveMirrorTray>, ksni::Error> {
    DriveMirrorTray { sender }
        .assume_sni_available(true)
        .spawn()
}
