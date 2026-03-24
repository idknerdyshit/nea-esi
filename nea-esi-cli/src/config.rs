use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Defaults {
    pub character_id: Option<i64>,
    pub corporation_id: Option<i64>,
    pub format: Option<String>,
    pub region_id: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub headless: bool,
}

impl Config {
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("", "", "nea-esi").map(|d| d.config_dir().to_path_buf())
    }

    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    pub fn token_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("tokens.json"))
    }

    pub fn history_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("history.txt"))
    }

    pub fn load(path: Option<&PathBuf>) -> anyhow::Result<Self> {
        let path = match path {
            Some(p) => p.clone(),
            None => match Self::config_path() {
                Some(p) => p,
                None => return Ok(Self::default()),
            },
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: Option<&PathBuf>) -> anyhow::Result<()> {
        let path = match path {
            Some(p) => p.clone(),
            None => Self::config_path()
                .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?,
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
