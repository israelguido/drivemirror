use drivemirror::accounts::{
    AnswerProblem, ConfigQuestion, ContinueRequest, CreateDriveRequest, Remote, RemoteProblem,
    config_dump_command, delete_command, incomplete_drive_remotes, list_remotes,
    list_remotes_command, reconnect_command, remote_problem, validate_remote_name,
};

const SECRET: &str = "GOCSPX-very-secret-value";

#[test]
fn lists_remotes_from_long_output() {
    let output = "gdrive:        drive Personal\nmy backup.v2:  s3\n\nweird-name@x:  drive\n";
    assert_eq!(
        list_remotes(output),
        vec![
            Remote {
                name: "gdrive".into(),
                kind: "drive".into()
            },
            Remote {
                name: "my backup.v2".into(),
                kind: "s3".into()
            },
            Remote {
                name: "weird-name@x".into(),
                kind: "drive".into()
            },
        ]
    );
    let list = list_remotes_command();
    assert_eq!(list.program, "rclone");
    assert_eq!(list.args, ["listremotes", "--long"]);
}

#[test]
fn parses_documented_question_json() {
    // Example from `rclone config create --help` (as valid JSON, which rclone emits).
    let json = r#"{
      "State": "*oauth-islocal,teamdrive,,",
      "Option": {
        "Name": "config_is_local",
        "Help": "Use web browser?\n * Say Y & go",
        "Default": true,
        "Examples": [{"Value": "true", "Help": "Yes"}, {"Value": "false", "Help": "No"}],
        "Required": false,
        "IsPassword": false,
        "Type": "bool",
        "Exclusive": true,
        "Hide": 0
      },
      "Error": ""
    }"#;
    let question = ConfigQuestion::parse(json).unwrap();
    assert_eq!(question.state, "*oauth-islocal,teamdrive,,");
    assert!(!question.is_done());
    let option = question.option.unwrap();
    assert_eq!(option.name, "config_is_local");
    assert_eq!(option.help, "Use web browser?\n * Say Y & go");
    assert_eq!(option.default, "true");
    assert_eq!(option.examples.len(), 2);
    assert_eq!(option.examples[1].value, "false");
    assert_eq!(option.examples[1].help, "No");
    assert!(!option.required && !option.is_password && option.exclusive);
    assert_eq!(option.kind, "bool");
}

