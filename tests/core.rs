use std::path::PathBuf;

use drivemirror::{Config, SyncCommand};

#[test]
fn defaults_are_conservative() {
    let config = Config::default();
    assert_eq!(config.interval_minutes, 10);
    assert!(config.exclude.contains(&".DS_Store".to_string()));
    assert!(config.exclude.contains(&"*.bak".to_string()));
    assert!(!config.exclude.contains(&"*.sql".to_string()));
}

#[test]
fn config_round_trips_toml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let mut config = Config::default();
    config.local_folder = PathBuf::from("/tmp/source");
    config.remote = "drive".into();
    config.remote_folder = "Backup/My files".into();
    config.save(&path).unwrap();
    assert_eq!(Config::load(&path).unwrap(), config);
}

#[test]
fn validation_rejects_dangerous_destinations_and_short_intervals() {
    let mut config = Config::default();
    config.local_folder = PathBuf::from("/tmp/source");
    config.remote = "drive".into();
    config.remote_folder = "/".into();
    assert!(config.validate().unwrap_err().contains("subfolder"));
    config.remote_folder = "Backup".into();
    config.interval_minutes = 0;
    assert!(config.validate().unwrap_err().contains("one minute"));
}

#[test]
fn command_keeps_arguments_literal_and_never_deletes_excluded_files() {
    let config = Config {
        local_folder: PathBuf::from("/tmp/My folder; touch /tmp/nope"),
        remote: "drive".into(),
        remote_folder: "Private backup".into(),
        interval_minutes: 10,
        exclude: vec!["*.bak; echo nope".into()],
        exclude_folders: vec![".cache/**".into()],
    };
    let command = SyncCommand::from_config(&config).unwrap();
    assert_eq!(command.program, "rclone");
    assert_eq!(&command.args[..3], &["sync", "/tmp/My folder; touch /tmp/nope", "drive:Private backup"]);
    assert!(command.args.windows(2).any(|a| a == ["--exclude", "*.bak; echo nope"]));
    assert!(!command.args.iter().any(|arg| arg == "--delete-excluded"));
}

