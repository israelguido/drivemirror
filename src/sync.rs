use crate::Config;
use std::{
    process::Command,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncCommand {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncResult {
    pub success: bool,
    pub message: String,
}

#[derive(Clone, Default)]
pub struct SyncRunner {
    running: Arc<AtomicBool>,
    result: Arc<Mutex<Option<SyncResult>>>,
}

impl SyncRunner {
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    pub fn last_result(&self) -> Option<SyncResult> {
        self.result.lock().ok()?.clone()
    }

    pub fn start(&self, command: SyncCommand) -> Result<(), String> {
        if self.running.swap(true, Ordering::AcqRel) {
            return Err("A synchronization is already running".into());
        }
        if let Ok(mut result) = self.result.lock() {
            *result = None;
        }
        let running = self.running.clone();
        let result = self.result.clone();
        std::thread::spawn(move || {
            let outcome = Command::new(&command.program).args(&command.args).output();
            let completed = match outcome {
                Ok(output) => {
                    let text = if output.status.success() {
                        &output.stdout
                    } else {
                        &output.stderr
                    };
                    SyncResult {
                        success: output.status.success(),
                        message: String::from_utf8_lossy(text).trim().to_string(),
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => SyncResult {
                    success: false,
                    message: format!(
                        "{} was not found; install it or check PATH",
                        command.program
                    ),
                },
                Err(error) => SyncResult {
                    success: false,
                    message: format!("Could not start {}: {error}", command.program),
                },
            };
            if let Ok(mut value) = result.lock() {
                *value = Some(completed);
            }
            running.store(false, Ordering::Release);
        });
        Ok(())
    }
}

impl SyncCommand {
    pub fn from_config(config: &Config) -> Result<Self, String> {
        config.validate()?;
        let source = config.local_folder.to_string_lossy().into_owned();
        let destination = format!(
            "{}:{}",
            config.remote.trim(),
            config.remote_folder.trim_matches('/')
        );
        let mut args = vec![
            "sync".into(),
            source,
            destination,
            "--stats-one-line".into(),
        ];
        for pattern in config.exclude.iter().chain(config.exclude_folders.iter()) {
            if !pattern.trim().is_empty() {
                args.push("--exclude".into());
                args.push(pattern.clone());
            }
        }
        Ok(Self {
            program: "rclone".into(),
            args,
        })
    }
}
