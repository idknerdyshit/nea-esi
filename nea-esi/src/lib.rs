// nea-esi: Client for the EVE Swagger Interface (ESI) API.
#![allow(clippy::missing_errors_doc)]

pub mod auth;
mod endpoints;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::time::Duration;

use rand::RngExt;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT as USER_AGENT_HEADER};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, warn};

pub use auth::{EsiAppCredentials, EsiTokens, PkceChallenge};
pub use endpoints::compute_best_bid_ask;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const BASE_URL: &str = "https://esi.evetech.net/latest";
pub const THE_FORGE: i32 = 10_000_002;
pub const DOMAIN: i32 = 10_000_043;
pub const SINQ_LAISON: i32 = 10_000_032;
pub const HEIMATAR: i32 = 10_000_030;
pub const METROPOLIS: i32 = 10_000_042;
pub const JITA_STATION: i64 = 60_003_760;
pub const AMARR_STATION: i64 = 60_008_494;
pub const DODIXIE_STATION: i64 = 60_011_866;
pub const RENS_STATION: i64 = 60_004_588;
pub const HEK_STATION: i64 = 60_005_686;
pub const DEFAULT_USER_AGENT: &str = "nea-esi (https://github.com/idknerdyshit/new-eden-analytics)";

const MAX_RETRIES: u32 = 3;
const RETRY_BASE_MS: u64 = 1000;

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

    #[error("Config error: {0}")]
    Config(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Token refresh error: {0}")]
    TokenRefresh(String),
}

pub type Result<T> = std::result::Result<T, EsiError>;

mod types;
pub use types::*;

// ---------------------------------------------------------------------------
// ETag cache
// ---------------------------------------------------------------------------

struct CachedResponse {
    etag: String,
    body: Vec<u8>,
}

// ---------------------------------------------------------------------------
// EsiClient
// ---------------------------------------------------------------------------

pub struct EsiClient {
    pub(crate) client: reqwest::Client,
    pub(crate) semaphore: Arc<tokio::sync::Semaphore>,
    pub(crate) error_budget: Arc<AtomicI32>,
    /// Unix epoch (seconds) at which the error budget resets.
    pub(crate) error_budget_reset_at: Arc<AtomicU64>,
    pub(crate) tokens: Arc<tokio::sync::RwLock<Option<EsiTokens>>>,
    pub(crate) app_credentials: Option<EsiAppCredentials>,
    /// Serializes token refresh operations to prevent concurrent refreshes
    /// from racing on the same refresh token.
    pub(crate) refresh_mutex: Arc<tokio::sync::Mutex<()>>,
    cache: Option<Arc<RwLock<HashMap<String, CachedResponse>>>>,
    max_cache_entries: usize,
    base_url: String,
    pub(crate) sso_token_url: String,
}

impl EsiClient {
    /// Create a new ESI client with the default User-Agent and 30-second timeout.
    ///
    /// # Panics
    ///
    /// Panics if the default user-agent string is invalid (this should never happen
    /// as it is a compile-time constant).
    #[must_use]
    pub fn new() -> Self {
        // SAFETY: DEFAULT_USER_AGENT is a compile-time constant with valid ASCII.
        Self::with_user_agent(DEFAULT_USER_AGENT).expect("default user-agent is valid")
    }

    /// Create a new ESI client with a custom User-Agent string and 30-second timeout.
    ///
    /// Returns an error if the user-agent string contains invalid HTTP header characters.
    ///
    /// ESI requires a descriptive User-Agent. Include your app name, contact info,
    /// and optionally your EVE character name. Example:
    ///
    /// ```text
    /// my-app (contact@example.com; +https://github.com/me/my-app; eve:MyCharacter)
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the `reqwest::Client` builder fails to build (should not happen
    /// with default settings).
    pub fn with_user_agent(user_agent: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT_HEADER,
            HeaderValue::from_str(user_agent)
                .map_err(|e| EsiError::Config(format!("invalid user-agent string: {e}")))?,
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");

        Ok(Self {
            client,
            semaphore: Arc::new(tokio::sync::Semaphore::new(20)),
            error_budget: Arc::new(AtomicI32::new(100)),
            error_budget_reset_at: Arc::new(AtomicU64::new(0)),
            tokens: Arc::new(tokio::sync::RwLock::new(None)),
            app_credentials: None,
            refresh_mutex: Arc::new(tokio::sync::Mutex::new(())),
            cache: None,
            max_cache_entries: 1000,
            base_url: BASE_URL.to_string(),
            sso_token_url: auth::SSO_TOKEN_URL.to_string(),
        })
    }

    /// Create an ESI client configured for a web application (confidential client).
    pub fn with_web_app(
        user_agent: &str,
        client_id: &str,
        client_secret: SecretString,
    ) -> Result<Self> {
        let mut client = Self::with_user_agent(user_agent)?;
        client.app_credentials = Some(EsiAppCredentials::Web {
            client_id: client_id.to_string(),
            client_secret,
        });
        Ok(client)
    }

    /// Create an ESI client configured for a native/desktop application (public client).
    pub fn with_native_app(user_agent: &str, client_id: &str) -> Result<Self> {
        let mut client = Self::with_user_agent(user_agent)?;
        client.app_credentials = Some(EsiAppCredentials::Native {
            client_id: client_id.to_string(),
        });
        Ok(client)
    }

    /// Set app credentials (builder pattern).
    #[must_use]
    pub fn credentials(mut self, creds: EsiAppCredentials) -> Self {
        self.app_credentials = Some(creds);
        self
    }

    /// Override the base URL (builder pattern). Useful for testing with mock servers.
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Override the SSO token URL (builder pattern). Useful for testing with mock servers.
    #[must_use]
    pub fn with_sso_token_url(mut self, url: impl Into<String>) -> Self {
        self.sso_token_url = url.into();
        self
    }

    /// Return the current error budget value.
    #[must_use]
    pub fn error_budget(&self) -> i32 {
        self.error_budget.load(Ordering::Relaxed)
    }

    /// Read `X-ESI-Error-Limit-Remain` and `X-ESI-Error-Limit-Reset` from
    /// response headers, updating the stored error budget and reset deadline.
    fn update_error_budget(&self, headers: &reqwest::header::HeaderMap) {
        if let Some(val) = headers.get("x-esi-error-limit-remain")
            && let Ok(s) = val.to_str()
            && let Ok(remain) = s.parse::<i32>()
        {
            self.error_budget.store(remain, Ordering::Relaxed);
        }
        if let Some(val) = headers.get("x-esi-error-limit-reset")
            && let Ok(s) = val.to_str()
            && let Ok(secs) = s.parse::<u64>()
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            self.error_budget_reset_at
                .store(now + secs, Ordering::Relaxed);
        }
    }

