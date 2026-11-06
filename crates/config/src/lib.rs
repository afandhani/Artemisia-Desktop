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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    Top,
    Bottom,
}

// nama widget yang boleh muncul di bar; nilai lain = error otomatis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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

        // Minimal harus ada satu widget di panel.
        if total == 0 {
            return Err(ConfigError::Rule("[bar] tidak boleh kosong semua".into()));
        }

        // Periksa apakah widget clock digunakan di salah satu slot.
        let clock_used = self.bar.left.contains(&Widget::Clock)
            || self.bar.center.contains(&Widget::Clock)
            || self.bar.right.contains(&Widget::Clock);

        if clock_used && self.clock.format.trim().is_empty() {
            return Err(ConfigError::Rule(
                "widget \"clock\" digunakan tapi [clock].format kosong".into(),
            ));
        }

        // Periksa apakah widget status digunakan di salah satu slot.
        let status_used = self.bar.left.contains(&Widget::Status)
            || self.bar.center.contains(&Widget::Status)
            || self.bar.right.contains(&Widget::Status);

        if status_used && self.status.items.is_empty() {
            return Err(ConfigError::Rule(
                "widget \"status\" digunakan tapi [status].items kosong".into(),
            ));
        }

        // Hindari status item yang sama muncul berulang.
        for (index, item) in self.status.items.iter().enumerate() {
            if self.status.items[..index].contains(item) {
                return Err(ConfigError::Rule(format!(
                    "status item \"{item:?}\" digunakan lebih dari sekali"
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> Config {
        Config {
            bar: Bar {
                position: Position::Top,
                left: vec![Widget::Launcher, Widget::Tags],
                center: vec![Widget::Clock],
                right: vec![Widget::Status],
            },
            clock: Clock {
                format: "%H:%M".into(),
            },
            status: Status {
                items: vec![StatusItem::Network, StatusItem::Volume, StatusItem::Battery],
            },
        }
    }

    #[test]
    fn accepts_valid_config() {
        let config = sample_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn rejects_empty_bar() {
        let mut config = sample_config();

        config.bar.left.clear();
        config.bar.center.clear();
        config.bar.right.clear();

        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_empty_clock_format_when_clock_is_used() {
        let mut config = sample_config();
        config.clock.format.clear();

        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_empty_status_items_when_status_is_used() {
        let mut config = sample_config();
        config.status.items.clear();

        assert!(config.validate().is_err());
    }

    #[test]
    fn checks_status_widget_in_center_slot() {
        let mut config = sample_config();

        config.bar.right.clear();
        config.bar.center = vec![Widget::Status];
        config.status.items.clear();

        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_status_items() {
        let mut config = sample_config();

        config.status.items = vec![StatusItem::Network, StatusItem::Network];

        assert!(config.validate().is_err());
    }
}
