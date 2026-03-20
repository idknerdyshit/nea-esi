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
// character_bookmarks — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_bookmarks() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "bookmark_id": 12345,
        "created": "2026-03-15T10:00:00Z",
        "location_id": 30000142,
        "creator_id": 91234567,
        "label": "Home"
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/bookmarks/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let bookmarks = client.character_bookmarks(91234567).await.unwrap();

    assert_eq!(bookmarks.len(), 1);
    assert_eq!(bookmarks[0].bookmark_id, 12345);
}

// ---------------------------------------------------------------------------
// character_calendar — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_calendar() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "event_id": 99999,
        "event_date": "2026-03-20T19:00:00Z",
        "title": "Fleet Op"
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/calendar/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let events = client.character_calendar(91234567).await.unwrap();

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].title, "Fleet Op");
}

// ---------------------------------------------------------------------------
// character_clones — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_clones() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "home_location": {"location_id": 60003760, "location_type": "station"},
        "jump_clones": [
            {"jump_clone_id": 1, "location_id": 60008494, "location_type": "station", "implants": [9899]}
        ]
    });

    Mock::given(method("GET"))
        .and(path("/characters/91234567/clones/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let clones = client.character_clones(91234567).await.unwrap();

    assert_eq!(clones.home_location.unwrap().location_id, 60003760);
    assert_eq!(clones.jump_clones.len(), 1);
}

// ---------------------------------------------------------------------------
// character_implants — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_implants() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/characters/91234567/implants/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([9899, 9941, 9942])))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let implants = client.character_implants(91234567).await.unwrap();

    assert_eq!(implants, vec![9899, 9941, 9942]);
}

// ---------------------------------------------------------------------------
// character_loyalty_points — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_loyalty_points() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        {"corporation_id": 1000035, "loyalty_points": 50000}
    ]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/loyalty/points/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let lp = client.character_loyalty_points(91234567).await.unwrap();

    assert_eq!(lp.len(), 1);
    assert_eq!(lp[0].loyalty_points, 50000);
}

// ---------------------------------------------------------------------------
// character_planets — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_planets() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "solar_system_id": 30000142,
        "planet_id": 40009082,
        "planet_type": "temperate",
        "num_pins": 5,
        "last_update": "2026-03-15T10:00:00Z",
        "upgrade_level": 4
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/planets/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let planets = client.character_planets(91234567).await.unwrap();

    assert_eq!(planets.len(), 1);
    assert_eq!(planets[0].planet_type, "temperate");
}

// ---------------------------------------------------------------------------
// character_mail — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_mail() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "mail_id": 123456,
        "timestamp": "2026-03-15T10:30:00Z",
        "from": 91234567,
        "subject": "Hello",
        "labels": [1],
        "recipients": [{"recipient_id": 92345678, "recipient_type": "character"}]
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/mail/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let mail = client.character_mail(91234567).await.unwrap();

    assert_eq!(mail.len(), 1);
    assert_eq!(mail[0].mail_id, 123456);
    assert_eq!(mail[0].subject, Some("Hello".to_string()));
}

// ---------------------------------------------------------------------------
// character_notifications — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_notifications() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "notification_id": 999888,
        "type": "StructureUnderAttack",
        "sender_id": 1000125,
        "sender_type": "corporation",
        "timestamp": "2026-03-15T10:30:00Z"
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/notifications/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let notifs = client.character_notifications(91234567).await.unwrap();

    assert_eq!(notifs.len(), 1);
    assert_eq!(notifs[0].notification_type, "StructureUnderAttack");
}

// ---------------------------------------------------------------------------
// character_contacts — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_contacts() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "contact_id": 91234567,
        "contact_type": "character",
        "standing": 10.0
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/contacts/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let contacts = client.character_contacts(91234567).await.unwrap();

    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].contact_type, "character");
}

// ---------------------------------------------------------------------------
// character_fittings — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_fittings() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "fitting_id": 12345,
        "name": "PvP Rifter",
        "description": "Standard PvP fit",
        "ship_type_id": 587,
        "items": [{"type_id": 2032, "flag": 11, "quantity": 1}]
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/fittings/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let fittings = client.character_fittings(91234567).await.unwrap();

    assert_eq!(fittings.len(), 1);
    assert_eq!(fittings[0].fitting_id, 12345);
    assert_eq!(fittings[0].name, "PvP Rifter");
}

