// Integration tests for Phase 1 ESI endpoints using wiremock.

use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use nea_esi::{EsiClient, EsiFittingItem, EsiMailRecipient, EsiNewFitting, EsiNewMail};

// ---------------------------------------------------------------------------
// create_fitting — POST
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_fitting() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/characters/91234567/fittings/"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"fitting_id": 99999})),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let fitting = EsiNewFitting {
        name: "Test Fit".to_string(),
        description: "A test".to_string(),
        ship_type_id: 587,
        items: vec![EsiFittingItem {
            type_id: 2032,
            flag: 11,
            quantity: 1,
        }],
    };
    let fitting_id = client.create_fitting(91234567, &fitting).await.unwrap();
    assert_eq!(fitting_id, 99999);
}

// ---------------------------------------------------------------------------
// delete_fitting — DELETE
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_delete_fitting() {
    let server = MockServer::start().await;

    Mock::given(method("DELETE"))
        .and(path("/characters/91234567/fittings/99999/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    client.delete_fitting(91234567, 99999).await.unwrap();
}

// ---------------------------------------------------------------------------
// send_mail — POST
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_send_mail() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/characters/91234567/mail/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(12345)))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let mail = EsiNewMail {
        recipients: vec![EsiMailRecipient {
            recipient_id: 92345678,
            recipient_type: "character".to_string(),
        }],
        subject: "Test".to_string(),
        body: "Hello".to_string(),
        approved_cost: None,
    };
    let mail_id = client.send_mail(91234567, &mail).await.unwrap();
    assert_eq!(mail_id, 12345);
}

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
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([4, 5])))
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
    let events = client.character_calendar(91234567, None).await.unwrap();

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
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([9899, 9941, 9942])),
        )
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
    let mail = client.character_mail(91234567, None).await.unwrap();

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
    let jobs = client
        .character_industry_jobs(91234567, false)
        .await
        .unwrap();

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
    let txns = client.wallet_transactions(91234567, None).await.unwrap();

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

// ===========================================================================
// Corporation endpoint tests (Phase 3)
// ===========================================================================

// ---------------------------------------------------------------------------
// corp_wallet_balances — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_wallet_balances() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        {"division": 1, "balance": 123456789.50},
        {"division": 2, "balance": 500.00}
    ]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/wallets/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let divs = client.corp_wallet_balances(98000001).await.unwrap();

    assert_eq!(divs.len(), 2);
    assert_eq!(divs[0].division, 1);
    assert!((divs[0].balance - 123456789.50).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// corp_wallet_journal — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_wallet_journal() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "id": 987654321,
        "date": "2026-03-15T10:30:00Z",
        "ref_type": "corporation_account_withdrawal",
        "amount": -5000000.0
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/wallets/1/journal/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let entries = client.corp_wallet_journal(98000001, 1).await.unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].id, 987654321);
}

// ---------------------------------------------------------------------------
// corp_wallet_transactions — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_wallet_transactions() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "transaction_id": 1234567890_i64,
        "date": "2026-03-15T10:30:00Z",
        "type_id": 34,
        "location_id": 60003760,
        "unit_price": 5.25,
        "quantity": 100000,
        "client_id": 91234567,
        "is_buy": true,
        "is_personal": false,
        "journal_ref_id": 987654321
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/wallets/1/transactions/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let txns = client
        .corp_wallet_transactions(98000001, 1, None)
        .await
        .unwrap();

    assert_eq!(txns.len(), 1);
    assert_eq!(txns[0].transaction_id, 1234567890);
}

// ---------------------------------------------------------------------------
// corp_assets — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_assets() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "item_id": 1234567890_i64,
        "type_id": 587,
        "location_id": 60003760,
        "location_type": "station",
        "location_flag": "Hangar",
        "quantity": 1,
        "is_singleton": true
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/assets/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let assets = client.corp_assets(98000001).await.unwrap();

    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].type_id, 587);
}

// ---------------------------------------------------------------------------
// corp_asset_names — POST
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_asset_names() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/corporations/98000001/assets/names/"))
        .and(body_json(serde_json::json!([1234567890_i64])))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"item_id": 1234567890_i64, "name": "My Ship"}
        ])))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let names = client
        .corp_asset_names(98000001, &[1234567890])
        .await
        .unwrap();

    assert_eq!(names.len(), 1);
    assert_eq!(names[0].name, "My Ship");
}

