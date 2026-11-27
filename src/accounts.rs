//! Rclone account protocol: literal argv builders and parsers. No process
//! execution and no GTK here; callers run commands off the GTK thread.

use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::fmt;

const RCLONE: &str = "rclone";
const REDACTED: &str = "<redacted>";

/// A literal program + argv. `Debug` redacts the secret argument, if any.
#[derive(Clone, PartialEq, Eq)]
pub struct AccountCommand {
    pub program: String,
    pub args: Vec<String>,
    secret_index: Option<usize>,
}

impl AccountCommand {
    fn new(args: &[&str], secret_index: Option<usize>) -> Self {
        Self {
            program: RCLONE.into(),
            args: args.iter().map(|arg| arg.to_string()).collect(),
            secret_index,
        }
    }
}

impl fmt::Debug for AccountCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<&str> = self
            .args
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                if Some(i) == self.secret_index {
                    REDACTED
                } else {
                    arg
                }
            })
            .collect();
        f.debug_struct("AccountCommand")
            .field("program", &self.program)
            .field("args", &args)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub name: String,
    pub kind: String,
}

/// Rclone's remote-name charset: letters, digits, `_ . + @ -` and inner spaces;
/// it may not start with `-` or a space nor end with a space.
pub fn validate_remote_name(name: &str) -> Result<(), String> {
    let allowed = |c: char| c.is_alphanumeric() || "_.+@- ".contains(c);
    if name.is_empty()
        || name.starts_with(['-', ' '])
        || name.ends_with(' ')
        || !name.chars().all(allowed)
    {
        return Err("Invalid remote name".into());
    }
    Ok(())
}

fn validate_value(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.starts_with('-') {
        return Err(format!("Invalid {label}"));
    }
    Ok(())
}

pub fn list_remotes_command() -> AccountCommand {
    AccountCommand::new(&["listremotes", "--long"], None)
}

/// Parses `rclone listremotes --long` (`name:  type  description`). It never
/// contains tokens, unlike `rclone config dump`.
pub fn list_remotes(output: &str) -> Vec<Remote> {
    output
        .lines()
        .filter_map(|line| {
            let (name, rest) = line.split_once(':')?;
            let kind = rest.split_whitespace().next().unwrap_or_default();
            (!name.is_empty()).then(|| Remote {
                name: name.into(),
                kind: kind.into(),
            })
        })
        .collect()
}

#[derive(Clone, PartialEq, Eq)]
pub struct CreateDriveRequest {
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
}

impl fmt::Debug for CreateDriveRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CreateDriveRequest")
            .field("name", &self.name)
            .field("client_id", &self.client_id)
            .field("client_secret", &REDACTED)
            .finish()
    }
}

impl CreateDriveRequest {
    pub fn command(&self) -> Result<AccountCommand, String> {
        validate_remote_name(&self.name)?;
        validate_value(&self.client_id, "Client ID")?;
        validate_value(&self.client_secret, "Client Secret")?;
        Ok(AccountCommand::new(
            &[
                "config",
                "create",
                &self.name,
                "drive",
                "client_id",
                &self.client_id,
                "client_secret",
                &self.client_secret,
                "scope",
                "drive",
                "--non-interactive",
            ],
            Some(7),
        ))
    }
}

/// Answer to a `ConfigQuestion`. Set `secret` when the option `is_password`.
#[derive(Clone, PartialEq, Eq)]
pub struct ContinueRequest {
    pub name: String,
    pub state: String,
    pub result: String,
    pub secret: bool,
}

impl fmt::Debug for ContinueRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result: &str = if self.secret { REDACTED } else { &self.result };
        f.debug_struct("ContinueRequest")
            .field("name", &self.name)
            .field("state", &self.state)
            .field("result", &result)
            .field("secret", &self.secret)
            .finish()
    }
}

impl ContinueRequest {
    pub fn command(&self) -> Result<AccountCommand, String> {
        validate_remote_name(&self.name)?;
        Ok(AccountCommand::new(
            &[
                "config",
                "update",
                &self.name,
                "--continue",
                "--state",
                &self.state,
                "--result",
                &self.result,
                "--non-interactive",
            ],
            self.secret.then_some(7),
        ))
    }
}

/// Refreshes the OAuth token via the same continuation protocol as create;
/// `rclone config reconnect` is interactive-only.
pub fn reconnect_command(name: &str) -> Result<AccountCommand, String> {
    validate_remote_name(name)?;
    Ok(AccountCommand::new(
        &[
            "config",
            "update",
            name,
            "config_refresh_token=true",
            "--non-interactive",
        ],
        None,
    ))
}

/// Removes the remote from rclone's config only; Drive files are untouched.
pub fn delete_command(name: &str) -> Result<AccountCommand, String> {
    validate_remote_name(name)?;
    Ok(AccountCommand::new(&["config", "delete", name], None))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigExample {
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub help: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigOption {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub help: String,
    /// JSON scalar rendered as rclone expects it back in `--result`.
    #[serde(default, deserialize_with = "scalar_text")]
    pub default: String,
    #[serde(default, deserialize_with = "null_as_empty")]
    pub examples: Vec<ConfigExample>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub is_password: bool,
    #[serde(default)]
    pub exclusive: bool,
    #[serde(default, rename = "Type")]
    pub kind: String,
}

/// One step of `rclone config create/update --non-interactive`. An empty
/// `state` means the process finished.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigQuestion {
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub option: Option<ConfigOption>,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub result: String,
}

impl ConfigQuestion {
    pub fn is_done(&self) -> bool {
        self.state.is_empty()
    }

    /// Errors never include the input: rclone output may carry secrets.
    pub fn parse(text: &str) -> Result<Self, String> {
        serde_json::from_str(text).map_err(|_| "Malformed rclone configuration response".into())
    }
}

fn scalar_text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::String(text) => text,
        Value::Null => String::new(),
        other => other.to_string(),
    })
}

fn null_as_empty<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<ConfigExample>, D::Error> {
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}
