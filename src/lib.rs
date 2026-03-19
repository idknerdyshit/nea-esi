// nea-esi: Client for the EVE Swagger Interface (ESI) API.

pub mod auth;
mod endpoints;

use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT as USER_AGENT_HEADER};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

pub use auth::{EsiAppCredentials, EsiTokens, PkceChallenge};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const BASE_URL: &str = "https://esi.evetech.net/latest";
pub const THE_FORGE: i32 = 10000002;
pub const JITA_STATION: i64 = 60003760;
pub const DEFAULT_USER_AGENT: &str =
    "nea-esi (https://github.com/idknerdyshit/new-eden-analytics)";

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum EsiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("Rate limited – error budget exhausted")]
    RateLimited,

    #[error("Deserialization error: {0}")]
    Deserialize(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Token refresh error: {0}")]
    TokenRefresh(String),
}

pub type Result<T> = std::result::Result<T, EsiError>;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct EsiMarketHistoryEntry {
    pub date: String,
    pub average: f64,
    pub highest: f64,
    pub lowest: f64,
    pub volume: i64,
    pub order_count: i64,
}

// ---------------------------------------------------------------------------
// Killmail types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct EsiKillmail {
    pub killmail_id: i64,
    pub killmail_time: String,
    #[serde(default)]
    pub solar_system_id: i32,
    pub victim: EsiKillmailVictim,
    #[serde(default)]
    pub attackers: Vec<EsiKillmailAttacker>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiKillmailAttacker {
    #[serde(default)]
    pub character_id: Option<i64>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub ship_type_id: i32,
    #[serde(default)]
    pub weapon_type_id: i32,
    #[serde(default)]
    pub damage_done: i32,
    #[serde(default)]
    pub final_blow: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiCharacterInfo {
    pub name: String,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiCorporationInfo {
    pub name: String,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub member_count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiAllianceInfo {
    pub name: String,
    #[serde(default)]
    pub ticker: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiKillmailVictim {
    #[serde(default)]
    pub ship_type_id: i32,
    #[serde(default)]
    pub character_id: Option<i64>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub items: Vec<EsiKillmailItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiKillmailItem {
    pub item_type_id: i32,
    #[serde(default)]
    pub quantity_destroyed: Option<i64>,
    #[serde(default)]
    pub quantity_dropped: Option<i64>,
    #[serde(default)]
    pub flag: i32,
    #[serde(default)]
    pub singleton: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiMarketOrder {
    pub order_id: i64,
    pub type_id: i32,
    pub location_id: i64,
    pub price: f64,
    pub volume_remain: i64,
    pub is_buy_order: bool,
    pub issued: String,
    pub duration: i32,
    pub min_volume: i32,
    pub range: String,
}

/// A single item in a character's asset list.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiAssetItem {
    pub item_id: i64,
    pub type_id: i32,
    pub location_id: i64,
    pub location_type: String,
    pub location_flag: String,
    pub quantity: i32,
    #[serde(default)]
    pub is_singleton: bool,
    #[serde(default)]
    pub is_blueprint_copy: Option<bool>,
}

/// Resolved name from POST /universe/names/.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiResolvedName {
    pub id: i64,
    pub name: String,
    pub category: String,
}

/// Player-owned structure info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiStructureInfo {
    pub name: String,
    #[serde(default)]
    pub owner_id: i64,
    #[serde(default)]
    pub solar_system_id: i32,
    #[serde(default)]
    pub type_id: Option<i32>,
}

/// Global average/adjusted price for a type.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiMarketPrice {
    pub type_id: i32,
    #[serde(default)]
    pub average_price: Option<f64>,
    #[serde(default)]
    pub adjusted_price: Option<f64>,
}

// ---------------------------------------------------------------------------
// EsiClient
// ---------------------------------------------------------------------------

pub struct EsiClient {
    pub(crate) client: reqwest::Client,
    pub(crate) semaphore: Arc<tokio::sync::Semaphore>,
    pub(crate) error_budget: Arc<AtomicI32>,
    pub(crate) tokens: Arc<tokio::sync::RwLock<Option<EsiTokens>>>,
    pub(crate) app_credentials: Option<EsiAppCredentials>,
}

impl EsiClient {
    /// Create a new ESI client with the default User-Agent and 30-second timeout.
    pub fn new() -> Self {
        Self::with_user_agent(DEFAULT_USER_AGENT)
    }

    /// Create a new ESI client with a custom User-Agent string and 30-second timeout.
    ///
    /// ESI requires a descriptive User-Agent. Include your app name, contact info,
    /// and optionally your EVE character name. Example:
    ///
    /// ```text
    /// my-app (contact@example.com; +https://github.com/me/my-app; eve:MyCharacter)
    /// ```
    pub fn with_user_agent(user_agent: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT_HEADER,
            HeaderValue::from_str(user_agent).expect("invalid user-agent string"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            semaphore: Arc::new(tokio::sync::Semaphore::new(20)),
            error_budget: Arc::new(AtomicI32::new(100)),
            tokens: Arc::new(tokio::sync::RwLock::new(None)),
            app_credentials: None,
        }
    }

    /// Create an ESI client configured for a web application (confidential client).
    pub fn with_web_app(user_agent: &str, client_id: &str, client_secret: SecretString) -> Self {
        let mut client = Self::with_user_agent(user_agent);
        client.app_credentials = Some(EsiAppCredentials::Web {
            client_id: client_id.to_string(),
            client_secret,
        });
        client
    }

    /// Create an ESI client configured for a native/desktop application (public client).
    pub fn with_native_app(user_agent: &str, client_id: &str) -> Self {
        let mut client = Self::with_user_agent(user_agent);
        client.app_credentials = Some(EsiAppCredentials::Native {
            client_id: client_id.to_string(),
        });
        client
    }

    /// Set app credentials (builder pattern).
    pub fn credentials(mut self, creds: EsiAppCredentials) -> Self {
        self.app_credentials = Some(creds);
        self
    }

    /// Return the current error budget value.
    pub fn error_budget(&self) -> i32 {
        self.error_budget.load(Ordering::Relaxed)
    }

    // -----------------------------------------------------------------------
    // Core request helper
    // -----------------------------------------------------------------------

    /// Make a rate-limited GET request to the given URL.
    ///
    /// Acquires a semaphore permit (max 20 concurrent), performs the request,
    /// reads the `X-ESI-Error-Limit-Remain` header to update the error budget,
    /// and returns the response. If OAuth tokens are configured, attaches a
    /// Bearer header automatically. On 401, attempts one token refresh and retry.
    pub async fn request(&self, url: &str) -> Result<reqwest::Response> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| EsiError::Internal("rate-limit semaphore closed".into()))?;

        // If the error budget is very low, back off briefly.
        let budget = self.error_budget.load(Ordering::Relaxed);
        if budget < 20 {
            warn!(
                budget,
                "ESI error budget low – adding 1 s delay before request"
            );
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        // If budget is zero we refuse to make the call.
        if self.error_budget.load(Ordering::Relaxed) <= 0 {
            return Err(EsiError::RateLimited);
        }

        // Get a valid token if we have one.
        let token = self.ensure_valid_token().await?;

        let start = std::time::Instant::now();
        let mut req = self.client.get(url);
        if let Some(ref tok) = token {
            req = req.bearer_auth(tok.expose_secret());
        }
        let response = req.send().await?;

        // Update error budget from response header.
        if let Some(val) = response.headers().get("x-esi-error-limit-remain") {
            if let Ok(s) = val.to_str() {
                if let Ok(remain) = s.parse::<i32>() {
                    self.error_budget.store(remain, Ordering::Relaxed);
                }
            }
        }

        // If 401 and we have tokens, try refreshing once and retry.
        if response.status().as_u16() == 401 && token.is_some() {
            debug!("got 401, attempting token refresh and retry");
            let refreshed = self.refresh_token().await?;
            let retry_resp = self
                .client
                .get(url)
                .bearer_auth(refreshed.access_token.expose_secret())
                .send()
                .await?;

            // Update error budget from retry response.
            if let Some(val) = retry_resp.headers().get("x-esi-error-limit-remain") {
                if let Ok(s) = val.to_str() {
                    if let Ok(remain) = s.parse::<i32>() {
                        self.error_budget.store(remain, Ordering::Relaxed);
                    }
                }
            }

            if !retry_resp.status().is_success() {
                let status = retry_resp.status().as_u16();
                let message = retry_resp.text().await.unwrap_or_default();
                warn!(url, status, "ESI API error after token refresh retry");
                return Err(EsiError::Api { status, message });
            }

            debug!(
                url,
                status = retry_resp.status().as_u16(),
                elapsed_ms = start.elapsed().as_millis() as u64,
                "ESI request (after 401 retry)"
            );

            return Ok(retry_resp);
        }

        // If the response is an error status, return an Api error.
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            warn!(url, status, "ESI API error");
            return Err(EsiError::Api { status, message });
        }

        debug!(
            url,
            status = response.status().as_u16(),
            elapsed_ms = start.elapsed().as_millis() as u64,
            error_budget = self.error_budget.load(Ordering::Relaxed),
            "ESI request"
        );

        Ok(response)
    }

    /// Make a rate-limited POST request with a JSON body.
    ///
    /// Same flow as `request()`: semaphore acquire, budget check, optional
    /// bearer auth, error budget update, and 401 retry.
    pub async fn request_post(
        &self,
        url: &str,
        body: &impl Serialize,
    ) -> Result<reqwest::Response> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| EsiError::Internal("rate-limit semaphore closed".into()))?;

        let budget = self.error_budget.load(Ordering::Relaxed);
        if budget < 20 {
            warn!(
                budget,
                "ESI error budget low – adding 1 s delay before request"
            );
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        if self.error_budget.load(Ordering::Relaxed) <= 0 {
            return Err(EsiError::RateLimited);
        }

        let token = self.ensure_valid_token().await?;

        let start = std::time::Instant::now();
        let mut req = self.client.post(url).json(body);
        if let Some(ref tok) = token {
            req = req.bearer_auth(tok.expose_secret());
        }
        let response = req.send().await?;

        if let Some(val) = response.headers().get("x-esi-error-limit-remain") {
            if let Ok(s) = val.to_str() {
                if let Ok(remain) = s.parse::<i32>() {
                    self.error_budget.store(remain, Ordering::Relaxed);
                }
            }
        }

        if response.status().as_u16() == 401 && token.is_some() {
            debug!("got 401 on POST, attempting token refresh and retry");
            let refreshed = self.refresh_token().await?;
            let retry_resp = self
                .client
                .post(url)
                .json(body)
                .bearer_auth(refreshed.access_token.expose_secret())
                .send()
                .await?;

            if let Some(val) = retry_resp.headers().get("x-esi-error-limit-remain") {
                if let Ok(s) = val.to_str() {
                    if let Ok(remain) = s.parse::<i32>() {
                        self.error_budget.store(remain, Ordering::Relaxed);
                    }
                }
            }

            if !retry_resp.status().is_success() {
                let status = retry_resp.status().as_u16();
                let message = retry_resp.text().await.unwrap_or_default();
                warn!(url, status, "ESI API error after token refresh retry (POST)");
                return Err(EsiError::Api { status, message });
            }

            debug!(
                url,
                status = retry_resp.status().as_u16(),
                elapsed_ms = start.elapsed().as_millis() as u64,
                "ESI POST request (after 401 retry)"
            );

            return Ok(retry_resp);
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            warn!(url, status, "ESI API error (POST)");
            return Err(EsiError::Api { status, message });
        }

        debug!(
            url,
            status = response.status().as_u16(),
            elapsed_ms = start.elapsed().as_millis() as u64,
            error_budget = self.error_budget.load(Ordering::Relaxed),
            "ESI POST request"
        );

        Ok(response)
    }
}