// ---------------------------------------------------------------------------
// corp_asset_locations — POST
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_asset_locations() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/corporations/98000001/assets/locations/"))
        .and(body_json(serde_json::json!([1234567890_i64])))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"item_id": 1234567890_i64, "position": {"x": 1.0, "y": 2.0, "z": 3.0}}
        ])))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let locs = client
        .corp_asset_locations(98000001, &[1234567890])
        .await
        .unwrap();

    assert_eq!(locs.len(), 1);
    assert!((locs[0].position.x - 1.0).abs() < f64::EPSILON);
}

// ---------------------------------------------------------------------------
// corp_industry_jobs — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_industry_jobs() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "job_id": 456,
        "installer_id": 91234567,
        "facility_id": 60003760,
        "activity_id": 1,
        "blueprint_id": 1234567890_i64,
        "blueprint_type_id": 687,
        "blueprint_location_id": 60003760,
        "output_location_id": 60003760,
        "runs": 5,
        "status": "active",
        "duration": 7200,
        "start_date": "2026-03-15T10:00:00Z",
        "end_date": "2026-03-15T12:00:00Z"
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/industry/jobs/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let jobs = client.corp_industry_jobs(98000001, false).await.unwrap();

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].job_id, 456);
}

// ---------------------------------------------------------------------------
// corp_blueprints — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_blueprints() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "item_id": 1234567890_i64,
        "type_id": 687,
        "location_id": 60003760,
        "location_flag": "CorpSAG1",
        "quantity": -1,
        "time_efficiency": 20,
        "material_efficiency": 10,
        "runs": -1
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/blueprints/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let bps = client.corp_blueprints(98000001).await.unwrap();

    assert_eq!(bps.len(), 1);
    assert_eq!(bps[0].type_id, 687);
}

// ---------------------------------------------------------------------------
// corp_contracts — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_contracts() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "contract_id": 789012,
        "issuer_id": 91234567,
        "issuer_corporation_id": 98000001,
        "type": "item_exchange",
        "status": "outstanding",
        "availability": "corporation",
        "date_issued": "2026-03-15T10:00:00Z",
        "date_expired": "2026-03-29T10:00:00Z",
        "for_corporation": true
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/contracts/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let contracts = client.corp_contracts(98000001).await.unwrap();

    assert_eq!(contracts.len(), 1);
    assert!(contracts[0].for_corporation);
}

// ---------------------------------------------------------------------------
// corp_orders — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_orders() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "order_id": 6789012345_i64,
        "type_id": 34,
        "region_id": 10000002,
        "location_id": 60003760,
        "range": "station",
        "is_buy_order": false,
        "price": 6.00,
        "volume_total": 1000000,
        "volume_remain": 500000,
        "issued": "2026-03-10T08:15:00Z",
        "min_volume": 1,
        "duration": 90
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/orders/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let orders = client.corp_orders(98000001).await.unwrap();

    assert_eq!(orders.len(), 1);
    assert!(!orders[0].is_buy_order);
}

// ---------------------------------------------------------------------------
// corp_order_history — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_order_history() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "order_id": 6789012345_i64,
        "type_id": 34,
        "region_id": 10000002,
        "location_id": 60003760,
        "range": "station",
        "is_buy_order": true,
        "price": 5.00,
        "volume_total": 500000,
        "volume_remain": 0,
        "issued": "2026-03-01T08:15:00Z",
        "min_volume": 1,
        "duration": 90,
        "state": "expired"
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/orders/history/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let orders = client.corp_order_history(98000001).await.unwrap();

    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].state, Some("expired".to_string()));
}

// ---------------------------------------------------------------------------
// corp_members — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_members() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/members/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([91234567_i64, 92345678_i64])),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let members = client.corp_members(98000001).await.unwrap();

    assert_eq!(members, vec![91234567_i64, 92345678_i64]);
}

// ---------------------------------------------------------------------------
// corp_member_titles — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_member_titles() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "character_id": 91234567,
        "titles": [1, 16]
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/members/titles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let titles = client.corp_member_titles(98000001).await.unwrap();

    assert_eq!(titles.len(), 1);
    assert_eq!(titles[0].titles, vec![1, 16]);
}

// ---------------------------------------------------------------------------
// corp_member_roles — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_member_roles() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "character_id": 91234567,
        "roles": ["Director", "Hangar_Access"],
        "roles_at_hq": ["Hangar_Access"],
        "roles_at_base": [],
        "roles_at_other": []
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/roles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let roles = client.corp_member_roles(98000001).await.unwrap();

    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].roles, vec!["Director", "Hangar_Access"]);
}

// ---------------------------------------------------------------------------
// corp_member_tracking — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_member_tracking() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "character_id": 91234567,
        "location_id": 60003760,
        "logon_date": "2026-03-20T10:00:00Z",
        "logoff_date": "2026-03-20T08:00:00Z",
        "ship_type_id": 587,
        "start_date": "2020-01-15T00:00:00Z"
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/membertracking/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let tracking = client.corp_member_tracking(98000001).await.unwrap();

    assert_eq!(tracking.len(), 1);
    assert_eq!(tracking[0].ship_type_id, Some(587));
}

