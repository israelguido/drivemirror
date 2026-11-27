use std::path::PathBuf;

use drivemirror::{
    Config, SyncCommand, SyncRunner,
    i18n::{Language, Message, tr},
};

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
        language: Language::En,
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
        language: Language::En,
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
    runner.start(command.clone(), Language::En).unwrap();
    assert!(runner.is_running());
    assert!(
        runner
            .start(command, Language::En)
            .unwrap_err()
            .contains("already")
    );
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
        .start(
            SyncCommand {
                program: "definitely-missing-drivemirror-test".into(),
                args: vec![],
            },
            Language::En,
        )
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

#[test]
fn legacy_toml_without_language_loads_as_system() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "remote = \"gdrive\"\ninterval_minutes = 5\n").unwrap();
    let config = Config::load(&path).unwrap();
    assert_eq!(config.language, Language::System);
    assert_eq!(config.remote, "gdrive");
}

#[test]
fn language_is_persisted_by_code_and_invalid_values_fall_back_to_system() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let config = Config {
        language: Language::Es,
        ..Default::default()
    };
    config.save(&path).unwrap();
    assert!(
        std::fs::read_to_string(&path)
            .unwrap()
            .contains("language = \"es\"")
    );
    assert_eq!(Config::load(&path).unwrap().language, Language::Es);
    std::fs::write(&path, "language = \"klingon\"\n").unwrap();
    assert_eq!(Config::load(&path).unwrap().language, Language::System);
}

#[test]
fn system_language_follows_the_locale() {
    let cases = [
        ("pt_BR.UTF-8", "pt-BR"),
        ("pt", "pt-BR"),
        ("es-MX", "es"),
        ("es_ES.UTF-8", "es"),
        ("en_US.UTF-8", "en"),
        ("fr_FR.UTF-8", "en"),
        ("C", "en"),
        ("", "en"),
    ];
    for (locale, code) in cases {
        assert_eq!(Language::System.resolve(locale).code(), code, "{locale}");
    }
    assert_eq!(Language::Es.resolve("pt_BR.UTF-8"), Language::Es);
    assert_eq!(Language::System.code(), "system");
}

#[test]
fn messages_exist_in_all_three_languages() {
    assert_eq!(tr(Language::PtBr, Message::SyncNow), "Sincronizar agora");
    assert_eq!(tr(Language::En, Message::SyncNow), "Sync now");
    assert_eq!(tr(Language::Es, Message::SyncNow), "Sincronizar ahora");
    assert_eq!(tr(Language::PtBr, Message::Quit), "Sair");
    assert_eq!(tr(Language::En, Message::Quit), "Quit");
    assert_eq!(tr(Language::Es, Message::Quit), "Salir");
    for language in [Language::PtBr, Language::En, Language::Es] {
        assert!(!tr(language, Message::RemoteRootNotAllowed).is_empty());
    }
}

#[test]
fn validation_errors_use_the_configured_language() {
    let source = tempfile::tempdir().unwrap();
    let config = Config {
        local_folder: source.path().to_path_buf(),
        remote: "drive".into(),
        remote_folder: "/".into(),
        language: Language::PtBr,
        ..Default::default()
    };
    assert_eq!(
        config.validate().unwrap_err(),
        tr(Language::PtBr, Message::RemoteRootNotAllowed)
    );
}

#[test]
fn config_shares_the_account_remote_name_rules() {
    let source = tempfile::tempdir().unwrap();
    let mut config = Config {
        local_folder: source.path().to_path_buf(),
        remote: "-v".into(),
        remote_folder: "Backup".into(),
        ..Default::default()
    };
    assert!(config.validate().is_err());
    config.remote = "my+drive@home".into();
    assert!(drivemirror::accounts::validate_remote_name(&config.remote).is_ok());
    assert!(config.validate().is_ok());
}