    /// When the error budget is low, sleep until the reset window instead of a
    /// flat delay. Falls back to 60 s if no reset header was ever received.
    async fn wait_for_budget_reset(&self) {
        let budget = self.error_budget.load(Ordering::Relaxed);
        if budget < 20 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let reset_at = self.error_budget_reset_at.load(Ordering::Relaxed);
            let wait_secs = if reset_at > now {
                reset_at - now
            } else {
                // No reset header seen yet – fall back to a conservative wait.
                60
            };
            warn!(
                budget,
                wait_secs, "ESI error budget low – sleeping until reset"
            );
            tokio::time::sleep(Duration::from_secs(wait_secs)).await;
        }
    }

    /// Enable the `ETag` response cache (builder pattern).
    #[must_use]
    pub fn with_cache(mut self) -> Self {
        self.cache = Some(Arc::new(RwLock::new(HashMap::new())));
        self
    }

    /// Set the maximum number of `ETag` cache entries (builder pattern).
    /// Default is 1000. When the cache is full, an arbitrary entry is evicted.
    #[must_use]
    pub fn with_max_cache_entries(mut self, n: usize) -> Self {
        self.max_cache_entries = n;
        self
    }

    /// Clear all cached `ETag` responses.
    pub async fn clear_cache(&self) {
        if let Some(ref cache) = self.cache {
            cache.write().await.clear();
        }
    }

    /// Create a lightweight clone that shares all Arc-wrapped state.
    pub(crate) fn clone_shared(&self) -> Self {
        Self {
            client: self.client.clone(),
            semaphore: Arc::clone(&self.semaphore),
            error_budget: Arc::clone(&self.error_budget),
            error_budget_reset_at: Arc::clone(&self.error_budget_reset_at),
            tokens: Arc::clone(&self.tokens),
            app_credentials: self.app_credentials.clone(),
            refresh_mutex: Arc::clone(&self.refresh_mutex),
            cache: self.cache.as_ref().map(Arc::clone),
            max_cache_entries: self.max_cache_entries,
            base_url: self.base_url.clone(),
            sso_token_url: self.sso_token_url.clone(),
        }
    }

    // -----------------------------------------------------------------------
    // Pagination
    // -----------------------------------------------------------------------

    /// Fetch all pages of a paginated GET endpoint and flatten into one Vec.
    pub async fn get_paginated<T: DeserializeOwned + Send + 'static>(
        &self,
        base_url: &str,
    ) -> Result<Vec<T>> {
        self.paginated_fetch(base_url, PageFetcher::Get).await
    }

    /// Fetch all pages of a paginated POST endpoint and flatten into one Vec.
    pub async fn post_paginated<T, B>(&self, base_url: &str, body: &B) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + Sync,
    {
        let body_bytes = serde_json::to_vec(body)
            .map_err(|e| EsiError::Internal(format!("failed to serialize body: {e}")))?;
        self.paginated_fetch(base_url, PageFetcher::Post(Arc::new(body_bytes)))
            .await
    }

    /// Shared pagination logic for both GET and POST.
    async fn paginated_fetch<T: DeserializeOwned + Send + 'static>(
        &self,
        base_url: &str,
        fetcher: PageFetcher,
    ) -> Result<Vec<T>> {
        let separator = if base_url.contains('?') { '&' } else { '?' };
        let first_url = format!("{base_url}{separator}page=1");

        let resp = match &fetcher {
            PageFetcher::Get => self.request(&first_url).await?,
            PageFetcher::Post(body) => self.request_post_raw(&first_url, body).await?,
        };

        let total_pages: i32 = resp
            .headers()
            .get("x-pages")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let mut items: Vec<T> = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;

        if total_pages > 1 {
            // Fetch remaining pages in batches to limit concurrent task count.
            let remaining_pages: Vec<i32> = (2..=total_pages).collect();
            for batch in remaining_pages.chunks(20) {
                let mut handles = Vec::with_capacity(batch.len());
                for &page in batch {
                    let url = format!("{base_url}{separator}page={page}");
                    let this = self.clone_shared();
                    let fetcher = fetcher.clone();
                    handles.push(tokio::spawn(async move {
                        let resp = match &fetcher {
                            PageFetcher::Get => this.request(&url).await?,
                            PageFetcher::Post(body) => this.request_post_raw(&url, body).await?,
                        };
                        let page_items: Vec<T> = resp
                            .json()
                            .await
                            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
                        Ok::<_, EsiError>(page_items)
                    }));
                }

                for handle in handles {
                    let page_items = handle
                        .await
                        .map_err(|e| EsiError::Deserialize(e.to_string()))??;
                    items.extend(page_items);
                }
            }
        }

        Ok(items)
    }

    // -----------------------------------------------------------------------
    // ETag caching
    // -----------------------------------------------------------------------

    /// Make a GET request with `ETag` caching support.
    ///
    /// Uses `execute_request` internally for retry/401 handling. On 304,
    /// returns the cached body. On 200, caches the response.
    pub async fn request_cached(&self, url: &str) -> Result<Vec<u8>> {
        let cached_etag = if let Some(ref cache) = self.cache {
            let guard = cache.read().await;
            guard.get(url).map(|c| c.etag.clone())
        } else {
            None
        };

        let etag_clone = cached_etag.clone();
        let result = self
            .execute_request(url, move |client, url| {
                let mut req = client.get(url);
                if let Some(ref etag) = etag_clone {
                    req = req.header("If-None-Match", etag.as_str());
                }
                req
            })
            .await;

        // Handle 304 Not Modified by returning cached body.
        if let Err(EsiError::Api { status: 304, .. }) = &result
            && let Some(ref cache) = self.cache
        {
            let guard = cache.read().await;
            if let Some(cached) = guard.get(url) {
                debug!(url, "ETag cache hit (304)");
                return Ok(cached.body.clone());
            }
        }

        let response = result?;

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let body = response.bytes().await.map_err(EsiError::Http)?.to_vec();

        if let (Some(cache), Some(etag)) = (&self.cache, etag) {
            let mut guard = cache.write().await;
            // Evict an arbitrary entry if the cache is at capacity.
            if guard.len() >= self.max_cache_entries
                && let Some(key) = guard.keys().next().cloned()
            {
                guard.remove(&key);
            }
            guard.insert(
                url.to_string(),
                CachedResponse {
                    etag,
                    body: body.clone(),
                },
            );
        }

        Ok(body)
    }

    // -----------------------------------------------------------------------
    // Core request helpers
    // -----------------------------------------------------------------------

    /// Unified request executor with semaphore, budget check, auth, retry
    /// (502-504 and network errors), and 401 token refresh.
    async fn execute_request(
        &self,
        url: &str,
        build_request: impl Fn(&reqwest::Client, &str) -> reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| EsiError::Internal("rate-limit semaphore closed".into()))?;

        self.wait_for_budget_reset().await;

        if self.error_budget.load(Ordering::Relaxed) <= 0 {
            return Err(EsiError::RateLimited);
        }

        let token = self.ensure_valid_token().await?;
        let start = std::time::Instant::now();

        // Retry loop for transient 502/503/504 errors and network errors.
        let response = {
            let mut last_err = None;
            let mut resp = None;
            for attempt in 0..=MAX_RETRIES {
                let mut req = build_request(&self.client, url);
                if let Some(ref tok) = token {
                    req = req.bearer_auth(tok.expose_secret());
                }
                match req.send().await {
                    Ok(r) => {
                        self.update_error_budget(r.headers());
                        let status = r.status().as_u16();
                        if matches!(status, 502..=504) && attempt < MAX_RETRIES {
                            let jitter = rand::rng().random_range(0..500);
                            let delay = RETRY_BASE_MS * 2u64.pow(attempt) + jitter;
                            warn!(
                                url,
                                status,
                                attempt,
                                delay_ms = delay,
                                "retrying transient error"
                            );
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                            continue;
                        }
                        resp = Some(r);
                        break;
                    }
                    Err(e) => {
                        if attempt < MAX_RETRIES {
                            let jitter = rand::rng().random_range(0..500);
                            let delay = RETRY_BASE_MS * 2u64.pow(attempt) + jitter;
                            warn!(url, attempt, delay_ms = delay, error = %e, "retrying network error");
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                            continue;
                        }
                        last_err = Some(e);
                        break;
                    }
                }
            }
            match resp {
                Some(r) => r,
                None => return Err(EsiError::Http(last_err.unwrap())),
            }
        };

        // If 401 and we have tokens, try refreshing once and retry.
        // Note: this single retry does not go through the 502/503/504 retry loop.
        if response.status().as_u16() == 401 && token.is_some() {
            debug!("got 401, attempting token refresh and retry");
            let refreshed = self.refresh_token().await?;
            let retry_resp = build_request(&self.client, url)
                .bearer_auth(refreshed.access_token.expose_secret())
                .send()
                .await?;

            self.update_error_budget(retry_resp.headers());

            if !retry_resp.status().is_success() {
                let status = retry_resp.status().as_u16();
                let message = retry_resp.text().await.unwrap_or_default();
                warn!(url, status, "ESI API error after token refresh retry");
                return Err(EsiError::Api { status, message });
            }

            #[allow(clippy::cast_possible_truncation)]
            let elapsed_ms = start.elapsed().as_millis() as u64;
            debug!(
                url,
                status = retry_resp.status().as_u16(),
                elapsed_ms,
                "ESI request (after 401 retry)"
            );

            return Ok(retry_resp);
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            warn!(url, status, "ESI API error");
            return Err(EsiError::Api { status, message });
        }

        #[allow(clippy::cast_possible_truncation)]
        let elapsed_ms = start.elapsed().as_millis() as u64;
        debug!(
            url,
            status = response.status().as_u16(),
            elapsed_ms,
            error_budget = self.error_budget.load(Ordering::Relaxed),
            "ESI request"
        );

        Ok(response)
    }

    /// Make a rate-limited GET request to the given URL.
    pub async fn request(&self, url: &str) -> Result<reqwest::Response> {
        self.execute_request(url, |client, url| client.get(url))
            .await
    }

    /// Make a rate-limited POST request with a JSON body.
    pub async fn request_post(
        &self,
        url: &str,
        body: &(impl Serialize + ?Sized),
    ) -> Result<reqwest::Response> {
        let body_bytes = serde_json::to_vec(body)
            .map_err(|e| EsiError::Internal(format!("failed to serialize body: {e}")))?;
        self.execute_request(url, move |client, url| {
            client
                .post(url)
                .header("content-type", "application/json")
                .body(body_bytes.clone())
        })
        .await
    }

    /// Make a rate-limited POST request with a pre-serialized JSON body.
    async fn request_post_raw(&self, url: &str, body: &Arc<Vec<u8>>) -> Result<reqwest::Response> {
        let body = Arc::clone(body);
        self.execute_request(url, move |client, url| {
            client
                .post(url)
                .header("content-type", "application/json")
                .body(body.as_ref().clone())
        })
        .await
    }

    /// Make a rate-limited DELETE request.
    pub async fn request_delete(&self, url: &str) -> Result<reqwest::Response> {
        self.execute_request(url, |client, url| client.delete(url))
            .await
    }

    /// Make a rate-limited PUT request with a JSON body.
    pub async fn request_put(
        &self,
        url: &str,
        body: &(impl Serialize + ?Sized),
    ) -> Result<reqwest::Response> {
        let body_bytes = serde_json::to_vec(body)
            .map_err(|e| EsiError::Internal(format!("failed to serialize body: {e}")))?;
        self.execute_request(url, move |client, url| {
            client
                .put(url)
                .header("content-type", "application/json")
                .body(body_bytes.clone())
        })
        .await
    }
}