// ---------------------------------------------------------------------------
// corp_structures — paginated GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_structures() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "structure_id": 1234567890123_i64,
        "corporation_id": 98000001,
        "system_id": 30000142,
        "type_id": 35832,
        "state": "shield_vulnerable",
        "name": "Home Citadel",
        "services": [{"name": "Manufacturing", "state": "online"}]
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/structures/"))
        .and(query_param("page", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(&body)
                .insert_header("x-pages", "1"),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let structures = client.corp_structures(98000001).await.unwrap();

    assert_eq!(structures.len(), 1);
    assert_eq!(structures[0].state, "shield_vulnerable");
    assert_eq!(structures[0].services.len(), 1);
    assert_eq!(structures[0].services[0].name, "Manufacturing");
}

// ---------------------------------------------------------------------------
// corp_starbases — simple GET
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_starbases() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "starbase_id": 12345678_i64,
        "system_id": 30000142,
        "type_id": 16213,
        "state": "online",
        "moon_id": 40009082
    }]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/starbases/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let starbases = client.corp_starbases(98000001).await.unwrap();

    assert_eq!(starbases.len(), 1);
    assert_eq!(starbases[0].state, "online");
    assert_eq!(starbases[0].moon_id, Some(40009082));
}

// ---------------------------------------------------------------------------
// corp_starbase_detail — simple GET with query param
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_starbase_detail() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "state": "online",
        "allow_alliance_members": true,
        "allow_corporation_members": true,
        "use_alliance_standings": false,
        "attack_if_at_war": true,
        "attack_if_other_security_status_dropping": false,
        "fuels": [{"type_id": 4051, "quantity": 960}]
    });

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/starbases/12345678/"))
        .and(query_param("system_id", "30000142"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let detail = client
        .corp_starbase_detail(98000001, 12345678, 30000142)
        .await
        .unwrap();

    assert_eq!(detail.state, "online");
    assert!(detail.allow_alliance_members);
    assert!(detail.attack_if_at_war);
    assert_eq!(detail.fuels.len(), 1);
    assert_eq!(detail.fuels[0].type_id, 4051);
}

// ===========================================================================
// Phase 4 — Supplementary & Niche Endpoints
// ===========================================================================

// ---------------------------------------------------------------------------
// get_dogma_attribute
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_dogma_attribute() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "attribute_id": 2,
        "name": "isOnline",
        "published": true,
        "default_value": 0.0,
        "stackable": true,
        "high_is_good": true
    });

    Mock::given(method("GET"))
        .and(path("/dogma/attributes/2/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let attr = client.get_dogma_attribute(2).await.unwrap();

    assert_eq!(attr.attribute_id, 2);
    assert_eq!(attr.name, "isOnline");
    assert!(attr.published);
    assert!(attr.stackable);
}

// ---------------------------------------------------------------------------
// get_dogma_effect
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_dogma_effect() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "effect_id": 16,
        "name": "alwaysOn",
        "published": false,
        "effect_category": 0,
        "modifiers": [
            {
                "domain": "shipID",
                "func": "ItemModifier",
                "modified_attribute_id": 9,
                "modifying_attribute_id": 10,
                "operator": 6
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/dogma/effects/16/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let effect = client.get_dogma_effect(16).await.unwrap();

    assert_eq!(effect.effect_id, 16);
    assert_eq!(effect.name, "alwaysOn");
    assert!(!effect.published);
    assert_eq!(effect.modifiers.len(), 1);
    assert_eq!(effect.modifiers[0].operator, Some(6));
}

// ---------------------------------------------------------------------------
// get_dynamic_item
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_dynamic_item() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "created_by": 91234567,
        "mutator_type_id": 47845,
        "source_type_id": 2281,
        "dogma_attributes": [
            {"attribute_id": 9, "value": 1.5}
        ],
        "dogma_effects": [
            {"effect_id": 16, "is_default": true}
        ]
    });

    Mock::given(method("GET"))
        .and(path("/dogma/dynamic/items/2281/1234567890/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let item = client.get_dynamic_item(2281, 1234567890).await.unwrap();

    assert_eq!(item.mutator_type_id, 47845);
    assert_eq!(item.dogma_attributes.len(), 1);
    assert!((item.dogma_attributes[0].value - 1.5).abs() < f64::EPSILON);
    assert!(item.dogma_effects[0].is_default);
}

// ---------------------------------------------------------------------------
// opportunity_group_ids
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_opportunity_group_ids() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/opportunities/groups/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([1, 2, 3])))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let ids = client.opportunity_group_ids().await.unwrap();
    assert_eq!(ids, vec![1, 2, 3]);
}

