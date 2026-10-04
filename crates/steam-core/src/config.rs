use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub last_user_id3: Option<String>,
    pub steam_path_override: Option<String>,
}

impl AppConfig {
    pub fn config_path() -> Result<PathBuf> {
        let base = dirs_next_data_dir()?;
        Ok(base.join("Shelfmark").join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.is_file() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(&path)?;
        serde_json::from_str(&raw).map_err(|e| Error::Parse(format!("config.json: {e}")))
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(self)
            .map_err(|e| Error::Internal(format!("serialize config: {e}")))?;
        std::fs::write(path, raw)?;
        Ok(())
    }
}

fn dirs_next_data_dir() -> Result<PathBuf> {
    // Keep steam-core free of `dirs` crate if possible: use env on Windows.
    #[cfg(windows)]
    {
        let appdata = std::env::var_os("APPDATA")
            .ok_or_else(|| Error::UserAction("APPDATA not set".into()))?;
        return Ok(PathBuf::from(appdata));
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join(".local/share"))
            .ok_or_else(|| Error::UserAction("HOME not set".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip_fields() {
        let c = AppConfig {
            last_user_id3: Some("123".into()),
            steam_path_override: None,
        };
        let s = serde_json::to_string(&c).unwrap();
        let back: AppConfig = serde_json::from_str(&s).unwrap();
        assert_eq!(back.last_user_id3.as_deref(), Some("123"));
    }
}
