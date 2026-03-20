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

    let client = EsiClient::new().with_base_url(server.uri());
    let info = client.get_type(587).await.unwrap();

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

    let client = EsiClient::new().with_base_url(server.uri());
    let ids = client.list_type_ids().await.unwrap();

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

    let client = EsiClient::new().with_base_url(server.uri());
    let names = vec!["Jita".to_string(), "CCP Bartender".to_string()];
    let resolved = client.resolve_ids(&names).await.unwrap();

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

    let client = EsiClient::new().with_base_url(server.uri());
    let result = client.search("Jita", "solar_system", true).await.unwrap();

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

    let client = EsiClient::new().with_base_url(server.uri());
    let status = client.server_status().await.unwrap();

    assert_eq!(status.players, 23456);
    assert_eq!(status.server_version, Some("2345678".to_string()));
}

// ---------------------------------------------------------------------------
// market_history — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_market_history() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "date": "2026-03-01",
        "average": 5.25,
        "highest": 5.27,
        "lowest": 5.11,
        "volume": 72016862,
        "order_count": 2267
    }]);

    Mock::given(method("GET"))
        .and(path("/markets/10000002/history/"))
        .and(query_param("type_id", "34"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let entries = client.market_history(10000002, 34).await.unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].date,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
    );
    assert_eq!(entries[0].volume, 72016862);
}

// ---------------------------------------------------------------------------
// market_prices — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_market_prices() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        {"type_id": 34, "average_price": 5.25, "adjusted_price": 5.10}
    ]);

    Mock::given(method("GET"))
        .and(path("/markets/prices/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let prices = client.market_prices().await.unwrap();

    assert_eq!(prices.len(), 1);
    assert_eq!(prices[0].type_id, 34);
}

// ---------------------------------------------------------------------------
// wallet_balance — simple GET returning bare f64
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_wallet_balance() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/characters/91234567/wallet/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(123456789.50)))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let balance = client.wallet_balance(91234567).await.unwrap();

    assert!((balance - 123456789.50).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// wallet_journal — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_wallet_journal() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "id": 123456789,
        "date": "2026-03-15T10:30:00Z",
        "ref_type": "market_transaction",
        "amount": -1500000.50
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/wallet/journal/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let entries = client.wallet_journal(91234567).await.unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, 123456789);
    assert_eq!(entries[0].ref_type, "market_transaction");
}

// ---------------------------------------------------------------------------
// wallet_transactions — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_wallet_transactions() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "transaction_id": 5678901234_i64,
        "date": "2026-03-15T10:30:00Z",
        "type_id": 34,
        "location_id": 60003760,
        "unit_price": 5.25,
        "quantity": 100000,
        "client_id": 91234567,
        "is_buy": true,
        "is_personal": true,
        "journal_ref_id": 123456789
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/wallet/transactions/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let txns = client.wallet_transactions(91234567).await.unwrap();

    assert_eq!(txns.len(), 1);
    assert_eq!(txns[0].transaction_id, 5678901234);
    assert_eq!(txns[0].type_id, 34);
    assert!(txns[0].is_buy);
}

// ---------------------------------------------------------------------------
// get_character — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_character() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "name": "Test Pilot",
        "corporation_id": 98000001
    });

    Mock::given(method("GET"))
        .and(path("/characters/91234567/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let info = client.get_character(91234567).await.unwrap();

    assert_eq!(info.name, "Test Pilot");
    assert_eq!(info.corporation_id, Some(98000001));
}
