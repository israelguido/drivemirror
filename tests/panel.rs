use std::path::Path;

use drivemirror::{
    i18n::Language,
    panel::{PanelState, last_sync_text, route_text},
};

#[test]
fn state_prefers_running_then_configuration_then_last_result() {
    assert_eq!(PanelState::new(false, true, None), PanelState::Syncing);
    assert_eq!(
        PanelState::new(false, false, Some(true)),
        PanelState::NotConfigured
    );
    assert_eq!(
        PanelState::new(true, false, Some(false)),
        PanelState::Failed
    );
    assert_eq!(
        PanelState::new(true, false, Some(true)),
        PanelState::UpToDate
    );
    assert_eq!(PanelState::new(true, false, None), PanelState::UpToDate);
    assert_eq!(PanelState::UpToDate.label(Language::PtBr), "Em dia");
    assert_eq!(PanelState::Failed.label(Language::En), "Failed");
    assert_eq!(
        PanelState::NotConfigured.label(Language::Es),
        "No configurado"
    );
    assert_eq!(PanelState::Syncing.label(Language::PtBr), "Sincronizando…");
}

#[test]
fn last_sync_is_never_or_zero_padded_time() {
    assert_eq!(last_sync_text(Language::PtBr, None), "Último sync: nunca");
    assert_eq!(
        last_sync_text(Language::En, Some((9, 5))),
        "Last sync: 09:05"
    );
    assert_eq!(
        last_sync_text(Language::Es, Some((23, 59))),
        "Última sincronización: 23:59"
    );
}

#[test]
fn route_abbreviates_home_and_joins_remote_folder() {
    let home = Some(Path::new("/home/ana"));
    assert_eq!(
        route_text(
            Path::new("/home/ana/Notes/Obsidian"),
            "gdrive",
            "/Backup/Obsidian/",
            home
        ),
        "~/Notes/Obsidian → gdrive:Backup/Obsidian"
    );
    assert_eq!(
        route_text(Path::new("/home/ana"), " gdrive ", "B", home),
        "~ → gdrive:B"
    );
    // A sibling like /home/anabel is not under /home/ana.
    assert_eq!(
        route_text(Path::new("/home/anabel/x"), "gdrive", "B", home),
        "/home/anabel/x → gdrive:B"
    );
    assert_eq!(
        route_text(Path::new("/srv/x"), "gdrive", "B", None),
        "/srv/x → gdrive:B"
    );
    assert_eq!(route_text(Path::new(""), "", "", home), "—");
}
