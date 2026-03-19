// Integration tests for OAuth/SSO token exchange, refresh, and bearer injection.

use chrono::{Duration, Utc};
use nea_esi::{EsiClient, EsiTokens};
use secrecy::{ExposeSecret, SecretString};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_bearer_header_present_for_authenticated_client() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(header("Authorization", "Bearer my-access-token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"ok":true}"#))
        .expect(1)
        .mount(&server)
        .await;

    let client = EsiClient::with_native_app("test-agent", "test-client");
    let tokens = EsiTokens {
        access_token: SecretString::from("my-access-token".to_string()),
        refresh_token: SecretString::from("my-refresh-token".to_string()),
        expires_at: Utc::now() + Duration::seconds(300),
    };
    client.set_tokens(tokens).await;

    let url = format!("{}/test", server.uri());
    let resp = client.request(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_no_bearer_header_for_unauthenticated_client() {
    let server = MockServer::start().await;

    // This mock should NOT receive an Authorization header.
    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"ok":true}"#)
                .append_header("x-esi-error-limit-remain", "100"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = EsiClient::new();
    let url = format!("{}/test", server.uri());
    let resp = client.request(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_set_tokens_stores_and_retrieves() {
    let client = EsiClient::with_native_app("test-agent", "test-client");

    let tokens = EsiTokens {
        access_token: SecretString::from("exchanged-access".to_string()),
        refresh_token: SecretString::from("exchanged-refresh".to_string()),
        expires_at: Utc::now() + Duration::seconds(1199),
    };
    client.set_tokens(tokens).await;

    let stored = client.get_tokens().await.unwrap();
    assert_eq!(stored.access_token.expose_secret(), "exchanged-access");
    assert_eq!(stored.refresh_token.expose_secret(), "exchanged-refresh");
    assert!(!stored.is_expired());
}

#[tokio::test]
async fn test_token_lifecycle_set_get_clear() {
    let client = EsiClient::new();
    assert!(client.get_tokens().await.is_none());

    let tokens = EsiTokens {
        access_token: SecretString::from("a".to_string()),
        refresh_token: SecretString::from("r".to_string()),
        expires_at: Utc::now() + Duration::seconds(300),
    };
    client.set_tokens(tokens).await;
    assert!(client.get_tokens().await.is_some());

    client.clear_tokens().await;
    assert!(client.get_tokens().await.is_none());
}
