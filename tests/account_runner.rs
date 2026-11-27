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
    assert_eq!(question.option.unwrap().name, "config_is_local");
    let answer = ContinueRequest {
        name: "gdrive".into(),
        state: question.state,
        result: "true".into(),
        secret: false,
    };
    runner
        .start(fake(answer.command().unwrap(), &program), Language::En)
        .unwrap();
    let AccountResult::Finished(output) = wait(&runner) else {
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
    runner.start(create("slow", &program), Language::En).unwrap();
    assert!(
        runner
            .start(create("slow", &program), Language::En)
            .unwrap_err()
            .contains("already")
    );
    runner.cancel();
    assert!(matches!(wait(&runner), AccountResult::Cancelled));

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
