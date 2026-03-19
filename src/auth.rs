// OAuth 2.0 / EVE SSO support with PKCE.

use chrono::{DateTime, Duration, Utc};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tracing::debug;

use crate::{EsiClient, EsiError, Result};

// ---------------------------------------------------------------------------
// SSO constants
// ---------------------------------------------------------------------------

const SSO_AUTH_URL: &str = "https://login.eveonline.com/v2/oauth/authorize";
const SSO_TOKEN_URL: &str = "https://login.eveonline.com/v2/oauth/token";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Credentials for EVE SSO. Debug-safe: client_secret is redacted.
#[derive(Debug, Clone)]
pub enum EsiAppCredentials {
    Web {
        client_id: String,
        client_secret: SecretString,
    },
    Native {
        client_id: String,
    },
}

impl EsiAppCredentials {
    pub fn client_id(&self) -> &str {
        match self {
            Self::Web { client_id, .. } | Self::Native { client_id } => client_id,
        }
    }
}

/// Returned by `authorize_url()`, consumed by `exchange_code()`.
pub struct PkceChallenge {
    pub authorize_url: String,
    pub code_verifier: SecretString,
    pub state: String,
}

/// Raw token response from ESI's token endpoint (private).
#[derive(Deserialize)]
struct TokenResponse {
    access_token: SecretString,
    expires_in: u64,
    #[allow(dead_code)]
    token_type: String,
    refresh_token: SecretString,
}

/// A live token pair with expiration tracking. Debug-safe.
#[derive(Debug, Clone)]
pub struct EsiTokens {
    pub access_token: SecretString,
    pub refresh_token: SecretString,
    pub expires_at: DateTime<Utc>,
}

impl EsiTokens {
    /// True if the access token has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// True if the access token expires within 60 seconds.
    pub fn needs_refresh(&self) -> bool {
        Utc::now() >= self.expires_at - Duration::seconds(60)
    }
}

// ---------------------------------------------------------------------------
// PKCE helpers
// ---------------------------------------------------------------------------

/// Generate a random 128-byte code verifier, base64url-encoded (no padding).
fn generate_code_verifier() -> SecretString {
    use rand::Rng;
    let random_bytes: Vec<u8> = (0..96).map(|_| rand::rng().random::<u8>()).collect();
    base64_url_encode(&random_bytes).into()
}

/// Compute the S256 code challenge from a verifier.
fn compute_code_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    base64_url_encode(&digest)
}

/// Generate a random state parameter.
fn generate_state() -> String {
    use rand::Rng;
    let bytes: Vec<u8> = (0..32).map(|_| rand::rng().random::<u8>()).collect();
    base64_url_encode(&bytes)
}

/// Base64url encode without padding (RFC 7636).
fn base64_url_encode(input: &[u8]) -> String {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine;
    URL_SAFE_NO_PAD.encode(input)
}

// ---------------------------------------------------------------------------
// OAuth methods on EsiClient
// ---------------------------------------------------------------------------

impl EsiClient {
    /// Build the EVE SSO authorization URL with PKCE.
    ///
    /// Returns a `PkceChallenge` containing the URL to redirect the user to,
    /// the code verifier (needed for `exchange_code`), and the state parameter
    /// (should be verified on callback).
    pub fn authorize_url(
        &self,
        redirect_uri: &str,
        scopes: &[&str],
    ) -> Result<PkceChallenge> {
        let creds = self
            .app_credentials
            .as_ref()
            .ok_or_else(|| EsiError::Auth("no app credentials configured".into()))?;

        let code_verifier = generate_code_verifier();
        let code_challenge = compute_code_challenge(code_verifier.expose_secret());
        let state = generate_state();

        let mut url = url::Url::parse(SSO_AUTH_URL)
            .map_err(|e| EsiError::Auth(format!("failed to parse SSO URL: {e}")))?;

        url.query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", redirect_uri)
            .append_pair("client_id", creds.client_id())
            .append_pair("scope", &scopes.join(" "))
            .append_pair("state", &state)
            .append_pair("code_challenge", &code_challenge)
            .append_pair("code_challenge_method", "S256");

        Ok(PkceChallenge {
            authorize_url: url.to_string(),
            code_verifier,
            state,
        })
    }