impl Default for EsiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_order(
        order_id: i64,
        location_id: i64,
        price: f64,
        volume_remain: i64,
        is_buy: bool,
    ) -> EsiMarketOrder {
        EsiMarketOrder {
            order_id,
            type_id: 34,
            location_id,
            price,
            volume_remain,
            is_buy_order: is_buy,
            issued: "2026-01-01T00:00:00Z".to_string(),
            duration: 90,
            min_volume: 1,
            range: "station".to_string(),
        }
    }

    #[test]
    fn test_compute_best_bid_ask_empty() {
        let (bid, ask, bv, av) = EsiClient::compute_best_bid_ask(&[], JITA_STATION);
        assert_eq!((bid, ask, bv, av), (None, None, 0, 0));
    }

    #[test]
    fn test_compute_best_bid_ask_wrong_station() {
        let orders = vec![make_order(1, 99999, 10.0, 100, true)];
        let (bid, ask, bv, av) = EsiClient::compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!((bid, ask, bv, av), (None, None, 0, 0));
    }

    #[test]
    fn test_compute_best_bid_ask_buys_only() {
        let orders = vec![
            make_order(1, JITA_STATION, 10.0, 100, true),
            make_order(2, JITA_STATION, 12.0, 200, true),
        ];
        let (bid, ask, bv, av) = EsiClient::compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!(bid, Some(12.0));
        assert_eq!(ask, None);
        assert_eq!(bv, 300);
        assert_eq!(av, 0);
    }

    #[test]
    fn test_compute_best_bid_ask_sells_only() {
        let orders = vec![
            make_order(1, JITA_STATION, 15.0, 50, false),
            make_order(2, JITA_STATION, 13.0, 75, false),
        ];
        let (bid, ask, bv, av) = EsiClient::compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!(bid, None);
        assert_eq!(ask, Some(13.0));
        assert_eq!(bv, 0);
        assert_eq!(av, 125);
    }

    #[test]
    fn test_compute_best_bid_ask_mixed() {
        let orders = vec![
            make_order(1, JITA_STATION, 10.0, 100, true),
            make_order(2, JITA_STATION, 12.0, 200, true),
            make_order(3, JITA_STATION, 15.0, 50, false),
            make_order(4, JITA_STATION, 13.0, 75, false),
        ];
        let (bid, ask, bv, av) = EsiClient::compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!(bid, Some(12.0));
        assert_eq!(ask, Some(13.0));
        assert_eq!(bv, 300);
        assert_eq!(av, 125);
    }

    #[test]
    fn test_compute_best_bid_ask_multi_station() {
        let amarr: i64 = 60008494;
        let orders = vec![
            make_order(1, JITA_STATION, 10.0, 100, true),
            make_order(2, amarr, 99.0, 999, true),
            make_order(3, JITA_STATION, 15.0, 50, false),
            make_order(4, amarr, 1.0, 999, false),
        ];
        let (bid, ask, bv, av) = EsiClient::compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!(bid, Some(10.0));
        assert_eq!(ask, Some(15.0));
        assert_eq!(bv, 100);
        assert_eq!(av, 50);
    }

    #[test]
    fn test_deserialize_esi_killmail() {
        let json = r#"{
            "killmail_id": 123456,
            "killmail_time": "2026-03-17T12:00:00Z",
            "solar_system_id": 30000142,
            "victim": {
                "ship_type_id": 587,
                "character_id": 91234567,
                "corporation_id": 98000001,
                "alliance_id": null,
                "items": [
                    {
                        "item_type_id": 2032,
                        "quantity_destroyed": 1,
                        "quantity_dropped": null,
                        "flag": 27,
                        "singleton": 0
                    },
                    {
                        "item_type_id": 3170,
                        "quantity_destroyed": null,
                        "quantity_dropped": 5,
                        "flag": 11,
                        "singleton": 0
                    }
                ]
            },
            "attackers": [
                {
                    "character_id": 95000001,
                    "corporation_id": 98000002,
                    "ship_type_id": 24690,
                    "weapon_type_id": 3170,
                    "damage_done": 5000,
                    "final_blow": true
                },
                {
                    "corporation_id": 1000125,
                    "ship_type_id": 0,
                    "weapon_type_id": 0,
                    "damage_done": 100,
                    "final_blow": false
                }
            ]
        }"#;

        let km: EsiKillmail = serde_json::from_str(json).unwrap();
        assert_eq!(km.killmail_id, 123456);
        assert_eq!(km.killmail_time, "2026-03-17T12:00:00Z");
        assert_eq!(km.solar_system_id, 30000142);
        assert_eq!(km.victim.ship_type_id, 587);
        assert_eq!(km.victim.character_id, Some(91234567));
        assert_eq!(km.victim.alliance_id, None);
        assert_eq!(km.victim.items.len(), 2);
        assert_eq!(km.victim.items[0].item_type_id, 2032);
        assert_eq!(km.victim.items[0].quantity_destroyed, Some(1));
        assert_eq!(km.victim.items[1].item_type_id, 3170);
        assert_eq!(km.victim.items[1].quantity_dropped, Some(5));
        assert_eq!(km.attackers.len(), 2);
        assert_eq!(km.attackers[0].character_id, Some(95000001));
        assert_eq!(km.attackers[0].ship_type_id, 24690);
        assert_eq!(km.attackers[0].damage_done, 5000);
        assert!(km.attackers[0].final_blow);
        assert_eq!(km.attackers[1].character_id, None);
        assert!(!km.attackers[1].final_blow);
    }

    #[test]
    fn test_deserialize_esi_killmail_minimal() {
        let json = r#"{
            "killmail_id": 999,
            "killmail_time": "2026-01-01T00:00:00Z",
            "solar_system_id": 30000001,
            "victim": {
                "ship_type_id": 670
            }
        }"#;

        let km: EsiKillmail = serde_json::from_str(json).unwrap();
        assert_eq!(km.killmail_id, 999);
        assert_eq!(km.victim.ship_type_id, 670);
        assert!(km.victim.items.is_empty());
        assert_eq!(km.victim.character_id, None);
    }

    #[test]
    fn test_deserialize_market_history_entry() {
        let json = r#"{"date":"2026-03-01","average":5.25,"highest":5.27,"lowest":5.11,"volume":72016862,"order_count":2267}"#;
        let entry: EsiMarketHistoryEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.date, "2026-03-01");
        assert!((entry.average - 5.25).abs() < f64::EPSILON);
        assert_eq!(entry.volume, 72016862);
        assert_eq!(entry.order_count, 2267);
    }

    #[test]
    fn test_deserialize_esi_asset_item() {
        let json = r#"{
            "item_id": 1234567890,
            "type_id": 587,
            "location_id": 60003760,
            "location_type": "station",
            "location_flag": "Hangar",
            "quantity": 1,
            "is_singleton": true,
            "is_blueprint_copy": null
        }"#;
        let item: EsiAssetItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.item_id, 1234567890);
        assert_eq!(item.type_id, 587);
        assert_eq!(item.location_id, 60003760);
        assert_eq!(item.location_type, "station");
        assert_eq!(item.location_flag, "Hangar");
        assert_eq!(item.quantity, 1);
        assert!(item.is_singleton);
        assert_eq!(item.is_blueprint_copy, None);
    }

    #[test]
    fn test_deserialize_esi_resolved_name() {
        let json = r#"{"id": 95465499, "name": "CCP Bartender", "category": "character"}"#;
        let name: EsiResolvedName = serde_json::from_str(json).unwrap();
        assert_eq!(name.id, 95465499);
        assert_eq!(name.name, "CCP Bartender");
        assert_eq!(name.category, "character");
    }

    #[test]
    fn test_deserialize_esi_structure_info() {
        let json = r#"{
            "name": "My Citadel",
            "owner_id": 98000001,
            "solar_system_id": 30000142,
            "type_id": 35832
        }"#;
        let info: EsiStructureInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "My Citadel");
        assert_eq!(info.owner_id, 98000001);
        assert_eq!(info.solar_system_id, 30000142);
        assert_eq!(info.type_id, Some(35832));
    }

    #[test]
    fn test_deserialize_esi_market_price() {
        let json = r#"{"type_id": 34, "average_price": 5.25}"#;
        let price: EsiMarketPrice = serde_json::from_str(json).unwrap();
        assert_eq!(price.type_id, 34);
        assert!((price.average_price.unwrap() - 5.25).abs() < f64::EPSILON);
        assert_eq!(price.adjusted_price, None);
    }

    #[test]
    fn test_deserialize_market_order() {
        let json = r#"{"order_id":6789012345,"type_id":34,"location_id":60003760,"price":5.13,"volume_remain":250000,"is_buy_order":true,"issued":"2026-03-10T08:15:00Z","duration":90,"min_volume":1,"range":"station"}"#;
        let order: EsiMarketOrder = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_id, 6789012345);
        assert_eq!(order.type_id, 34);
        assert!(order.is_buy_order);
        assert_eq!(order.location_id, JITA_STATION);
    }
}