// ---------------------------------------------------------------------------
// opportunity_task_ids
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_opportunity_task_ids() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/opportunities/tasks/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([10, 20])))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let ids = client.opportunity_task_ids().await.unwrap();
    assert_eq!(ids, vec![10, 20]);
}

// ---------------------------------------------------------------------------
// character_opportunities
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_opportunities() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        {"opportunity_id": 1, "completed_at": "2026-01-01T12:00:00Z"}
    ]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/opportunities/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let opps = client.character_opportunities(91234567).await.unwrap();
    assert_eq!(opps.len(), 1);
    assert_eq!(opps[0].opportunity_id, 1);
}

// ---------------------------------------------------------------------------
// character_fleet
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_fleet() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "fleet_id": 1234567890123_i64,
        "role": "squad_member",
        "squad_id": 111,
        "wing_id": 222
    });

    Mock::given(method("GET"))
        .and(path("/characters/91234567/fleet/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let fleet = client.character_fleet(91234567).await.unwrap();
    assert_eq!(fleet.fleet_id, 1234567890123);
    assert_eq!(fleet.role, "squad_member");
}

// ---------------------------------------------------------------------------
// get_fleet
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_fleet() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "fleet_id": 1234567890123_i64,
        "is_free_move": true,
        "is_registered": false,
        "is_voice_enabled": false,
        "motd": "Welcome"
    });

    Mock::given(method("GET"))
        .and(path("/fleets/1234567890123/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let info = client.get_fleet(1234567890123).await.unwrap();
    assert!(info.is_free_move);
    assert_eq!(info.motd, Some("Welcome".to_string()));
}

// ---------------------------------------------------------------------------
// fleet_members
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fleet_members() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "character_id": 91234567,
        "join_time": "2026-01-01T00:00:00Z",
        "role": "squad_commander",
        "role_name": "Squad Commander",
        "ship_type_id": 587,
        "solar_system_id": 30000142,
        "squad_id": 111,
        "takes_fleet_warp": true,
        "wing_id": 222
    }]);

    Mock::given(method("GET"))
        .and(path("/fleets/1234567890123/members/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let members = client.fleet_members(1234567890123).await.unwrap();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].ship_type_id, 587);
    assert!(members[0].takes_fleet_warp);
}

// ---------------------------------------------------------------------------
// fleet_wings
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fleet_wings() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "id": 222,
        "name": "Wing 1",
        "squads": [{"id": 111, "name": "Squad 1"}]
    }]);

    Mock::given(method("GET"))
        .and(path("/fleets/1234567890123/wings/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let wings = client.fleet_wings(1234567890123).await.unwrap();
    assert_eq!(wings.len(), 1);
    assert_eq!(wings[0].name, "Wing 1");
    assert_eq!(wings[0].squads.len(), 1);
    assert_eq!(wings[0].squads[0].name, "Squad 1");
}

// ---------------------------------------------------------------------------
// get_war
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_war() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "id": 1941,
        "declared": "2026-01-01T00:00:00Z",
        "mutual": false,
        "open_for_allies": true,
        "aggressor": {
            "corporation_id": 98000001,
            "isk_destroyed": 1000000.0,
            "ships_killed": 5
        },
        "defender": {
            "corporation_id": 98000002,
            "isk_destroyed": 500000.0,
            "ships_killed": 2
        },
        "started": "2026-01-02T00:00:00Z",
        "allies": [{"corporation_id": 98000003}]
    });

    Mock::given(method("GET"))
        .and(path("/wars/1941/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let war = client.get_war(1941).await.unwrap();
    assert_eq!(war.id, 1941);
    assert!(!war.mutual);
    assert!(war.open_for_allies);
    assert_eq!(war.aggressor.ships_killed, 5);
    assert_eq!(war.allies.len(), 1);
    assert!(war.started.is_some());
}

// ---------------------------------------------------------------------------
// fw_stats
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fw_stats() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "faction_id": 500001,
        "pilots": 1234,
        "systems_controlled": 20,
        "kills": {"last_week": 100, "total": 5000, "yesterday": 15},
        "victory_points": {"last_week": 200, "total": 10000, "yesterday": 30}
    }]);

    Mock::given(method("GET"))
        .and(path("/fw/stats/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let stats = client.fw_stats().await.unwrap();
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].faction_id, 500001);
    assert_eq!(stats[0].kills.total, 5000);
    assert_eq!(stats[0].victory_points.yesterday, 30);
}

