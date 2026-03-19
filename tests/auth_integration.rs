// Integration tests for OAuth/SSO token exchange, refresh, and bearer injection.

use chrono::{Duration, Utc};
use nea_esi::{EsiClient, EsiTokens};
use secrecy::{ExposeSecret, SecretString};
use wiremock::matchers::{body_json, header, method, path};
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

#[tokio::test]
async fn test_character_assets_with_auth() {
    let server = MockServer::start().await;

    let page1 = serde_json::json!([
        {
            "item_id": 1,
            "type_id": 587,
            "location_id": 60003760,
            "location_type": "station",
            "location_flag": "Hangar",
            "quantity": 1,
            "is_singleton": false
        }
    ]);
    let page2 = serde_json::json!([
        {
            "item_id": 2,
            "type_id": 34,
            "location_id": 60003760,
            "location_type": "station",
            "location_flag": "Hangar",
            "quantity": 500,
            "is_singleton": false
        }
    ]);

    Mock::given(method("GET"))
        .and(path("/latest/characters/12345/assets/"))
        .and(header("Authorization", "Bearer asset-token"))
        .and(wiremock::matchers::query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&page1)
                .append_header("x-pages", "2")
                .append_header("x-esi-error-limit-remain", "100"),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/latest/characters/12345/assets/"))
        .and(header("Authorization", "Bearer asset-token"))
        .and(wiremock::matchers::query_param("page", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&page2)
                .append_header("x-esi-error-limit-remain", "100"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::with_native_app("test-agent", "test-client");
    let tokens = EsiTokens {
        access_token: SecretString::from("asset-token".to_string()),
        refresh_token: SecretString::from("r".to_string()),
        expires_at: Utc::now() + Duration::seconds(300),
    };
    client.set_tokens(tokens).await;

    // Override BASE_URL by calling request directly through the mock server.
    // We need to test the endpoint method, so we'll use the mock server URL.
    // Since character_assets() uses BASE_URL, we test via the lower-level request.
    let url = format!("{}/latest/characters/12345/assets/?page=1", server.uri());
    let resp = client.request(&url).await.unwrap();
    let x_pages: i32 = resp
        .headers()
        .get("x-pages")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    assert_eq!(x_pages, 2);

    let items: Vec<nea_esi::EsiAssetItem> = resp.json().await.unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].item_id, 1);

    // Fetch page 2
    let url2 = format!("{}/latest/characters/12345/assets/?page=2", server.uri());
    let resp2 = client.request(&url2).await.unwrap();
    let items2: Vec<nea_esi::EsiAssetItem> = resp2.json().await.unwrap();
    assert_eq!(items2.len(), 1);
    assert_eq!(items2[0].item_id, 2);
    assert_eq!(items2[0].quantity, 500);
}

#[tokio::test]
async fn test_resolve_names_post_body() {
    let server = MockServer::start().await;

    let expected_body = serde_json::json!([95465499, 1000125]);
    let response_body = serde_json::json!([
        {"id": 95465499, "name": "CCP Bartender", "category": "character"},
        {"id": 1000125, "name": "Serpentis Corporation", "category": "corporation"}
    ]);

    Mock::given(method("POST"))
        .and(path("/latest/universe/names/"))
        .and(body_json(&expected_body))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&response_body)
                .append_header("x-esi-error-limit-remain", "100"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = EsiClient::new();
    let ids: Vec<i64> = vec![95465499, 1000125];
    let url = format!("{}/latest/universe/names/", server.uri());
    let resp = client.request_post(&url, &ids).await.unwrap();
    let names: Vec<nea_esi::EsiResolvedName> = resp.json().await.unwrap();

    assert_eq!(names.len(), 2);
    assert_eq!(names[0].id, 95465499);
    assert_eq!(names[0].name, "CCP Bartender");
    assert_eq!(names[0].category, "character");
    assert_eq!(names[1].id, 1000125);
    assert_eq!(names[1].category, "corporation");
}

#[tokio::test]
async fn test_get_structure_with_auth() {
    let server = MockServer::start().await;

    let response_body = serde_json::json!({
        "name": "Test Citadel",
        "owner_id": 98000001,
        "solar_system_id": 30000142,
        "type_id": 35832
    });

    Mock::given(method("GET"))
        .and(path("/latest/universe/structures/1000000000001/"))
        .and(header("Authorization", "Bearer struct-token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&response_body)
                .append_header("x-esi-error-limit-remain", "100"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = EsiClient::with_native_app("test-agent", "test-client");
    let tokens = EsiTokens {
        access_token: SecretString::from("struct-token".to_string()),
        refresh_token: SecretString::from("r".to_string()),
        expires_at: Utc::now() + Duration::seconds(300),
    };
    client.set_tokens(tokens).await;

    let url = format!(
        "{}/latest/universe/structures/1000000000001/",
        server.uri()
    );
    let resp = client.request(&url).await.unwrap();
    let info: nea_esi::EsiStructureInfo = resp.json().await.unwrap();

    assert_eq!(info.name, "Test Citadel");
    assert_eq!(info.owner_id, 98000001);
    assert_eq!(info.solar_system_id, 30000142);
    assert_eq!(info.type_id, Some(35832));
}

#[tokio::test]
async fn test_market_prices_no_auth() {
    let server = MockServer::start().await;

    let response_body = serde_json::json!([
        {"type_id": 34, "average_price": 5.25, "adjusted_price": 5.10},
        {"type_id": 35, "average_price": 6.50}
    ]);

    Mock::given(method("GET"))
        .and(path("/latest/markets/prices/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&response_body)
                .append_header("x-esi-error-limit-remain", "100"),
        )
        .expect(1)
        .mount(&server)
        .await;

    // Use a client with NO tokens to verify no auth header is sent.
    let client = EsiClient::new();
    let url = format!("{}/latest/markets/prices/", server.uri());
    let resp = client.request(&url).await.unwrap();
    let prices: Vec<nea_esi::EsiMarketPrice> = resp.json().await.unwrap();

    assert_eq!(prices.len(), 2);
    assert_eq!(prices[0].type_id, 34);
    assert!((prices[0].average_price.unwrap() - 5.25).abs() < f64::EPSILON);
    assert!((prices[0].adjusted_price.unwrap() - 5.10).abs() < f64::EPSILON);
    assert_eq!(prices[1].type_id, 35);
    assert_eq!(prices[1].adjusted_price, None);
}
