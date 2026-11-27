//! Rclone account protocol: literal argv builders and parsers. No process
//! execution and no GTK here; callers run commands off the GTK thread.

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

/// `rclone config reconnect` takes `remote:` and has no `--non-interactive`.
pub fn reconnect_command(name: &str) -> Result<AccountCommand, String> {
    validate_remote_name(name)?;
    Ok(AccountCommand::new(
        &["config", "reconnect", &format!("{name}:")],
        None,
    ))
}

/// Removes the remote from rclone's config only; Drive files are untouched.
pub fn delete_command(name: &str) -> Result<AccountCommand, String> {
    validate_remote_name(name)?;
    Ok(AccountCommand::new(&["config", "delete", name], None))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigExample {
    pub value: String,
    pub help: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigOption {
    pub name: String,
    pub help: String,
    /// JSON scalar rendered as rclone expects it back in `--result`.
    pub default: String,
    pub examples: Vec<ConfigExample>,
    pub required: bool,
    pub is_password: bool,
    pub exclusive: bool,
    pub kind: String,
}

/// One step of `rclone config create/update --non-interactive`. An empty
/// `state` means the process finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigQuestion {
    pub state: String,
    pub option: Option<ConfigOption>,
    pub error: String,
    pub result: String,
}

impl ConfigQuestion {
    pub fn is_done(&self) -> bool {
        self.state.is_empty()
    }

    /// Errors never include the input: rclone output may carry secrets.
    pub fn parse(text: &str) -> Result<Self, String> {
        let malformed = || "Malformed rclone configuration response".to_string();
        let mut parser = Parser {
            bytes: text.as_bytes(),
            pos: 0,
        };
        let value = parser.value().ok_or_else(malformed)?;
        parser.skip_ws();
        if parser.pos != parser.bytes.len() {
            return Err(malformed());
        }
        let Json::Obj(_) = value else {
            return Err(malformed());
        };
        let option = match value.get("Option") {
            Some(option @ Json::Obj(_)) => Some(ConfigOption {
                name: option.text("Name"),
                help: option.text("Help"),
                default: option.text("Default"),
                examples: match option.get("Examples") {
                    Some(Json::Arr(items)) => items
                        .iter()
                        .map(|item| ConfigExample {
                            value: item.text("Value"),
                            help: item.text("Help"),
                        })
                        .collect(),
                    _ => Vec::new(),
                },
                required: option.flag("Required"),
                is_password: option.flag("IsPassword"),
                exclusive: option.flag("Exclusive"),
                kind: option.text("Type"),
            }),
            _ => None,
        };
        Ok(Self {
            state: value.text("State"),
            option,
            error: value.text("Error"),
            result: value.text("Result"),
        })
    }
}

// shortcut: minimal JSON reader (no serde_json dependency allowed); numbers kept as raw text.
enum Json {
    Null,
    Bool(bool),
    Num(String),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn text(&self, key: &str) -> String {
        match self.get(key) {
            Some(Json::Str(s) | Json::Num(s)) => s.clone(),
            Some(Json::Bool(b)) => b.to_string(),
            _ => String::new(),
        }
    }

    fn flag(&self, key: &str) -> bool {
        matches!(self.get(key), Some(Json::Bool(true)))
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn skip_ws(&mut self) {
        while self
            .bytes
            .get(self.pos)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.pos += 1;
        }
    }

    fn eat(&mut self, byte: u8) -> bool {
        self.skip_ws();
        let found = self.bytes.get(self.pos) == Some(&byte);
        if found {
            self.pos += 1;
        }
        found
    }

    fn literal(&mut self, word: &str, value: Json) -> Option<Json> {
        let end = self.pos + word.len();
        (self.bytes.get(self.pos..end)? == word.as_bytes()).then(|| {
            self.pos = end;
            value
        })
    }

    fn value(&mut self) -> Option<Json> {
        self.skip_ws();
        match *self.bytes.get(self.pos)? {
            b'{' => {
                self.pos += 1;
                let mut fields = Vec::new();
                // Trailing commas are tolerated, as in rclone's documented example.
                while !self.eat(b'}') {
                    self.skip_ws();
                    let key = self.string()?;
                    if !self.eat(b':') {
                        return None;
                    }
                    fields.push((key, self.value()?));
                    if !self.eat(b',') && self.bytes.get(self.pos) != Some(&b'}') {
                        return None;
                    }
                }
                Some(Json::Obj(fields))
            }
            b'[' => {
                self.pos += 1;
                let mut items = Vec::new();
                while !self.eat(b']') {
                    items.push(self.value()?);
                    if !self.eat(b',') && self.bytes.get(self.pos) != Some(&b']') {
                        return None;
                    }
                }
                Some(Json::Arr(items))
            }
            b'"' => self.string().map(Json::Str),
            b't' => self.literal("true", Json::Bool(true)),
            b'f' => self.literal("false", Json::Bool(false)),
            b'n' => self.literal("null", Json::Null),
            b'-' | b'0'..=b'9' => {
                let start = self.pos;
                while self
                    .bytes
                    .get(self.pos)
                    .is_some_and(|b| b.is_ascii_digit() || b"+-.eE".contains(b))
                {
                    self.pos += 1;
                }
                let raw = std::str::from_utf8(&self.bytes[start..self.pos]).ok()?;
                raw.parse::<f64>().ok()?;
                Some(Json::Num(raw.into()))
            }
            _ => None,
        }
    }

    fn string(&mut self) -> Option<String> {
        if self.bytes.get(self.pos) != Some(&b'"') {
            return None;
        }
        self.pos += 1;
        let mut out = Vec::new();
        loop {
            let byte = *self.bytes.get(self.pos)?;
            self.pos += 1;
            match byte {
                b'"' => return String::from_utf8(out).ok(),
                b'\\' => {
                    let escaped = *self.bytes.get(self.pos)?;
                    self.pos += 1;
                    let c = match escaped {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let high = self.hex4()?;
                            if (0xD800..0xDC00).contains(&high) {
                                if self.bytes.get(self.pos..self.pos + 2)? != b"\\u" {
                                    return None;
                                }
                                self.pos += 2;
                                let low = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&low) {
                                    return None;
                                }
                                char::from_u32(0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00))?
                            } else {
                                char::from_u32(high)?
                            }
                        }
                        _ => return None,
                    };
                    out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
                }
                0..0x20 => return None,
                _ => out.push(byte),
            }
        }
    }

    fn hex4(&mut self) -> Option<u32> {
        let digits = std::str::from_utf8(self.bytes.get(self.pos..self.pos + 4)?).ok()?;
        self.pos += 4;
        u32::from_str_radix(digits, 16).ok()
    }
}
