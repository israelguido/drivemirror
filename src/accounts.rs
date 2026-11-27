//! Rclone account protocol: literal argv builders and parsers. No process
//! execution and no GTK here; callers run commands off the GTK thread.

use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::fmt;

const RCLONE: &str = "rclone";
const REDACTED: &str = "<redacted>";

/// A literal program + argv. `Debug` redacts the secret arguments.
#[derive(Clone, PartialEq, Eq)]
pub struct AccountCommand {
    pub program: String,
    pub args: Vec<String>,
    secret_indices: Vec<usize>,
}

impl AccountCommand {
    fn new(args: &[&str], secret_indices: Vec<usize>) -> Self {
        Self {
            program: RCLONE.into(),
            args: args.iter().map(|arg| arg.to_string()).collect(),
            secret_indices,
        }
    }

    /// The secret arguments, so a runner can redact them from rclone output.
    pub fn secrets(&self) -> Vec<&str> {
        self.secret_indices
            .iter()
            .filter_map(|index| self.args.get(*index))
            .map(String::as_str)
            .collect()
    }
}

impl fmt::Debug for AccountCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args: Vec<&str> = self
            .args
            .iter()
            .enumerate()
            .map(|(i, arg)| {
                if self.secret_indices.contains(&i) {
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
/// it may not start with `-` or a space nor end with a space. Shared with
/// `Config::validate` so a remote created here is accepted for sync.
pub fn validate_remote_name(name: &str) -> Result<(), String> {
    let allowed = |c: char| c.is_alphanumeric() || "_.+@- ".contains(c);
    if name.is_empty()
        || name == "."
        || name == ".."
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
    AccountCommand::new(&["listremotes", "--long"], vec![])
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
    /// Config pairs sent on create and repeated on every `--continue`, as
    /// `rclone config create --help` requires. The secret is the 4th element.
    fn pairs(&self) -> Result<[&str; 6], String> {
        validate_remote_name(&self.name)?;
        validate_value(&self.client_id, "Client ID")?;
        validate_value(&self.client_secret, "Client Secret")?;
        Ok([
            "client_id",
            &self.client_id,
            "client_secret",
            &self.client_secret,
            "scope",
            "drive",
        ])
    }

    pub fn command(&self) -> Result<AccountCommand, String> {
        let mut args = vec!["config", "create", &self.name, "drive"];
        args.extend(self.pairs()?);
        args.push("--non-interactive");
        Ok(AccountCommand::new(&args, vec![7]))
    }
}

/// Answer to a pending `ConfigQuestion`; build it with `answer`, which marks
/// the result secret when the option is a password.
#[derive(Clone, PartialEq, Eq)]
pub struct ContinueRequest {
    name: String,
    creation: Option<CreateDriveRequest>,
    state: String,
    result: String,
    secret: bool,
}

impl fmt::Debug for ContinueRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result: &str = if self.secret { REDACTED } else { &self.result };
        f.debug_struct("ContinueRequest")
            .field("name", &self.name)
            .field("creation", &self.creation)
            .field("state", &self.state)
            .field("result", &result)
            .finish()
    }
}

impl ContinueRequest {
    /// `creation` is the request that started this flow (None for reconnect,
    /// whose values are already in rclone.conf).
    pub fn answer(
        name: &str,
        creation: Option<&CreateDriveRequest>,
        question: &ConfigQuestion,
        result: &str,
    ) -> Result<Self, String> {
        validate_remote_name(name)?;
        if question.is_done() {
            return Err("No pending rclone question".into());
        }
        if creation.is_some_and(|creation| creation.name != name) {
            return Err("Remote name does not match the account being created".into());
        }
        Ok(Self {
            name: name.into(),
            creation: creation.cloned(),
            state: question.state.clone(),
            result: result.into(),
            secret: question
                .option
                .as_ref()
                .is_some_and(|option| option.is_password),
        })
    }

    pub fn command(&self) -> Result<AccountCommand, String> {
        let mut args = vec!["config", "update", &self.name];
        let mut secret_indices = Vec::new();
        if let Some(creation) = &self.creation {
            args.extend(creation.pairs()?);
            secret_indices.push(6);
        }
        args.extend([
            "--continue",
            "--state",
            &self.state,
            "--result",
            &self.result,
        ]);
        if self.secret {
            secret_indices.push(args.len() - 1);
        }
        args.push("--non-interactive");
        Ok(AccountCommand::new(&args, secret_indices))
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
        vec![],
    ))
}

/// Removes the remote from rclone's config only; Drive files are untouched.
pub fn delete_command(name: &str) -> Result<AccountCommand, String> {
    validate_remote_name(name)?;
    Ok(AccountCommand::new(&["config", "delete", name], vec![]))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigExample {
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub help: String,
}

#[derive(Clone, PartialEq, Eq, Deserialize)]
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
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigQuestion {
    pub state: String,
    #[serde(default)]
    pub option: Option<ConfigOption>,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub result: String,
}

impl fmt::Debug for ConfigOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let default: &str = if self.is_password {
            REDACTED
        } else {
            &self.default
        };
        f.debug_struct("ConfigOption")
            .field("name", &self.name)
            .field("default", &default)
            .field("examples", &self.examples)
            .field("required", &self.required)
            .field("is_password", &self.is_password)
            .field("exclusive", &self.exclusive)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

/// `result` may carry config values, so it is always redacted.
impl fmt::Debug for ConfigQuestion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigQuestion")
            .field("state", &self.state)
            .field("option", &self.option)
            .field("error", &self.error)
            .field("result", &REDACTED)
            .finish()
    }
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