    /// Exchange an authorization code for tokens.
    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &SecretString,
        redirect_uri: &str,
    ) -> Result<EsiTokens> {
        let creds = self
            .app_credentials
            .as_ref()
            .ok_or_else(|| EsiError::Auth("no app credentials configured".into()))?;

        let form = vec![
            ("grant_type", "authorization_code".to_string()),
            ("code", code.to_string()),
            ("redirect_uri", redirect_uri.to_string()),
            ("client_id", creds.client_id().to_string()),
            (
                "code_verifier",
                code_verifier.expose_secret().to_string(),
            ),
        ];

        let request = if let EsiAppCredentials::Web { client_secret, .. } = creds {
            self.client
                .post(SSO_TOKEN_URL)
                .form(&form)
                .basic_auth(creds.client_id(), Some(client_secret.expose_secret()))
        } else {
            self.client.post(SSO_TOKEN_URL).form(&form)
        };

        let resp = request
            .send()
            .await
            .map_err(|e| EsiError::TokenRefresh(format!("token exchange request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(EsiError::TokenRefresh(format!(
                "token exchange failed (HTTP {status}): {body}"
            )));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| EsiError::TokenRefresh(format!("failed to parse token response: {e}")))?;

        let tokens = EsiTokens {
            access_token: token_resp.access_token,
            refresh_token: token_resp.refresh_token,
            expires_at: Utc::now() + Duration::seconds(token_resp.expires_in as i64),
        };

        *self.tokens.write().await = Some(tokens.clone());
        debug!("token exchange complete");
        Ok(tokens)
    }

    /// Refresh the access token using the stored refresh token.
    pub async fn refresh_token(&self) -> Result<EsiTokens> {
        let creds = self
            .app_credentials
            .as_ref()
            .ok_or_else(|| EsiError::Auth("no app credentials configured".into()))?;

        // Take write lock — prevents concurrent refresh storms.
        let mut guard = self.tokens.write().await;

        // Re-check: another task may have already refreshed while we waited.
        if let Some(ref existing) = *guard {
            if !existing.needs_refresh() {
                return Ok(existing.clone());
            }
        }

        let current_refresh = guard
            .as_ref()
            .ok_or_else(|| EsiError::TokenRefresh("no tokens to refresh".into()))?
            .refresh_token
            .clone();

        let form_params = vec![
            ("grant_type".to_string(), "refresh_token".to_string()),
            ("client_id".to_string(), creds.client_id().to_string()),
            (
                "refresh_token".to_string(),
                current_refresh.expose_secret().to_string(),
            ),
        ];

        let request = if let EsiAppCredentials::Web { client_secret, .. } = creds {
            self.client
                .post(SSO_TOKEN_URL)
                .form(&form_params)
                .basic_auth(creds.client_id(), Some(client_secret.expose_secret()))
        } else {
            self.client.post(SSO_TOKEN_URL).form(&form_params)
        };

        let resp = request
            .send()
            .await
            .map_err(|e| EsiError::TokenRefresh(format!("token refresh request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(EsiError::TokenRefresh(format!(
                "token refresh failed (HTTP {status}): {body}"
            )));
        }

        let token_resp: TokenResponse = resp
            .json()
            .await
            .map_err(|e| EsiError::TokenRefresh(format!("failed to parse token response: {e}")))?;

        let tokens = EsiTokens {
            access_token: token_resp.access_token,
            refresh_token: token_resp.refresh_token,
            expires_at: Utc::now() + Duration::seconds(token_resp.expires_in as i64),
        };

        *guard = Some(tokens.clone());
        debug!("token refresh complete");
        Ok(tokens)
    }

    /// Store tokens (e.g. restored from a database).
    pub async fn set_tokens(&self, tokens: EsiTokens) {
        *self.tokens.write().await = Some(tokens);
    }

    /// Read the current tokens.
    pub async fn get_tokens(&self) -> Option<EsiTokens> {
        self.tokens.read().await.clone()
    }

    /// Clear stored tokens (logout).
    pub async fn clear_tokens(&self) {
        *self.tokens.write().await = None;
    }

    /// Ensure we have a valid (non-expired) access token. Refreshes if needed.
    /// Returns the access token string if available.
    pub(crate) async fn ensure_valid_token(&self) -> Result<Option<SecretString>> {
        let guard = self.tokens.read().await;
        match &*guard {
            None => Ok(None),
            Some(tokens) => {
                if tokens.needs_refresh() {
                    // Drop read lock before taking write lock for refresh.
                    drop(guard);
                    let refreshed = self.refresh_token().await?;
                    Ok(Some(refreshed.access_token))
                } else {
                    Ok(Some(tokens.access_token.clone()))
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_verifier_length() {
        let verifier = generate_code_verifier();
        // 96 random bytes -> 128 base64url chars
        assert_eq!(verifier.expose_secret().len(), 128);
    }

    #[test]
    fn test_pkce_challenge_is_sha256() {
        let verifier = generate_code_verifier();
        let challenge = compute_code_challenge(verifier.expose_secret());

        // Manually compute expected challenge.
        let digest = Sha256::digest(verifier.expose_secret().as_bytes());
        let expected = base64_url_encode(&digest);
        assert_eq!(challenge, expected);
    }

    #[test]
    fn test_state_is_nonempty() {
        let state = generate_state();
        assert!(!state.is_empty());
    }

    #[test]
    fn test_authorize_url_params() {
        let client = EsiClient::with_native_app("test-agent", "my-client-id");
        let challenge = client
            .authorize_url("http://localhost:8080/callback", &["esi-wallet.read_character_wallet.v1"])
            .unwrap();

        let parsed = url::Url::parse(&challenge.authorize_url).unwrap();
        let params: std::collections::HashMap<_, _> = parsed.query_pairs().collect();

        assert_eq!(params.get("response_type").unwrap(), "code");
        assert_eq!(params.get("client_id").unwrap(), "my-client-id");
        assert_eq!(
            params.get("redirect_uri").unwrap(),
            "http://localhost:8080/callback"
        );
        assert_eq!(
            params.get("scope").unwrap(),
            "esi-wallet.read_character_wallet.v1"
        );
        assert_eq!(params.get("code_challenge_method").unwrap(), "S256");
        assert!(params.contains_key("code_challenge"));
        assert!(params.contains_key("state"));
        assert_eq!(params.get("state").unwrap(), &challenge.state);
    }

    #[test]
    fn test_authorize_url_without_credentials() {
        let client = EsiClient::new();
        let result = client.authorize_url("http://localhost", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_tokens_is_expired() {
        let expired = EsiTokens {
            access_token: SecretString::from("test".to_string()),
            refresh_token: SecretString::from("test".to_string()),
            expires_at: Utc::now() - Duration::seconds(10),
        };
        assert!(expired.is_expired());
        assert!(expired.needs_refresh());

        let valid = EsiTokens {
            access_token: SecretString::from("test".to_string()),
            refresh_token: SecretString::from("test".to_string()),
            expires_at: Utc::now() + Duration::seconds(300),
        };
        assert!(!valid.is_expired());
        assert!(!valid.needs_refresh());
    }

    #[test]
    fn test_tokens_needs_refresh_within_60s() {
        let soon = EsiTokens {
            access_token: SecretString::from("test".to_string()),
            refresh_token: SecretString::from("test".to_string()),
            expires_at: Utc::now() + Duration::seconds(30),
        };
        assert!(!soon.is_expired());
        assert!(soon.needs_refresh());
    }

    #[test]
    fn test_token_response_deserialization() {
        let json = r#"{
            "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.test",
            "expires_in": 1199,
            "token_type": "Bearer",
            "refresh_token": "abc123refresh"
        }"#;
        let resp: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token.expose_secret(), "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.test");
        assert_eq!(resp.expires_in, 1199);
        assert_eq!(resp.token_type, "Bearer");
        assert_eq!(resp.refresh_token.expose_secret(), "abc123refresh");
    }

    #[test]
    fn test_secret_redaction() {
        let tokens = EsiTokens {
            access_token: SecretString::from("super_secret_token".to_string()),
            refresh_token: SecretString::from("super_secret_refresh".to_string()),
            expires_at: Utc::now() + Duration::seconds(300),
        };
        let debug_output = format!("{:?}", tokens);
        assert!(
            !debug_output.contains("super_secret_token"),
            "access_token leaked in debug output"
        );
        assert!(
            !debug_output.contains("super_secret_refresh"),
            "refresh_token leaked in debug output"
        );
    }

    #[tokio::test]
    async fn test_new_client_has_no_tokens() {
        let client = EsiClient::new();
        assert!(client.get_tokens().await.is_none());
    }

    #[tokio::test]
    async fn test_set_and_get_tokens() {
        let client = EsiClient::new();
        let tokens = EsiTokens {
            access_token: SecretString::from("access".to_string()),
            refresh_token: SecretString::from("refresh".to_string()),
            expires_at: Utc::now() + Duration::seconds(300),
        };
        client.set_tokens(tokens).await;
        let retrieved = client.get_tokens().await.unwrap();
        assert_eq!(retrieved.access_token.expose_secret(), "access");
    }

    #[tokio::test]
    async fn test_clear_tokens() {
        let client = EsiClient::new();
        let tokens = EsiTokens {
            access_token: SecretString::from("access".to_string()),
            refresh_token: SecretString::from("refresh".to_string()),
            expires_at: Utc::now() + Duration::seconds(300),
        };
        client.set_tokens(tokens).await;
        client.clear_tokens().await;
        assert!(client.get_tokens().await.is_none());
    }
}