// ---------------------------------------------------------------------------
// character_location — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_location() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "solar_system_id": 30000142,
        "station_id": 60003760
    });

    Mock::given(method("GET"))
        .and(path("/characters/91234567/location/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let loc = client.character_location(91234567).await.unwrap();

    assert_eq!(loc.solar_system_id, 30000142);
    assert_eq!(loc.station_id, Some(60003760));
}

// ---------------------------------------------------------------------------
// character_ship — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_ship() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "ship_type_id": 587,
        "ship_item_id": 1234567890_i64,
        "ship_name": "My Rifter"
    });

    Mock::given(method("GET"))
        .and(path("/characters/91234567/ship/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let ship = client.character_ship(91234567).await.unwrap();

    assert_eq!(ship.ship_type_id, 587);
    assert_eq!(ship.ship_name, "My Rifter");
}

// ---------------------------------------------------------------------------
// character_online — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_online() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "online": true,
        "last_login": "2026-03-20T10:00:00Z",
        "logins": 500
    });

    Mock::given(method("GET"))
        .and(path("/characters/91234567/online/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let status = client.character_online(91234567).await.unwrap();

    assert!(status.online);
    assert_eq!(status.logins, Some(500));
}

// ---------------------------------------------------------------------------
// character_industry_jobs — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_industry_jobs() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "job_id": 123,
        "installer_id": 91234567,
        "facility_id": 60003760,
        "activity_id": 1,
        "blueprint_id": 1234567890_i64,
        "blueprint_type_id": 687,
        "blueprint_location_id": 60003760,
        "output_location_id": 60003760,
        "runs": 10,
        "status": "active",
        "duration": 3600,
        "start_date": "2026-03-15T10:00:00Z",
        "end_date": "2026-03-15T11:00:00Z"
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/industry/jobs/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let jobs = client.character_industry_jobs(91234567).await.unwrap();

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].job_id, 123);
    assert_eq!(jobs[0].status, "active");
}

// ---------------------------------------------------------------------------
// character_contracts — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_contracts() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "contract_id": 123456,
        "issuer_id": 91234567,
        "issuer_corporation_id": 98000001,
        "type": "item_exchange",
        "status": "outstanding",
        "availability": "personal",
        "date_issued": "2026-03-15T10:00:00Z",
        "date_expired": "2026-03-29T10:00:00Z",
        "for_corporation": false
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/contracts/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let contracts = client.character_contracts(91234567).await.unwrap();

    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0].contract_type, "item_exchange");
}

// ---------------------------------------------------------------------------
// character_orders — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_orders() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "order_id": 6789012345_i64,
        "type_id": 34,
        "region_id": 10000002,
        "location_id": 60003760,
        "range": "station",
        "is_buy_order": true,
        "price": 5.13,
        "volume_total": 500000,
        "volume_remain": 250000,
        "issued": "2026-03-10T08:15:00Z",
        "min_volume": 1,
        "duration": 90
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/orders/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let orders = client.character_orders(91234567).await.unwrap();

    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].order_id, 6789012345);
    assert!(orders[0].is_buy_order);
}

// ---------------------------------------------------------------------------
// character_skills — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_skills() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "skills": [
            {"skill_id": 3300, "trained_skill_level": 5, "active_skill_level": 5, "skillpoints_in_skill": 256000}
        ],
        "total_sp": 50000000,
        "unallocated_sp": 100000
    });

    Mock::given(method("GET"))
        .and(path("/characters/91234567/skills/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let skills = client.character_skills(91234567).await.unwrap();

    assert_eq!(skills.total_sp, 50000000);
    assert_eq!(skills.skills.len(), 1);
    assert_eq!(skills.skills[0].skill_id, 3300);
}

// ---------------------------------------------------------------------------
// character_skillqueue — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_skillqueue() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "skill_id": 3300,
        "finish_level": 5,
        "queue_position": 0,
        "start_date": "2026-03-15T10:00:00Z",
        "finish_date": "2026-03-20T10:00:00Z"
    }]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/skillqueue/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let queue = client.character_skillqueue(91234567).await.unwrap();

    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].skill_id, 3300);
    assert_eq!(queue[0].finish_level, 5);
}

// ---------------------------------------------------------------------------
// character_attributes — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_attributes() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "intelligence": 20,
        "memory": 20,
        "perception": 20,
        "willpower": 20,
        "charisma": 19
    });

    Mock::given(method("GET"))
        .and(path("/characters/91234567/attributes/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let attrs = client.character_attributes(91234567).await.unwrap();

    assert_eq!(attrs.intelligence, 20);
    assert_eq!(attrs.charisma, 19);
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
