//! Runs one rclone account command at a time off the GTK thread, with cancel.

use std::{
    fmt,
    io::{BufRead, BufReader, Read},
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
    auth_url: Arc<Mutex<Option<String>>>,
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

    /// The OAuth URL rclone printed while it is still running (redacted), so the
    /// user can open it when the browser did not.
    pub fn auth_url(&self) -> Option<String> {
        self.auth_url.lock().ok()?.clone()
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
            if let Ok(mut url) = runner.auth_url.lock() {
                *url = None;
            }
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
        let auth_url = self.auth_url.clone();
        let stderr_secrets: Vec<String> = secrets.iter().map(|secret| secret.to_string()).collect();
        let stderr = std::thread::spawn(move || {
            let secrets: Vec<&str> = stderr_secrets.iter().map(String::as_str).collect();
            read_lines(stderr, |line| {
                if let Some(url) = find_url(line)
                    && let Ok(mut current) = auth_url.lock()
                    && !current.as_deref().is_some_and(is_local)
                {
                    *current = Some(redact(url, &secrets));
                }
            })
        });
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

/// Reads every line as it arrives, calling `on_line` before the process exits.
fn read_lines(pipe: Option<impl Read>, mut on_line: impl FnMut(&str)) -> String {
    let mut text = String::new();
    if let Some(pipe) = pipe {
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            on_line(&line);
            text.push_str(&line);
            text.push('\n');
        }
    }
    text
}

fn is_local(url: &str) -> bool {
    url.starts_with("http://127.0.0.1")
}

/// rclone's local OAuth URL wins over other https links (e.g. documentation).
fn find_url(line: &str) -> Option<&str> {
    let start = line
        .find("http://127.0.0.1")
        .or_else(|| line.find("https://"))?;
    line[start..].split_whitespace().next()
}
