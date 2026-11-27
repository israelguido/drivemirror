//! Text for the tray panel summary; pure so it is testable without GTK.

use std::path::Path;

use crate::i18n::{Language, Message, tr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelState {
    UpToDate,
    Syncing,
    Failed,
    NotConfigured,
}

impl PanelState {
    /// `last_success` is the outcome of the latest sync attempt, if any.
    pub fn new(configured: bool, running: bool, last_success: Option<bool>) -> Self {
        if running {
            Self::Syncing
        } else if !configured {
            Self::NotConfigured
        } else if last_success == Some(false) {
            Self::Failed
        } else {
            Self::UpToDate
        }
    }

    pub fn label(self, language: Language) -> &'static str {
        tr(
            language,
            match self {
                Self::UpToDate => Message::PanelUpToDate,
                Self::Syncing => Message::Syncing,
                Self::Failed => Message::PanelFailed,
                Self::NotConfigured => Message::PanelNotConfigured,
            },
        )
    }
}

/// `time` is the local (hour, minute) of the last successful sync.
pub fn last_sync_text(language: Language, time: Option<(u32, u32)>) -> String {
    let time = match time {
        Some((hour, minute)) => format!("{hour:02}:{minute:02}"),
        None => tr(language, Message::Never).into(),
    };
    tr(language, Message::LastSync).replace("{time}", &time)
}

/// `~/local → remote:folder`, or `—` when nothing is configured.
pub fn route_text(local: &Path, remote: &str, remote_folder: &str, home: Option<&Path>) -> String {
    let remote = remote.trim();
    if local.as_os_str().is_empty() && remote.is_empty() {
        return "—".into();
    }
    let local = match home.and_then(|home| local.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".into(),
        Some(rest) => format!("~/{}", rest.display()),
        None => local.display().to_string(),
    };
    format!("{local} → {remote}:{}", remote_folder.trim_matches('/'))
}
