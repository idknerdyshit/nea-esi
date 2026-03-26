use chrono::{DateTime, Utc};
use nea_esi::EsiTokens;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

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

pub fn save_tokens_at(tokens: &EsiTokens, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let stored = StoredTokens::from(tokens);
    let json = serde_json::to_string_pretty(&stored)?;

    // Set restrictive permissions on Unix before writing
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(json.as_bytes())?;
    }

    #[cfg(not(unix))]
    std::fs::write(&path, json)?;

    Ok(())
}

pub fn load_tokens_at(path: impl AsRef<std::path::Path>) -> anyhow::Result<Option<EsiTokens>> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)?;
    let stored: StoredTokens = serde_json::from_str(&content)?;
    Ok(Some(EsiTokens::from(stored)))
}

pub fn delete_tokens_at(path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use nea_esi::EsiTokens;
    use secrecy::{ExposeSecret, SecretString};
    use tempfile::tempdir;

    use super::{delete_tokens_at, load_tokens_at, save_tokens_at};

    #[test]
    fn token_round_trip_uses_explicit_path() {
        let dir = tempdir().unwrap();
        let token_path = dir.path().join("tokens.json");
        let tokens = EsiTokens {
            access_token: SecretString::from("access".to_string()),
            refresh_token: SecretString::from("refresh".to_string()),
            expires_at: Utc::now(),
        };

        save_tokens_at(&tokens, &token_path).unwrap();
        let loaded = load_tokens_at(&token_path).unwrap().unwrap();

        assert_eq!(loaded.access_token.expose_secret(), "access");
        assert_eq!(loaded.refresh_token.expose_secret(), "refresh");
    }

    #[test]
    fn delete_tokens_removes_explicit_path() {
        let dir = tempdir().unwrap();
        let token_path = dir.path().join("tokens.json");
        std::fs::write(&token_path, "{}").unwrap();

        delete_tokens_at(&token_path).unwrap();

        assert!(!token_path.exists());
    }
}
