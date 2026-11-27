use crate::Config;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl SyncCommand {
    pub fn from_config(config: &Config) -> Result<Self, String> {
        config.validate()?;
        let source = config.local_folder.to_string_lossy().into_owned();
        let destination = format!("{}:{}", config.remote.trim(), config.remote_folder.trim_matches('/'));
        let mut args = vec!["sync".into(), source, destination, "--stats-one-line".into()];
        for pattern in config.exclude.iter().chain(config.exclude_folders.iter()) {
            if !pattern.trim().is_empty() {
                args.push("--exclude".into());
                args.push(pattern.clone());
            }
        }
        Ok(Self { program: "rclone".into(), args })
    }
}