#[test]
fn parses_final_answer_and_error() {
    let question =
        ConfigQuestion::parse(r#"{"State":"","Option":{"Name":"n","Default":null,"Examples":null},"Error":"bad \"id\"","Result":"ok"}"#)
            .unwrap();
    assert!(question.is_done());
    let option = question.option.unwrap();
    assert_eq!(option.default, "");
    assert!(option.examples.is_empty());
    assert_eq!(question.error, "bad \"id\"");
    assert_eq!(question.result, "ok");
}

#[test]
fn malformed_json_errors_without_echoing_input() {
    for bad in [
        format!(r#"{{"State": "x", "Option": {{"Default": "{SECRET}""#),
        format!("token {SECRET}"),
        String::new(),
        r#"["not", "an", "object"]"#.to_string(),
    ] {
        let error = ConfigQuestion::parse(&bad).unwrap_err();
        assert!(!error.contains(SECRET), "{error}");
        assert!(!error.contains("token"), "{error}");
    }
}

#[test]
fn create_drive_command_is_literal_and_debug_is_redacted() {
    let request = CreateDriveRequest {
        name: "gdrive".into(),
        client_id: "123.apps.googleusercontent.com".into(),
        client_secret: SECRET.into(),
    };
    assert!(!format!("{request:?}").contains(SECRET));
    let command = request.command().unwrap();
    assert_eq!(command.program, "rclone");
    assert_eq!(
        command.args,
        [
            "config",
            "create",
            "gdrive",
            "drive",
            "client_id",
            "123.apps.googleusercontent.com",
            "client_secret",
            SECRET,
            "scope",
            "drive",
            "--non-interactive",
        ]
    );
    let debug = format!("{command:?}");
    assert!(!debug.contains(SECRET), "{debug}");
    assert!(debug.contains("client_secret"));
}

#[test]
fn create_drive_rejects_invalid_input() {
    let base = CreateDriveRequest {
        name: "gdrive".into(),
        client_id: "id".into(),
        client_secret: SECRET.into(),
    };
    for request in [
        CreateDriveRequest {
            name: "-x".into(),
            ..base.clone()
        },
        CreateDriveRequest {
            client_id: String::new(),
            ..base.clone()
        },
        CreateDriveRequest {
            client_id: "--all".into(),
            ..base.clone()
        },
        CreateDriveRequest {
            client_secret: " ".into(),
            ..base.clone()
        },
        CreateDriveRequest {
            client_secret: "-secret".into(),
            ..base.clone()
        },
    ] {
        let error = request.command().unwrap_err();
        assert!(!error.contains(SECRET));
    }
}

fn question(state: &str, is_password: bool) -> ConfigQuestion {
    ConfigQuestion::parse(&format!(
        r#"{{"State": {state:?}, "Option": {{"Name": "x", "IsPassword": {is_password}, "Default": "{SECRET}"}}, "Error": "", "Result": "{SECRET}"}}"#
    ))
    .unwrap()
}

fn creation() -> CreateDriveRequest {
    CreateDriveRequest {
        name: "gdrive".into(),
        client_id: "id.apps".into(),
        client_secret: SECRET.into(),
    }
}

#[test]
fn continue_arguments_stay_literal_and_repeat_creation_values() {
    let tricky = r#"a b "c" 'd'; rm -rf ~ $(x) -flag"#;
    let request = ContinueRequest::answer(
        "gdrive",
        Some(&creation()),
        &question(tricky, false),
        tricky,
    )
    .unwrap();
    let command = request.command().unwrap();
    assert_eq!(command.program, "rclone");
    assert_eq!(
        command.args,
        [
            "config",
            "update",
            "gdrive",
            "client_id",
            "id.apps",
            "client_secret",
            SECRET,
            "scope",
            "drive",
            "--continue",
            "--state",
            tricky,
            "--result",
            tricky,
            "--non-interactive",
        ]
    );
    let debug = format!("{request:?} {command:?}");
    assert!(!debug.contains(SECRET), "{debug}");
    assert!(debug.contains("rm -rf ~ $(x) -flag"));
}

#[test]
fn reconnect_continuation_has_no_creation_values() {
    let request = ContinueRequest::answer("gdrive", None, &question("*s", false), "true").unwrap();
    assert_eq!(
        request.command().unwrap().args,
        [
            "config",
            "update",
            "gdrive",
            "--continue",
            "--state",
            "*s",
            "--result",
            "true",
            "--non-interactive"
        ]
    );
}

#[test]
fn password_answers_are_redacted_from_the_question() {
    let password = question("*pass", true);
    let debug = format!("{password:?}");
    assert!(!debug.contains(SECRET), "{debug}");
    let request = ContinueRequest::answer("gdrive", None, &password, SECRET).unwrap();
    let command = request.command().unwrap();
    assert_eq!(command.args[7], SECRET);
    assert!(!format!("{request:?} {command:?}").contains(SECRET));
}

#[test]
fn answers_need_a_pending_question_and_matching_creation() {
    assert!(ContinueRequest::answer("gdrive", None, &question("", false), "x").is_err());
    assert!(
        ContinueRequest::answer("other", Some(&creation()), &question("*s", false), "x").is_err()
    );
    assert!(ContinueRequest::answer("-bad", None, &question("*s", false), "x").is_err());
}

#[test]
fn parse_requires_state() {
    assert!(ConfigQuestion::parse("{}").is_err());
}

#[test]
fn reconnect_and_delete_commands() {
    let reconnect = reconnect_command("my drive").unwrap();
    assert_eq!(reconnect.program, "rclone");
    assert_eq!(
        reconnect.args,
        [
            "config",
            "update",
            "my drive",
            "config_refresh_token=true",
            "--non-interactive"
        ]
    );
    let delete = delete_command("my drive").unwrap();
    assert_eq!(delete.program, "rclone");
    assert_eq!(delete.args, ["config", "delete", "my drive"]);
}

#[test]
fn remote_names_follow_rclone_charset() {
    for good in ["gdrive", "a", "my drive", "x-y_z.w+v@u", "Café2"] {
        assert!(validate_remote_name(good).is_ok(), "{good}");
    }
    for bad in ["", "-x", " x", "x ", "a:b", "a/b", "a;b", "a\nb", "a\"b"] {
        assert!(validate_remote_name(bad).is_err(), "{bad:?}");
        assert!(reconnect_command(bad).is_err());
        assert!(delete_command(bad).is_err());
    }
}

#[test]
fn question_debug_redacts_error_and_password_examples() {
    let question = ConfigQuestion::parse(&format!(
        r#"{{"State": "*p", "Option": {{"Name": "x", "IsPassword": true, "Examples": [{{"Value": "{SECRET}", "Help": "h"}}]}}, "Error": "bad {SECRET}"}}"#
    ))
    .unwrap();
    let debug = format!("{question:?}");
    assert!(!debug.contains(SECRET), "{debug}");
    assert_eq!(question.error, format!("bad {SECRET}"));
    let plain = ConfigQuestion::parse(
        r#"{"State": "*p", "Option": {"Name": "x", "Examples": [{"Value": "visible"}]}}"#,
    )
    .unwrap();
    assert!(format!("{plain:?}").contains("visible"));
}

#[test]
fn detects_drive_remotes_without_token_and_never_returns_values() {
    let dump = format!(
        r#"{{
          "ok": {{"type": "drive", "client_secret": "{SECRET}", "token": "{{\"access_token\":\"ya29.fake-token\"}}"}},
          "half": {{"type": "drive", "client_id": "id", "client_secret": "{SECRET}"}},
          "empty": {{"type": "drive", "token": ""}},
          "s3box": {{"type": "s3", "secret_access_key": "{SECRET}"}}
        }}"#
    );
    let mut incomplete = incomplete_drive_remotes(&dump).unwrap();
    incomplete.sort();
    assert_eq!(incomplete, ["empty", "half"]);
    let debug = format!("{incomplete:?}");
    assert!(
        !debug.contains(SECRET) && !debug.contains("ya29"),
        "{debug}"
    );
    let error = incomplete_drive_remotes(&format!("{{\"x\": {SECRET}")).unwrap_err();
    assert!(!error.contains(SECRET), "{error}");
    assert_eq!(config_dump_command().args, ["config", "dump"]);
}