// ---------------------------------------------------------------------------
// fw_systems
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fw_systems() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "solar_system_id": 30002057,
        "contested": "contested",
        "occupier_faction_id": 500001,
        "owner_faction_id": 500002,
        "victory_points": 100,
        "victory_points_threshold": 3000
    }]);

    Mock::given(method("GET"))
        .and(path("/fw/systems/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let systems = client.fw_systems().await.unwrap();
    assert_eq!(systems.len(), 1);
    assert_eq!(systems[0].contested, "contested");
    assert_eq!(systems[0].victory_points_threshold, 3000);
}

// ---------------------------------------------------------------------------
// fw_leaderboards
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fw_leaderboards() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "kills": {
            "active_total": [{"amount": 100, "id": 500001}],
            "last_week": [],
            "yesterday": []
        },
        "victory_points": {
            "active_total": [],
            "last_week": [{"amount": 200, "id": 500002}],
            "yesterday": []
        }
    });

    Mock::given(method("GET"))
        .and(path("/fw/leaderboards/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let lb = client.fw_leaderboards().await.unwrap();
    assert_eq!(lb.kills.active_total.len(), 1);
    assert_eq!(lb.kills.active_total[0].amount, 100);
    assert_eq!(lb.victory_points.last_week.len(), 1);
}

// ---------------------------------------------------------------------------
// fw_wars
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_fw_wars() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        {"against_id": 500002, "faction_id": 500001},
        {"against_id": 500001, "faction_id": 500002}
    ]);

    Mock::given(method("GET"))
        .and(path("/fw/wars/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let wars = client.fw_wars().await.unwrap();
    assert_eq!(wars.len(), 2);
    assert_eq!(wars[0].faction_id, 500001);
}

// ---------------------------------------------------------------------------
// insurance_prices
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_insurance_prices() {
    let server = MockServer::start().await;

    let body = serde_json::json!([{
        "type_id": 587,
        "levels": [
            {"cost": 10.0, "name": "Basic", "payout": 40.0},
            {"cost": 20.0, "name": "Standard", "payout": 100.0}
        ]
    }]);

    Mock::given(method("GET"))
        .and(path("/insurance/prices/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let prices = client.insurance_prices().await.unwrap();
    assert_eq!(prices.len(), 1);
    assert_eq!(prices[0].type_id, 587);
    assert_eq!(prices[0].levels.len(), 2);
    assert_eq!(prices[0].levels[0].name, "Basic");
}

// ---------------------------------------------------------------------------
// get_route — basic
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_route_basic() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/route/30000142/30002187/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([30000142, 30000144, 30002187])),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let route = client
        .get_route(30000142, 30002187, None, &[], None)
        .await
        .unwrap();
    assert_eq!(route, vec![30000142, 30000144, 30002187]);
}

// ---------------------------------------------------------------------------
// get_route — with flag and avoid
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_route_with_options() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/route/30000142/30002187/"))
        .and(query_param("flag", "secure"))
        .and(query_param("avoid", "30000144"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!([30000142, 30000145, 30002187])),
        )
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let route = client
        .get_route(30000142, 30002187, Some("secure"), &[30000144], None)
        .await
        .unwrap();
    assert_eq!(route, vec![30000142, 30000145, 30002187]);
}

// ---------------------------------------------------------------------------
// corp_alliance_history
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_corp_alliance_history() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        {"record_id": 1, "start_date": "2020-01-01T00:00:00Z", "alliance_id": 99000001},
        {"record_id": 2, "start_date": "2022-06-15T00:00:00Z"}
    ]);

    Mock::given(method("GET"))
        .and(path("/corporations/98000001/alliancehistory/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let history = client.corp_alliance_history(98000001).await.unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].alliance_id, Some(99000001));
    assert_eq!(history[1].alliance_id, None);
}

// ---------------------------------------------------------------------------
// character_corporation_history
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_character_corporation_history() {
    let server = MockServer::start().await;

    let body = serde_json::json!([
        {"record_id": 1, "start_date": "2020-01-01T00:00:00Z", "corporation_id": 98000001},
        {"record_id": 2, "start_date": "2023-03-01T00:00:00Z", "corporation_id": 98000002, "is_deleted": true}
    ]);

    Mock::given(method("GET"))
        .and(path("/characters/91234567/corporationhistory/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = EsiClient::new().with_base_url(server.uri());
    let history = client
        .character_corporation_history(91234567)
        .await
        .unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].corporation_id, 98000001);
    assert!(!history[0].is_deleted);
    assert!(history[1].is_deleted);
}
