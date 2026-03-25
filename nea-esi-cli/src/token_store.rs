use chrono::{DateTime, Utc};
use nea_esi::EsiTokens;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Serialize, Deserialize)]
struct StoredTokens {
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
}

impl From<&EsiTokens> for StoredTokens {
    fn from(tokens: &EsiTokens) -> Self {
        Self {
            access_token: tokens.access_token.expose_secret().to_string(),
            refresh_token: tokens.refresh_token.expose_secret().to_string(),
            expires_at: tokens.expires_at,
        }
    }
}

impl From<StoredTokens> for EsiTokens {
    fn from(stored: StoredTokens) -> Self {
        Self {
            access_token: SecretString::from(stored.access_token),
            refresh_token: SecretString::from(stored.refresh_token),
            expires_at: stored.expires_at,
        }
    }
}

pub fn save_tokens(tokens: &EsiTokens) -> anyhow::Result<()> {
    let path = Config::token_path()
        .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let stored = StoredTokens::from(tokens);
    let json = serde_json::to_string_pretty(&stored)?;

    // Set restrictive permissions on Unix before writing
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&path)?;
        use std::io::Write;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(json.as_bytes())?;
    }

    #[cfg(not(unix))]
    std::fs::write(&path, json)?;

    Ok(())
}

pub fn load_tokens() -> anyhow::Result<Option<EsiTokens>> {
    let path = match Config::token_path() {
        Some(p) => p,
        None => return Ok(None),
    };

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)?;
    let stored: StoredTokens = serde_json::from_str(&content)?;
    Ok(Some(EsiTokens::from(stored)))
}

pub fn delete_tokens() -> anyhow::Result<()> {
    if let Some(path) = Config::token_path() {
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}
