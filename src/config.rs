use std::{
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::i18n::{Language, Message, tr};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub local_folder: PathBuf,
    pub remote: String,
    pub remote_folder: String,
    pub interval_minutes: u64,
    pub exclude: Vec<String>,
    pub exclude_folders: Vec<String>,
    pub language: Language,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            local_folder: PathBuf::new(),
            remote: String::new(),
            remote_folder: String::new(),
            interval_minutes: 10,
            exclude: vec![
                ".DS_Store",
                "Thumbs.db",
                "desktop.ini",
                "*~",
                "*.swp",
                "*.tmp",
                "*.bak",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            exclude_folders: vec![".Trash-*/**", ".cache/**"]
                .into_iter()
                .map(String::from)
                .collect(),
            language: Language::System,
        }
    }
}

fn error_text(language: Language, message: Message, error: impl std::fmt::Display) -> String {
    tr(language, message).replace("{error}", &error.to_string())
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let language = Language::System.current();
        let text = fs::read_to_string(path)
            .map_err(|error| error_text(language, Message::ReadConfigFailed, error))?;
        toml::from_str(&text).map_err(|error| error_text(language, Message::InvalidConfig, error))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let language = self.language.current();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| error_text(language, Message::CreateConfigFolderFailed, error))?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|error| error_text(language, Message::EncodeConfigFailed, error))?;
        let temporary = path.with_extension("toml.tmp");
        fs::write(&temporary, text)
            .map_err(|error| error_text(language, Message::WriteConfigFailed, error))?;
        fs::rename(&temporary, path)
            .map_err(|error| error_text(language, Message::ReplaceConfigFailed, error))
    }

    pub fn save_if_unchanged(&self, path: &Path, baseline: &Self) -> Result<(), String> {
        if path.exists() {
            let current = Self::load(path)?;
            if &current != baseline {
                return Err(tr(self.language.current(), Message::ConfigChangedExternally).into());
            }
        }
        self.save(path)
    }

    pub fn save_for_sync(&self, path: &Path, baseline: &Self) -> Result<(), String> {
        self.validate()?;
        self.save_if_unchanged(path, baseline)
    }

    pub fn validate(&self) -> Result<(), String> {
        let fail = |message| Err(tr(self.language.current(), message).into());
        if !self.local_folder.is_absolute() || !self.local_folder.is_dir() {
            return fail(Message::InvalidLocalFolder);
        }
        if crate::accounts::validate_remote_name(self.remote.trim()).is_err() {
            return fail(Message::InvalidRemote);
        }
        let folder = self.remote_folder.trim_matches('/').trim();
        if folder.is_empty()
            || folder
                .split('/')
                .any(|component| component.is_empty() || matches!(component, "." | ".."))
        {
            return fail(Message::RemoteRootNotAllowed);
        }
        if self.interval_minutes < 1 {
            return fail(Message::IntervalTooShort);
        }
        if self.interval_minutes > 1440 {
            return fail(Message::IntervalTooLong);
        }
        Ok(())
    }
}

pub fn default_config_path() -> PathBuf {
    ProjectDirs::from("com", "DriveMirror", "drivemirror")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}
