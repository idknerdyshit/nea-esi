// Integration tests for Phase 1 ESI endpoints using wiremock.

use wiremock::matchers::{method, path, query_param, body_json};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nea_esi::EsiClient;

// ---------------------------------------------------------------------------
// get_type — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_type_via_request() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "type_id": 587,
        "name": "Rifter",
        "description": "A Minmatar frigate.",
        "group_id": 25,
        "market_group_id": 61,
        "published": true
    });

    Mock::given(method("GET"))
        .and(path("/universe/types/587/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new();
    let url = format!("{}/universe/types/587/", server.uri());
    let resp = client.request(&url).await.unwrap();
    let info: nea_esi::EsiTypeInfo = resp.json().await.unwrap();

    assert_eq!(info.type_id, 587);
    assert_eq!(info.name, "Rifter");
    assert_eq!(info.group_id, 25);
    assert!(info.published);
}

// ---------------------------------------------------------------------------
// list_type_ids — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_paginated_type_ids() {
    let server = MockServer::start().await;

    // Page 1 returns IDs and x-pages header
    Mock::given(method("GET"))
        .and(path("/universe/types/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([1, 2, 3]))
                .insert_header("x-pages", "2"),
        )
        .mount(&server)
        .await;

    // Page 2
    Mock::given(method("GET"))
        .and(path("/universe/types/"))
        .and(query_param("page", "2"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([4, 5])),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new();
    let url = format!("{}/universe/types/", server.uri());
    let ids = client.get_paginated::<i32>(&url).await.unwrap();

    assert_eq!(ids.len(), 5);
    assert_eq!(ids, vec![1, 2, 3, 4, 5]);
}

// ---------------------------------------------------------------------------
// resolve_ids — POST + chunking
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_resolve_ids_via_post() {
    let server = MockServer::start().await;

    let request_body = serde_json::json!(["Jita", "CCP Bartender"]);
    let response_body = serde_json::json!({
        "systems": [{"id": 30000142, "name": "Jita"}],
        "characters": [{"id": 95465499, "name": "CCP Bartender"}]
    });

    Mock::given(method("POST"))
        .and(path("/universe/ids/"))
        .and(body_json(&request_body))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&server)
        .await;

    let client = EsiClient::new();
    let url = format!("{}/universe/ids/", server.uri());
    let names = vec!["Jita".to_string(), "CCP Bartender".to_string()];
    let resp = client.request_post(&url, &names).await.unwrap();
    let resolved: nea_esi::EsiResolvedIds = resp.json().await.unwrap();

    assert_eq!(resolved.systems.len(), 1);
    assert_eq!(resolved.systems[0].name, "Jita");
    assert_eq!(resolved.characters.len(), 1);
    assert_eq!(resolved.characters[0].id, 95465499);
}

// ---------------------------------------------------------------------------
// search — URL encoding
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_search_url_encoding() {
    let server = MockServer::start().await;

    let response_body = serde_json::json!({
        "solar_system": [30000142]
    });

    Mock::given(method("GET"))
        .and(path("/search/"))
        .and(query_param("search", "Jita"))
        .and(query_param("categories", "solar_system"))
        .and(query_param("strict", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&server)
        .await;

    // Build the URL the same way the search method does
    let base = format!("{}/search/", server.uri());
    let url = url::Url::parse_with_params(
        &base,
        &[
            ("search", "Jita"),
            ("categories", "solar_system"),
            ("strict", "true"),
        ],
    )
    .unwrap();

    let client = EsiClient::new();
    let resp = client.request(url.as_str()).await.unwrap();
    let result: nea_esi::EsiSearchResult = resp.json().await.unwrap();

    assert_eq!(result.solar_system, vec![30000142]);
}

// ---------------------------------------------------------------------------
// server_status — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_server_status() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "players": 23456,
        "server_version": "2345678",
        "start_time": "2026-03-20T11:00:00Z"
    });

    Mock::given(method("GET"))
        .and(path("/status/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new();
    let url = format!("{}/status/", server.uri());
    let resp = client.request(&url).await.unwrap();
    let status: nea_esi::EsiServerStatus = resp.json().await.unwrap();

    assert_eq!(status.players, 23456);
    assert_eq!(status.server_version, Some("2345678".to_string()));
}
