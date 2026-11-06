use serde::Deserialize;
use std::{fmt, fs, io, path::PathBuf};

const DEFAULT_TOML: &str = include_str!("../../../config/default.toml");

// ---------- struct ketat: key salah ketik langsung error ----------
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bar: Bar,
    pub clock: Clock,
    pub status: Status,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bar {
    pub position: Position,
    pub left: Vec<Widget>,
    pub center: Vec<Widget>,
    pub right: Vec<Widget>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Top,
    Bottom,
}

// nama widget yang boleh muncul di bar; nilai lain = error otomatis
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Widget {
    Launcher,
    Tags,
    Clock,
    Status,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clock {
    pub format: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub items: Vec<StatusItem>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusItem {
    Network,
    Volume,
    Battery,
}

// ---------- error ----------
#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Syntax(toml::de::Error),  // salah sintaks TOML (ada baris/kolom)
    Invalid(toml::de::Error), // key/nilai tidak dikenal
    Rule(String),             // gagal validasi
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "gagal membaca config: {e}"),
            Self::Syntax(e) => write!(f, "sintaks TOML salah:\n{e}"),
            Self::Invalid(e) => write!(f, "isi config tidak valid:\n{e}"),
            Self::Rule(m) => write!(f, "config tidak valid: {m}"),
        }
    }
}
impl std::error::Error for ConfigError {}

// ---------- loading ----------
pub fn config_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("artemisia/config.toml")
}

pub fn load() -> Result<Config, ConfigError> {
    let mut merged: toml::Table = DEFAULT_TOML
        .parse()
        .expect("config/default.toml bawaan rusak (bug Artemisia)");

    match fs::read_to_string(config_path()) {
        Ok(text) => {
            let user: toml::Table = text.parse().map_err(ConfigError::Syntax)?;
            merge(&mut merged, user);
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {} // pakai default saja
        Err(e) => return Err(ConfigError::Io(e)),
    }

    let cfg: Config = merged.try_into().map_err(ConfigError::Invalid)?;
    cfg.validate()?;
    Ok(cfg)
}

fn merge(base: &mut toml::Table, over: toml::Table) {
    for (k, v) in over {
        match v {
            toml::Value::Table(o) => {
                if let Some(toml::Value::Table(b)) = base.get_mut(&k) {
                    merge(b, o);
                } else {
                    base.insert(k, toml::Value::Table(o));
                }
            }
            other => {
                base.insert(k, other);
            }
        }
    }
}

impl Config {
    fn validate(&self) -> Result<(), ConfigError> {
        let total = self.bar.left.len() + self.bar.center.len() + self.bar.right.len();
        if total == 0 {
            return Err(ConfigError::Rule("[bar] tidak boleh kosong semua".into()));
        }
        if self.status.items.is_empty() && self.bar.right.contains(&Widget::Status) {
            return Err(ConfigError::Rule(
                "widget \"status\" dipakai tapi [status].items kosong".into(),
            ));
        }
        Ok(())
    }
}
