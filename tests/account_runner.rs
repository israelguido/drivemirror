use std::{os::unix::fs::PermissionsExt, path::Path, time::Duration};

use drivemirror::{
    account_runner::{AccountResult, AccountRunner},
    accounts::{AccountCommand, ConfigQuestion, ContinueRequest, CreateDriveRequest},
    i18n::Language,
};

const SECRET: &str = "GOCSPX-fake-secret-123";

const FAKE_RCLONE: &str = r#"#!/bin/sh
case "$*" in
  *--continue*--result\ true*) printf '{"State":"","Result":"done"}' ;;
  *failing*) echo "error: bad credentials for $*" >&2; exit 1 ;;
  *slow*) exec sleep 30 ;;
  *authwait*) echo "NOTICE: see https://rclone.org/drive/ for help" >&2
     echo "Please go to http://127.0.0.1:53682/auth?state=$8 to log in" >&2
     sleep 1 ;;
  *) echo "creating with $*" >&2
     printf '{"State":"*oauth,1","Option":{"Name":"config_is_local","Help":"Use web browser? %s","Default":true,"Examples":[{"Value":"true","Help":"Yes"}],"Type":"bool"}}' "$*" ;;
esac
"#;

fn wait(runner: &AccountRunner) -> AccountResult {
    for _ in 0..500 {
        if let Some(result) = runner.take_result() {
            return result;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("fake rclone did not finish");
}

fn fake(mut command: AccountCommand, program: &Path) -> AccountCommand {
    command.program = program.to_string_lossy().into_owned();
    command
}

fn create(name: &str, program: &Path) -> AccountCommand {
    let request = CreateDriveRequest {
        name: name.into(),
        client_id: "id.apps.googleusercontent.com".into(),
        client_secret: SECRET.into(),
    };
    fake(request.command().unwrap(), program)
}

fn assert_no_secret(result: &AccountResult) {
    let text = format!("{result:?}");
    assert!(!text.contains(SECRET), "secret leaked: {text}");
}

// One test so no other test thread forks while the script is open for writing (ETXTBSY).
#[test]
fn account_process_lifecycle_with_fake_rclone() {
    let dir = tempfile::tempdir().unwrap();
    let program = dir.path().join("rclone");
    std::fs::write(&program, FAKE_RCLONE).unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
    let runner = AccountRunner::default();

    // Question -> continuation -> done, with the secret redacted from every output.
    runner
        .start(create("gdrive", &program), Language::En)
        .unwrap();
    assert!(runner.is_running());
    let result = wait(&runner);
    assert_no_secret(&result);
    let AccountResult::Finished(output) = result else {
        panic!("expected output")
    };
    assert!(output.success);
    assert!(output.stderr.contains("<redacted>"));
    let question = ConfigQuestion::parse(&output.stdout).unwrap();
    assert!(!question.is_done());
    assert_eq!(question.option.as_ref().unwrap().name, "config_is_local");
    let creation = CreateDriveRequest {
        name: "gdrive".into(),
        client_id: "id.apps.googleusercontent.com".into(),
        client_secret: SECRET.into(),
    };
    let answer = ContinueRequest::answer("gdrive", Some(&creation), &question, "true").unwrap();
    runner
        .start(fake(answer.command().unwrap(), &program), Language::En)
        .unwrap();
    let result = wait(&runner);
    assert_no_secret(&result);
    let AccountResult::Finished(output) = result else {
        panic!("expected output")
    };
    assert!(ConfigQuestion::parse(&output.stdout).unwrap().is_done());

    // Failure keeps stderr visible but redacted.
    runner
        .start(create("failing", &program), Language::En)
        .unwrap();
    let result = wait(&runner);
    assert_no_secret(&result);
    let AccountResult::Finished(output) = result else {
        panic!("expected output")
    };
    assert!(!output.success);
    assert!(output.stderr.contains("bad credentials"));

    // Cancel kills the child (e.g. while waiting for browser authorization).
    runner
        .start(create("slow", &program), Language::En)
        .unwrap();
    assert!(
        runner
            .start(create("slow", &program), Language::En)
            .unwrap_err()
            .contains("already")
    );
    runner.cancel();
    assert!(matches!(wait(&runner), AccountResult::Cancelled));

    // The OAuth URL arrives while rclone still waits, redacted, preferring the local one.
    runner
        .start(create("authwait", &program), Language::En)
        .unwrap();
    let mut url = None;
    for _ in 0..80 {
        url = runner.auth_url();
        if url.as_deref().is_some_and(|url| url.contains("127.0.0.1")) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(runner.is_running(), "URL must arrive before rclone exits");
    assert_eq!(
        url.as_deref(),
        Some("http://127.0.0.1:53682/auth?state=<redacted>")
    );
    let result = wait(&runner);
    assert_no_secret(&result);
    assert_eq!(runner.auth_url(), None);

    // Missing rclone is actionable.
    let missing = fake(create("gdrive", &program), Path::new("missing-rclone-test"));
    runner.start(missing, Language::En).unwrap();
    let result = wait(&runner);
    assert_no_secret(&result);
    let AccountResult::Failed(message) = result else {
        panic!("expected failure")
    };
    assert!(message.contains("install"));
}
