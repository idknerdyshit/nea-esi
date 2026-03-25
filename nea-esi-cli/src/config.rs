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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliPaths {
    pub config_path: PathBuf,
    pub token_path: PathBuf,
    pub history_path: PathBuf,
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

    pub fn cli_paths(config_override: Option<&PathBuf>) -> anyhow::Result<CliPaths> {
        if let Some(path) = config_override {
            let parent = path.parent().ok_or_else(|| {
                anyhow::anyhow!(
                    "Config path must include a parent directory: {}",
                    path.display()
                )
            })?;
            return Ok(CliPaths {
                config_path: path.clone(),
                token_path: parent.join("tokens.json"),
                history_path: parent.join("history.txt"),
            });
        }

        let config_path = Self::config_path()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
        let token_path = Self::token_path()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;
        let history_path = Self::history_path()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;

        Ok(CliPaths {
            config_path,
            token_path,
            history_path,
        })
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

        // Set restrictive permissions on Unix (config may contain client_secret).
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)?;
            let mut writer = std::io::BufWriter::new(file);
            writer.write_all(content.as_bytes())?;
        }

        #[cfg(not(unix))]
        std::fs::write(&path, content)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Config;

    #[test]
    fn cli_paths_for_override_use_sibling_files() {
        let config_path = PathBuf::from("/tmp/profile/config.toml");
        let paths = Config::cli_paths(Some(&config_path)).unwrap();

        assert_eq!(paths.config_path, config_path);
        assert_eq!(paths.token_path, PathBuf::from("/tmp/profile/tokens.json"));
        assert_eq!(
            paths.history_path,
            PathBuf::from("/tmp/profile/history.txt")
        );
    }
}
