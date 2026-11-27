use std::path::PathBuf;

use drivemirror::{Config, SyncCommand, SyncRunner};

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
    let config = Config {
        local_folder: PathBuf::from("/tmp/source"),
        remote: "drive".into(),
        remote_folder: "Backup/My files".into(),
        ..Default::default()
    };
    config.save(&path).unwrap();
    assert_eq!(Config::load(&path).unwrap(), config);
}

#[test]
fn validation_rejects_dangerous_destinations_and_short_intervals() {
    let source = tempfile::tempdir().unwrap();
    let mut config = Config {
        local_folder: source.path().to_path_buf(),
        remote: "drive".into(),
        remote_folder: "/".into(),
        ..Default::default()
    };
    assert!(config.validate().unwrap_err().contains("subfolder"));
    config.remote_folder = "Backup".into();
    config.interval_minutes = 0;
    assert!(config.validate().unwrap_err().contains("one minute"));
    config.interval_minutes = 1441;
    assert!(config.validate().unwrap_err().contains("24 hours"));
}

#[test]
fn command_keeps_arguments_literal_and_never_deletes_excluded_files() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("My folder; touch nope");
    std::fs::create_dir(&source).unwrap();
    let config = Config {
        local_folder: source.clone(),
        remote: "drive".into(),
        remote_folder: "Private backup".into(),
        interval_minutes: 10,
        exclude: vec!["*.bak; echo nope".into()],
        exclude_folders: vec![".cache/**".into()],
    };
    let command = SyncCommand::from_config(&config).unwrap();
    assert_eq!(command.program, "rclone");
    assert_eq!(command.args[0], "sync");
    assert_eq!(command.args[1], source.to_string_lossy());
    assert_eq!(command.args[2], "drive:Private backup");
    assert!(
        command
            .args
            .windows(2)
            .any(|a| a == ["--exclude", "*.bak; echo nope"])
    );
    assert!(!command.args.iter().any(|arg| arg == "--delete-excluded"));
}

#[test]
fn runner_rejects_a_second_process_and_records_completion() {
    let runner = SyncRunner::default();
    let command = SyncCommand {
        program: "/bin/sh".into(),
        args: vec!["-c".into(), "sleep 0.1; printf synced".into()],
    };
    runner.start(command.clone()).unwrap();
    assert!(runner.is_running());
    assert!(runner.start(command).unwrap_err().contains("already"));
    for _ in 0..50 {
        if !runner.is_running() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(!runner.is_running());
    let result = runner.last_result().unwrap();
    assert!(result.success);
    assert_eq!(result.message, "synced");
}

#[test]
fn validation_requires_a_real_absolute_source_and_safe_remote_path() {
    let source = tempfile::tempdir().unwrap();
    let valid = Config {
        local_folder: source.path().to_path_buf(),
        remote: "gdrive".into(),
        remote_folder: "Backup/Documents".into(),
        ..Default::default()
    };
    assert!(valid.validate().is_ok());

    for local in [PathBuf::from("relative"), source.path().join("missing")] {
        let mut config = valid.clone();
        config.local_folder = local;
        assert!(config.validate().is_err());
    }
    for remote in ["../drive", "drive/path", ":drive"] {
        let mut config = valid.clone();
        config.remote = remote.into();
        assert!(config.validate().is_err());
    }
    for folder in ["..", "Backup/../Other", "Backup/./Other", "//"] {
        let mut config = valid.clone();
        config.remote_folder = folder.into();
        assert!(config.validate().is_err());
    }
}

#[test]
fn missing_executable_has_an_actionable_result() {
    let runner = SyncRunner::default();
    runner
        .start(SyncCommand {
            program: "definitely-missing-drivemirror-test".into(),
            args: vec![],
        })
        .unwrap();
    for _ in 0..50 {
        if !runner.is_running() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let result = runner.last_result().unwrap();
    assert!(!result.success);
    assert!(
        result
            .message
            .contains("definitely-missing-drivemirror-test")
    );
    assert!(result.message.contains("install"));
}

#[test]
fn guarded_save_preserves_invalid_external_edits() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, b"not = [valid").unwrap();
    let config = Config::default();
    assert!(config.save_if_unchanged(&path, &Config::default()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"not = [valid");
}

#[test]
fn manual_sync_persists_the_selected_config_before_running() {
    let source = tempfile::tempdir().unwrap();
    let config_path = source.path().join("settings/config.toml");
    let selected = Config {
        local_folder: source.path().to_path_buf(),
        remote: "gdrive".into(),
        remote_folder: "Backup/Documents".into(),
        ..Default::default()
    };
    selected
        .save_for_sync(&config_path, &Config::default())
        .unwrap();
    assert_eq!(Config::load(&config_path).unwrap(), selected);
}