#[test]
fn answers_respect_required_and_exclusive() {
    let option = |json: &str| {
        ConfigQuestion::parse(&format!(r#"{{"State": "*s", "Option": {json}}}"#))
            .unwrap()
            .option
            .unwrap()
    };
    let required = option(r#"{"Name": "x", "Required": true}"#);
    assert_eq!(required.check_answer(""), Err(AnswerProblem::Required));
    assert_eq!(required.check_answer("v"), Ok(()));
    let optional = option(r#"{"Name": "x"}"#);
    assert_eq!(optional.check_answer(""), Ok(()));
    let exclusive = option(
        r#"{"Name": "x", "Exclusive": true, "Examples": [{"Value": "true"}, {"Value": "false"}]}"#,
    );
    assert_eq!(exclusive.check_answer("true"), Ok(()));
    assert_eq!(
        exclusive.check_answer("maybe"),
        Err(AnswerProblem::NotAnOption)
    );
}

#[test]
fn sync_remote_must_exist_when_the_list_is_known() {
    let known = vec!["gdrive".to_string(), "half".to_string()];
    let incomplete = vec!["half".to_string()];
    assert_eq!(remote_problem("gdrive", Some(&known), &incomplete), None);
    assert_eq!(
        remote_problem("missing", Some(&known), &incomplete),
        Some(RemoteProblem::Missing)
    );
    assert_eq!(
        remote_problem("half", Some(&known), &incomplete),
        Some(RemoteProblem::Incomplete)
    );
    assert_eq!(
        remote_problem("gdrive", Some(&[]), &[]),
        Some(RemoteProblem::Missing)
    );
    // Unknown list (not loaded yet or listing failed): never block on existence.
    assert_eq!(remote_problem("missing", None, &[]), None);
    assert_eq!(
        remote_problem("half", None, &incomplete),
        Some(RemoteProblem::Incomplete)
    );
}