/// Internal enum to dispatch between GET and POST in paginated fetches.
#[derive(Clone)]
enum PageFetcher {
    Get,
    Post(Arc<Vec<u8>>),
}

impl Default for EsiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDate, Utc};

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
            issued: "2026-01-01T00:00:00Z".parse().unwrap(),
            duration: 90,
            min_volume: 1,
            range: "station".to_string(),
        }
    }

    #[test]
    fn test_compute_best_bid_ask_empty() {
        let (bid, ask, bv, av) = compute_best_bid_ask(&[], JITA_STATION);
        assert_eq!((bid, ask, bv, av), (None, None, 0, 0));
    }

    #[test]
    fn test_compute_best_bid_ask_wrong_station() {
        let orders = vec![make_order(1, 99999, 10.0, 100, true)];
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!((bid, ask, bv, av), (None, None, 0, 0));
    }

    #[test]
    fn test_compute_best_bid_ask_buys_only() {
        let orders = vec![
            make_order(1, JITA_STATION, 10.0, 100, true),
            make_order(2, JITA_STATION, 12.0, 200, true),
        ];
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
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
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
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
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
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
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
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
        assert_eq!(
            km.killmail_time,
            "2026-03-17T12:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
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
        assert_eq!(entry.date, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
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

    // -----------------------------------------------------------------------
    // Phase 1 deserialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_deserialize_type_info() {
        let json = r#"{
            "type_id": 587,
            "name": "Rifter",
            "description": "A Minmatar frigate.",
            "group_id": 25,
            "market_group_id": 61,
            "mass": 1067000.0,
            "volume": 27289.0,
            "packaged_volume": 2500.0,
            "capacity": 130.0,
            "published": true,
            "portion_size": 1,
            "icon_id": 587,
            "graphic_id": 46
        }"#;
        let info: EsiTypeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.type_id, 587);
        assert_eq!(info.name, "Rifter");
        assert_eq!(info.group_id, 25);
        assert_eq!(info.market_group_id, Some(61));
        assert!(info.published);
    }

    #[test]
    fn test_deserialize_type_info_minimal() {
        let json = r#"{"type_id": 34, "name": "Tritanium", "group_id": 18, "published": true}"#;
        let info: EsiTypeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.type_id, 34);
        assert_eq!(info.name, "Tritanium");
        assert_eq!(info.group_id, 18);
        assert!(info.published);
        assert_eq!(info.market_group_id, None);
    }

    #[test]
    fn test_deserialize_group_info() {
        let json = r#"{
            "group_id": 25,
            "name": "Frigate",
            "category_id": 6,
            "published": true,
            "types": [587, 603, 608]
        }"#;
        let info: EsiGroupInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.group_id, 25);
        assert_eq!(info.name, "Frigate");
        assert_eq!(info.category_id, 6);
        assert_eq!(info.types.len(), 3);
    }

    #[test]
    fn test_deserialize_category_info() {
        let json = r#"{
            "category_id": 6,
            "name": "Ship",
            "published": true,
            "groups": [25, 26, 27]
        }"#;
        let info: EsiCategoryInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.category_id, 6);
        assert_eq!(info.name, "Ship");
        assert_eq!(info.groups.len(), 3);
    }

    #[test]
    fn test_deserialize_solar_system_info() {
        let json = r#"{
            "system_id": 30000142,
            "name": "Jita",
            "constellation_id": 20000020,
            "security_status": 0.9459131,
            "security_class": "B",
            "star_id": 40009081,
            "stargates": [50001248, 50001249],
            "stations": [60003760],
            "planets": [
                {"planet_id": 40009082, "moons": [40009083], "asteroid_belts": []}
            ]
        }"#;
        let info: EsiSolarSystemInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.system_id, 30000142);
        assert_eq!(info.name, "Jita");
        assert!((info.security_status - 0.9459131).abs() < 0.0001);
        assert_eq!(info.stargates.len(), 2);
        assert_eq!(info.planets.len(), 1);
        assert_eq!(info.planets[0].planet_id, 40009082);
        assert_eq!(info.planets[0].moons, vec![40009083]);
    }

    #[test]
    fn test_deserialize_constellation_info() {
        let json = r#"{
            "constellation_id": 20000020,
            "name": "Kimotoro",
            "region_id": 10000002,
            "systems": [30000142, 30000143]
        }"#;
        let info: EsiConstellationInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.constellation_id, 20000020);
        assert_eq!(info.name, "Kimotoro");
        assert_eq!(info.systems.len(), 2);
    }

    #[test]
    fn test_deserialize_region_info() {
        let json = r#"{
            "region_id": 10000002,
            "name": "The Forge",
            "description": "Home of Jita",
            "constellations": [20000020, 20000021]
        }"#;
        let info: EsiRegionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.region_id, 10000002);
        assert_eq!(info.name, "The Forge");
        assert_eq!(info.constellations.len(), 2);
    }

    #[test]
    fn test_deserialize_station_info() {
        let json = r#"{
            "station_id": 60003760,
            "name": "Jita IV - Moon 4 - Caldari Navy Assembly Plant",
            "system_id": 30000142,
            "type_id": 52678,
            "owner": 1000035,
            "race_id": 1,
            "reprocessing_efficiency": 0.5,
            "reprocessing_stations_take": 0.05,
            "office_rental_cost": 1234567.89
        }"#;
        let info: EsiStationInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.station_id, 60003760);
        assert_eq!(info.system_id, 30000142);
        assert_eq!(info.owner, Some(1000035));
    }

    #[test]
    fn test_deserialize_stargate_info() {
        let json = r#"{
            "stargate_id": 50001248,
            "name": "Stargate (Perimeter)",
            "system_id": 30000142,
            "type_id": 29624,
            "destination": {"stargate_id": 50001249, "system_id": 30000144}
        }"#;
        let info: EsiStargateInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.stargate_id, 50001248);
        assert_eq!(info.destination.as_ref().unwrap().system_id, 30000144);
    }

    #[test]
    fn test_deserialize_resolved_ids() {
        let json = r#"{
            "characters": [{"id": 95465499, "name": "CCP Bartender"}],
            "systems": [{"id": 30000142, "name": "Jita"}]
        }"#;
        let resolved: EsiResolvedIds = serde_json::from_str(json).unwrap();
        assert_eq!(resolved.characters.len(), 1);
        assert_eq!(resolved.characters[0].id, 95465499);
        assert_eq!(resolved.systems.len(), 1);
        assert!(resolved.corporations.is_empty());
    }

    #[test]
    fn test_deserialize_market_group_info() {
        let json = r#"{
            "market_group_id": 61,
            "name": "Frigates",
            "description": "Small ships",
            "parent_group_id": 4,
            "types": [587, 603]
        }"#;
        let info: EsiMarketGroupInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.market_group_id, 61);
        assert_eq!(info.name, "Frigates");
        assert_eq!(info.parent_group_id, Some(4));
        assert_eq!(info.types.len(), 2);
    }

    #[test]
    fn test_deserialize_search_result() {
        let json = r#"{
            "solar_system": [30000142],
            "station": [60003760, 60003761]
        }"#;
        let result: EsiSearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.solar_system, vec![30000142]);
        assert_eq!(result.station.len(), 2);
        assert!(result.character.is_empty());
    }

    #[test]
    fn test_deserialize_killmail_ref() {
        let json = r#"{"killmail_id": 123456789, "killmail_hash": "abc123def456"}"#;
        let km: EsiKillmailRef = serde_json::from_str(json).unwrap();
        assert_eq!(km.killmail_id, 123456789);
        assert_eq!(km.killmail_hash, "abc123def456");
    }

    #[test]
    fn test_deserialize_sovereignty_map() {
        let json = r#"{"system_id": 30000001, "alliance_id": 99000001, "corporation_id": 98000001, "faction_id": null}"#;
        let entry: EsiSovereigntyMap = serde_json::from_str(json).unwrap();
        assert_eq!(entry.system_id, 30000001);
        assert_eq!(entry.alliance_id, Some(99000001));
        assert_eq!(entry.faction_id, None);
    }

    #[test]
    fn test_deserialize_sovereignty_campaign() {
        let json = r#"{"campaign_id": 1, "solar_system_id": 30000001, "structure_id": 1234567890, "event_type": "tcu_defense"}"#;
        let campaign: EsiSovereigntyCampaign = serde_json::from_str(json).unwrap();
        assert_eq!(campaign.campaign_id, 1);
        assert_eq!(campaign.event_type, Some("tcu_defense".to_string()));
    }

    #[test]
    fn test_deserialize_sovereignty_structure() {
        let json = r#"{"alliance_id": 99000001, "solar_system_id": 30000001, "structure_id": 1234567890, "structure_type_id": 32226}"#;
        let s: EsiSovereigntyStructure = serde_json::from_str(json).unwrap();
        assert_eq!(s.alliance_id, Some(99000001));
        assert_eq!(s.structure_type_id, 32226);
    }

    #[test]
    fn test_deserialize_incursion() {
        let json = r#"{
            "constellation_id": 20000020,
            "type": "Incursion",
            "state": "established",
            "staging_solar_system_id": 30000142,
            "influence": 0.5,
            "has_boss": true,
            "faction_id": 500019,
            "infested_solar_systems": [30000142, 30000143]
        }"#;
        let inc: EsiIncursion = serde_json::from_str(json).unwrap();
        assert_eq!(inc.constellation_id, 20000020);
        assert_eq!(inc.incursion_type, Some("Incursion".to_string()));
        assert_eq!(inc.state, Some("established".to_string()));
        assert!(inc.has_boss);
        assert_eq!(inc.infested_solar_systems.len(), 2);
    }

    #[test]
    fn test_deserialize_server_status() {
        let json = r#"{"players": 23456, "server_version": "2345678", "start_time": "2026-03-20T11:00:00Z", "vip": false}"#;
        let status: EsiServerStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.players, 23456);
        assert_eq!(status.server_version, Some("2345678".to_string()));
        assert_eq!(
            status.start_time,
            Some("2026-03-20T11:00:00Z".parse::<DateTime<Utc>>().unwrap())
        );
        assert_eq!(status.vip, Some(false));
    }

    #[test]
    fn test_deserialize_server_status_minimal() {
        let json = r#"{"players": 100}"#;
        let status: EsiServerStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.players, 100);
        assert_eq!(status.server_version, None);
        assert_eq!(status.vip, None);
    }

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Wallet
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Industry, Contracts, Orders
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Fittings, Location
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Mail, Notifications, Contacts
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Calendar, Clones, Loyalty, PI
    // -----------------------------------------------------------------------

    #[test]
    fn test_deserialize_calendar_event() {
        let json = r#"{
            "event_id": 99999,
            "event_date": "2026-03-20T19:00:00Z",
            "title": "Fleet Op",
            "importance": 0,
            "event_response": "accepted"
        }"#;
        let event: EsiCalendarEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_id, 99999);
        assert_eq!(event.title, "Fleet Op");
        assert_eq!(event.event_response, Some("accepted".to_string()));
    }

    #[test]
    fn test_deserialize_calendar_event_detail() {
        let json = r#"{
            "event_id": 99999,
            "date": "2026-03-20T19:00:00Z",
            "title": "Fleet Op",
            "owner_id": 98000001,
            "owner_name": "Test Corp",
            "owner_type": "corporation",
            "duration": 60,
            "text": "Bring your best ships"
        }"#;
        let detail: EsiCalendarEventDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.event_id, 99999);
        assert_eq!(detail.duration, 60);
        assert_eq!(detail.text, Some("Bring your best ships".to_string()));
    }

    #[test]
    fn test_deserialize_clones() {
        let json = r#"{
            "home_location": {"location_id": 60003760, "location_type": "station"},
            "jump_clones": [
                {"jump_clone_id": 1, "location_id": 60008494, "location_type": "station", "implants": [9899, 9941], "name": "Amarr clone"}
            ],
            "last_clone_jump_date": "2026-03-10T00:00:00Z"
        }"#;
        let clones: EsiClones = serde_json::from_str(json).unwrap();
        assert_eq!(clones.home_location.as_ref().unwrap().location_id, 60003760);
        assert_eq!(clones.jump_clones.len(), 1);
        assert_eq!(clones.jump_clones[0].implants, vec![9899, 9941]);
        assert_eq!(clones.jump_clones[0].name, Some("Amarr clone".to_string()));
    }

    #[test]
    fn test_deserialize_loyalty_points() {
        let json = r#"{"corporation_id": 1000035, "loyalty_points": 50000}"#;
        let lp: EsiLoyaltyPoints = serde_json::from_str(json).unwrap();
        assert_eq!(lp.corporation_id, 1000035);
        assert_eq!(lp.loyalty_points, 50000);
    }

    #[test]
    fn test_deserialize_loyalty_store_offer() {
        let json = r#"{
            "offer_id": 100,
            "type_id": 587,
            "quantity": 1,
            "lp_cost": 5000,
            "isk_cost": 1000000,
            "required_items": [{"type_id": 34, "quantity": 1000}]
        }"#;
        let offer: EsiLoyaltyStoreOffer = serde_json::from_str(json).unwrap();
        assert_eq!(offer.offer_id, 100);
        assert_eq!(offer.lp_cost, 5000);
        assert_eq!(offer.required_items.len(), 1);
        assert_eq!(offer.required_items[0].type_id, 34);
    }

    #[test]
    fn test_deserialize_planet_summary() {
        let json = r#"{
            "solar_system_id": 30000142,
            "planet_id": 40009082,
            "planet_type": "temperate",
            "num_pins": 5,
            "last_update": "2026-03-15T10:00:00Z",
            "upgrade_level": 4
        }"#;
        let planet: EsiPlanetSummary = serde_json::from_str(json).unwrap();
        assert_eq!(planet.planet_id, 40009082);
        assert_eq!(planet.planet_type, "temperate");
        assert_eq!(planet.num_pins, 5);
        assert_eq!(planet.upgrade_level, 4);
    }

    #[test]
    fn test_deserialize_planet_detail() {
        let json = r#"{
            "links": [{"source_pin_id": 1, "destination_pin_id": 2}],
            "pins": [{"pin_id": 1, "type_id": 2254}],
            "routes": []
        }"#;
        let detail: EsiPlanetDetail = serde_json::from_str(json).unwrap();
        assert_eq!(detail.links.len(), 1);
        assert_eq!(detail.pins.len(), 1);
        assert!(detail.routes.is_empty());
    }

    #[test]
    fn test_deserialize_mail_header() {
        let json = r#"{
            "mail_id": 123456,
            "timestamp": "2026-03-15T10:30:00Z",
            "from": 91234567,
            "subject": "Hello",
            "is_read": false,
            "labels": [1, 3],
            "recipients": [{"recipient_id": 92345678, "recipient_type": "character"}]
        }"#;
        let header: EsiMailHeader = serde_json::from_str(json).unwrap();
        assert_eq!(header.mail_id, 123456);
        assert_eq!(header.from, Some(91234567));
        assert_eq!(header.subject, Some("Hello".to_string()));
        assert_eq!(header.labels, vec![1, 3]);
        assert_eq!(header.recipients.len(), 1);
    }

    #[test]
    fn test_deserialize_mail_body() {
        let json = r#"{
            "body": "<p>Hello world</p>",
            "from": 91234567,
            "read": true,
            "subject": "Hello",
            "timestamp": "2026-03-15T10:30:00Z",
            "labels": [1],
            "recipients": [{"recipient_id": 92345678, "recipient_type": "character"}]
        }"#;
        let body: EsiMailBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.body, Some("<p>Hello world</p>".to_string()));
        assert_eq!(body.read, Some(true));
    }

    #[test]
    fn test_deserialize_mail_labels() {
        let json = r##"{
            "total_unread_count": 5,
            "labels": [{"label_id": 1, "name": "Inbox", "color": "#ffffff", "unread_count": 3}]
        }"##;
        let labels: EsiMailLabels = serde_json::from_str(json).unwrap();
        assert_eq!(labels.total_unread_count, 5);
        assert_eq!(labels.labels.len(), 1);
        assert_eq!(labels.labels[0].name, "Inbox");
    }

    #[test]
    fn test_deserialize_notification() {
        let json = r#"{
            "notification_id": 999888,
            "type": "StructureUnderAttack",
            "sender_id": 1000125,
            "sender_type": "corporation",
            "timestamp": "2026-03-15T10:30:00Z",
            "is_read": false,
            "text": "structureID: 1234567890"
        }"#;
        let notif: EsiNotification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.notification_id, 999888);
        assert_eq!(notif.notification_type, "StructureUnderAttack");
        assert_eq!(notif.sender_type, "corporation");
        assert_eq!(notif.is_read, Some(false));
    }

    #[test]
    fn test_deserialize_contact() {
        let json = r#"{
            "contact_id": 91234567,
            "contact_type": "character",
            "standing": 10.0,
            "label_ids": [1, 2],
            "is_watched": true
        }"#;
        let contact: EsiContact = serde_json::from_str(json).unwrap();
        assert_eq!(contact.contact_id, 91234567);
        assert_eq!(contact.contact_type, "character");
        assert!((contact.standing - 10.0).abs() < f64::EPSILON);
        assert_eq!(contact.label_ids, vec![1, 2]);
        assert_eq!(contact.is_watched, Some(true));
    }

    #[test]
    fn test_deserialize_contact_label() {
        let json = r#"{"label_id": 1, "label_name": "Blues"}"#;
        let label: EsiContactLabel = serde_json::from_str(json).unwrap();
        assert_eq!(label.label_id, 1);
        assert_eq!(label.label_name, "Blues");
    }

    #[test]
    fn test_deserialize_fitting() {
        let json = r#"{
            "fitting_id": 12345,
            "name": "PvP Rifter",
            "description": "Standard PvP fit",
            "ship_type_id": 587,
            "items": [
                {"type_id": 2032, "flag": 11, "quantity": 1},
                {"type_id": 3170, "flag": 12, "quantity": 1}
            ]
        }"#;
        let fit: EsiFitting = serde_json::from_str(json).unwrap();
        assert_eq!(fit.fitting_id, 12345);
        assert_eq!(fit.name, "PvP Rifter");
        assert_eq!(fit.ship_type_id, 587);
        assert_eq!(fit.items.len(), 2);
        assert_eq!(fit.items[0].type_id, 2032);
    }

    #[test]
    fn test_deserialize_location() {
        let json = r#"{"solar_system_id": 30000142, "station_id": 60003760}"#;
        let loc: EsiLocation = serde_json::from_str(json).unwrap();
        assert_eq!(loc.solar_system_id, 30000142);
        assert_eq!(loc.station_id, Some(60003760));
        assert_eq!(loc.structure_id, None);
    }

    #[test]
    fn test_deserialize_ship() {
        let json = r#"{"ship_type_id": 587, "ship_item_id": 1234567890, "ship_name": "My Rifter"}"#;
        let ship: EsiShip = serde_json::from_str(json).unwrap();
        assert_eq!(ship.ship_type_id, 587);
        assert_eq!(ship.ship_name, "My Rifter");
    }

    #[test]
    fn test_deserialize_online_status() {
        let json = r#"{
            "online": true,
            "last_login": "2026-03-20T10:00:00Z",
            "last_logout": "2026-03-19T22:00:00Z",
            "logins": 500
        }"#;
        let status: EsiOnlineStatus = serde_json::from_str(json).unwrap();
        assert!(status.online);
        assert!(status.last_login.is_some());
        assert_eq!(status.logins, Some(500));
    }

    #[test]
    fn test_deserialize_industry_job() {
        let json = r#"{
            "job_id": 123,
            "installer_id": 91234567,
            "facility_id": 60003760,
            "activity_id": 1,
            "blueprint_id": 1234567890,
            "blueprint_type_id": 687,
            "blueprint_location_id": 60003760,
            "output_location_id": 60003760,
            "runs": 10,
            "status": "active",
            "duration": 3600,
            "start_date": "2026-03-15T10:00:00Z",
            "end_date": "2026-03-15T11:00:00Z",
            "cost": 1500.50,
            "product_type_id": 687
        }"#;
        let job: EsiIndustryJob = serde_json::from_str(json).unwrap();
        assert_eq!(job.job_id, 123);
        assert_eq!(job.activity_id, 1);
        assert_eq!(job.status, "active");
        assert_eq!(job.runs, 10);
        assert!((job.cost.unwrap() - 1500.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_blueprint() {
        let json = r#"{
            "item_id": 1234567890,
            "type_id": 687,
            "location_id": 60003760,
            "location_flag": "Hangar",
            "quantity": -2,
            "time_efficiency": 20,
            "material_efficiency": 10,
            "runs": 100
        }"#;
        let bp: EsiBlueprint = serde_json::from_str(json).unwrap();
        assert_eq!(bp.item_id, 1234567890);
        assert_eq!(bp.type_id, 687);
        assert_eq!(bp.quantity, -2);
        assert_eq!(bp.time_efficiency, 20);
        assert_eq!(bp.material_efficiency, 10);
    }

    #[test]
    fn test_deserialize_contract() {
        let json = r#"{
            "contract_id": 123456,
            "issuer_id": 91234567,
            "issuer_corporation_id": 98000001,
            "type": "item_exchange",
            "status": "outstanding",
            "availability": "personal",
            "date_issued": "2026-03-15T10:00:00Z",
            "date_expired": "2026-03-29T10:00:00Z",
            "for_corporation": false,
            "title": "Selling stuff",
            "price": 1000000.0,
            "start_location_id": 60003760,
            "end_location_id": 60003760
        }"#;
        let c: EsiContract = serde_json::from_str(json).unwrap();
        assert_eq!(c.contract_id, 123456);
        assert_eq!(c.contract_type, "item_exchange");
        assert_eq!(c.status, "outstanding");
        assert!(!c.for_corporation);
        assert_eq!(c.title, Some("Selling stuff".to_string()));
        assert!((c.price.unwrap() - 1000000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_contract_item() {
        let json = r#"{
            "record_id": 999,
            "type_id": 34,
            "quantity": 100000,
            "is_included": true,
            "is_singleton": false
        }"#;
        let item: EsiContractItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.record_id, 999);
        assert_eq!(item.type_id, 34);
        assert_eq!(item.quantity, 100000);
        assert!(item.is_included);
    }

    #[test]
    fn test_deserialize_contract_bid() {
        let json = r#"{
            "bid_id": 555,
            "bidder_id": 91234567,
            "date_bid": "2026-03-16T12:00:00Z",
            "amount": 5000000.0
        }"#;
        let bid: EsiContractBid = serde_json::from_str(json).unwrap();
        assert_eq!(bid.bid_id, 555);
        assert!((bid.amount - 5000000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_character_order() {
        let json = r#"{
            "order_id": 6789012345,
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
            "duration": 90,
            "escrow": 1282500.0,
            "is_corporation": false
        }"#;
        let order: EsiCharacterOrder = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_id, 6789012345);
        assert!(order.is_buy_order);
        assert_eq!(order.volume_total, 500000);
        assert_eq!(order.volume_remain, 250000);
        assert_eq!(order.state, None);
        assert!((order.escrow.unwrap() - 1282500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_wallet_journal_entry_full() {
        let json = r#"{
            "id": 123456789,
            "date": "2026-03-15T10:30:00Z",
            "ref_type": "market_transaction",
            "amount": -1500000.50,
            "balance": 98500000.00,
            "description": "Market: Tritanium",
            "first_party_id": 91234567,
            "second_party_id": 92345678,
            "reason": "For the lulz",
            "context_id": 6789012345,
            "context_id_type": "market_transaction_id",
            "tax": 15000.00,
            "tax_receiver_id": 1000035
        }"#;
        let entry: EsiWalletJournalEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, 123456789);
        assert_eq!(entry.ref_type, "market_transaction");
        assert!((entry.amount.unwrap() - (-1500000.50)).abs() < f64::EPSILON);
        assert!((entry.balance.unwrap() - 98500000.00).abs() < f64::EPSILON);
        assert_eq!(entry.description, Some("Market: Tritanium".to_string()));
        assert_eq!(entry.first_party_id, Some(91234567));
        assert_eq!(entry.second_party_id, Some(92345678));
        assert_eq!(
            entry.context_id_type,
            Some("market_transaction_id".to_string())
        );
        assert_eq!(entry.tax_receiver_id, Some(1000035));
    }

    #[test]
    fn test_deserialize_wallet_journal_entry_minimal() {
        let json = r#"{
            "id": 999,
            "date": "2026-01-01T00:00:00Z",
            "ref_type": "player_donation"
        }"#;
        let entry: EsiWalletJournalEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, 999);
        assert_eq!(entry.ref_type, "player_donation");
        assert_eq!(entry.amount, None);
        assert_eq!(entry.description, None);
    }

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Skills
    // -----------------------------------------------------------------------

    #[test]
    fn test_deserialize_skills() {
        let json = r#"{
            "skills": [
                {"skill_id": 3300, "trained_skill_level": 5, "active_skill_level": 5, "skillpoints_in_skill": 256000}
            ],
            "total_sp": 50000000,
            "unallocated_sp": 100000
        }"#;
        let skills: EsiSkills = serde_json::from_str(json).unwrap();
        assert_eq!(skills.total_sp, 50000000);
        assert_eq!(skills.unallocated_sp, Some(100000));
        assert_eq!(skills.skills.len(), 1);
        assert_eq!(skills.skills[0].skill_id, 3300);
        assert_eq!(skills.skills[0].trained_skill_level, 5);
    }

    #[test]
    fn test_deserialize_skillqueue_entry() {
        let json = r#"{
            "skill_id": 3300,
            "finish_level": 5,
            "queue_position": 0,
            "start_date": "2026-03-15T10:00:00Z",
            "finish_date": "2026-03-20T10:00:00Z",
            "training_start_sp": 45255,
            "level_start_sp": 45255,
            "level_end_sp": 256000
        }"#;
        let entry: EsiSkillqueueEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.skill_id, 3300);
        assert_eq!(entry.finish_level, 5);
        assert_eq!(entry.queue_position, 0);
        assert!(entry.start_date.is_some());
        assert_eq!(entry.level_end_sp, Some(256000));
    }

    #[test]
    fn test_deserialize_attributes() {
        let json = r#"{
            "intelligence": 20,
            "memory": 20,
            "perception": 20,
            "willpower": 20,
            "charisma": 19,
            "bonus_remaps": 1,
            "last_remap_date": "2025-01-01T00:00:00Z"
        }"#;
        let attrs: EsiAttributes = serde_json::from_str(json).unwrap();
        assert_eq!(attrs.intelligence, 20);
        assert_eq!(attrs.charisma, 19);
        assert_eq!(attrs.bonus_remaps, Some(1));
        assert!(attrs.last_remap_date.is_some());
        assert_eq!(attrs.accrued_remap_cooldown_date, None);
    }

    #[test]
    fn test_deserialize_wallet_transaction() {
        let json = r#"{
            "transaction_id": 5678901234,
            "date": "2026-03-15T10:30:00Z",
            "type_id": 34,
            "location_id": 60003760,
            "unit_price": 5.25,
            "quantity": 100000,
            "client_id": 91234567,
            "is_buy": true,
            "is_personal": true,
            "journal_ref_id": 123456789
        }"#;
        let tx: EsiWalletTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.transaction_id, 5678901234);
        assert_eq!(tx.type_id, 34);
        assert_eq!(tx.location_id, JITA_STATION);
        assert!((tx.unit_price - 5.25).abs() < f64::EPSILON);
        assert_eq!(tx.quantity, 100000);
        assert!(tx.is_buy);
        assert!(tx.is_personal);
    }
}
