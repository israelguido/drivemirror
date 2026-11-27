//! Runs one rclone account command at a time off the GTK thread, with cancel.

use std::{
    fmt,
    io::Read,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    accounts::AccountCommand,
    i18n::{Language, Message, tr},
};

/// Output is already redacted; it never contains the command's secret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountResult {
    Finished(AccountOutput),
    Cancelled,
    /// rclone could not be started; translated and redacted.
    Failed(String),
}

#[derive(Clone, Default)]
pub struct AccountRunner {
    running: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    child: Arc<Mutex<Option<Child>>>,
    result: Arc<Mutex<Option<AccountResult>>>,
}

impl fmt::Debug for AccountRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountRunner")
            .field("running", &self.is_running())
            .finish()
    }
}

/// Replaces every occurrence of each non-empty secret with `<redacted>`.
pub fn redact(text: &str, secrets: &[&str]) -> String {
    secrets
        .iter()
        .filter(|secret| !secret.is_empty())
        .fold(text.into(), |text, secret| {
            text.replace(secret, "<redacted>")
        })
}

impl AccountRunner {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    /// The finished result, once; `None` while running or after it was taken.
    pub fn take_result(&self) -> Option<AccountResult> {
        self.result.lock().ok()?.take()
    }

    /// Kills the running rclone, e.g. while it waits for browser authorization.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        if let Ok(mut child) = self.child.lock()
            && let Some(child) = child.as_mut()
        {
            let _ = child.kill();
        }
    }

    pub fn start(&self, command: AccountCommand, language: Language) -> Result<(), String> {
        let language = language.current();
        if self.running.swap(true, Ordering::AcqRel) {
            return Err(tr(language, Message::AlreadyRunning).into());
        }
        self.cancelled.store(false, Ordering::Release);
        if let Ok(mut result) = self.result.lock() {
            *result = None;
        }
        let runner = self.clone();
        std::thread::spawn(move || {
            let completed = runner.run(&command, language);
            if let Ok(mut value) = runner.result.lock() {
                *value = Some(completed);
            }
            runner.running.store(false, Ordering::Release);
        });
        Ok(())
    }

    fn run(&self, command: &AccountCommand, language: Language) -> AccountResult {
        let secrets = command.secrets();
        let spawned = Command::new(&command.program)
            .args(&command.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();
        let mut child = match spawned {
            Ok(child) => child,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return AccountResult::Failed(
                    tr(language, Message::ProgramNotFound).replace("{program}", &command.program),
                );
            }
            Err(error) => {
                return AccountResult::Failed(redact(
                    &tr(language, Message::StartFailed)
                        .replace("{program}", &command.program)
                        .replace("{error}", &error.to_string()),
                    &secrets,
                ));
            }
        };
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        if let Ok(mut slot) = self.child.lock() {
            *slot = Some(child);
            if self.cancelled.load(Ordering::Acquire)
                && let Some(child) = slot.as_mut()
            {
                let _ = child.kill();
            }
        }
        // stderr on its own thread so neither pipe can fill up and block rclone.
        let stderr = std::thread::spawn(move || read_all(stderr));
        let stdout = read_all(stdout);
        let stderr = stderr.join().unwrap_or_default();
        let status = self
            .child
            .lock()
            .ok()
            .and_then(|mut slot| slot.take())
            .and_then(|mut child| child.wait().ok());
        if self.cancelled.load(Ordering::Acquire) {
            return AccountResult::Cancelled;
        }
        AccountResult::Finished(AccountOutput {
            success: status.is_some_and(|status| status.success()),
            stdout: redact(stdout.trim(), &secrets),
            stderr: redact(stderr.trim(), &secrets),
        })
    }
}

fn read_all(pipe: Option<impl Read>) -> String {
    let mut text = String::new();
    if let Some(mut pipe) = pipe {
        let _ = pipe.read_to_string(&mut text);
    }
    text
}
