use std::{
    fs,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub local_folder: PathBuf,
    pub remote: String,
    pub remote_folder: String,
    pub interval_minutes: u64,
    pub exclude: Vec<String>,
    pub exclude_folders: Vec<String>,
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
        }
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let text =
            fs::read_to_string(path).map_err(|error| format!("Could not read config: {error}"))?;
        toml::from_str(&text).map_err(|error| format!("Invalid config: {error}"))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create config folder: {error}"))?;
        }
        let text = toml::to_string_pretty(self)
            .map_err(|error| format!("Could not encode config: {error}"))?;
        let temporary = path.with_extension("toml.tmp");
        fs::write(&temporary, text).map_err(|error| format!("Could not write config: {error}"))?;
        fs::rename(&temporary, path).map_err(|error| format!("Could not replace config: {error}"))
    }

    pub fn save_if_unchanged(&self, path: &Path, baseline: &Self) -> Result<(), String> {
        if path.exists() {
            let current = Self::load(path)?;
            if &current != baseline {
                return Err("The config changed externally; review it before saving".into());
            }
        }
        self.save(path)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.local_folder.is_absolute() || !self.local_folder.is_dir() {
            return Err("Choose an existing absolute local folder".into());
        }
        let remote = self.remote.trim();
        if remote.is_empty()
            || remote == "."
            || remote == ".."
            || !remote.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.' | ' ')
            })
        {
            return Err("Choose a valid rclone remote name".into());
        }
        let folder = self.remote_folder.trim_matches('/').trim();
        if folder.is_empty()
            || folder
                .split('/')
                .any(|component| component.is_empty() || matches!(component, "." | ".."))
        {
            return Err("Choose a remote subfolder; the Drive root is not allowed".into());
        }
        if self.interval_minutes < 1 {
            return Err("The interval must be at least one minute".into());
        }
        if self.interval_minutes > 1440 {
            return Err("The interval cannot exceed 24 hours".into());
        }
        Ok(())
    }
}

pub fn default_config_path() -> PathBuf {
    ProjectDirs::from("com", "DriveMirror", "drivemirror")
        .map(|dirs| dirs.config_dir().join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}
