# nea-esi

Rate-limited Rust client for EVE Online's [ESI](https://esi.evetech.net/ui/) (EVE Swagger Interface) API. This library handles concurrency, pagination, retry, ETag caching, OAuth/SSO, and ESI's error budget system so callers don't have to.

## Adding to a project

```toml
[dependencies]
nea-esi = { path = "../nea-esi" }  # or wherever this lives relative to the consuming crate
tokio = { version = "1", features = ["full"] }
```

The crate re-exports everything from `lib.rs` — there are no feature flags.

## Quick start

```rust
use nea_esi::{EsiClient, THE_FORGE, JITA_STATION};

#[tokio::main]
async fn main() -> nea_esi::Result<()> {
    let client = EsiClient::with_user_agent(
        "my-app (contact@example.com; +https://github.com/me/my-app; eve:MyCharacter)",
    );

    // Market history for Tritanium (type_id 34) in The Forge
    let history = client.market_history(THE_FORGE, 34).await?;
    println!("Got {} daily entries", history.len());

    // Market orders + best bid/ask at Jita 4-4
    let orders = client.market_orders(THE_FORGE, 34).await?;
    let (bid, ask, bid_vol, ask_vol) = EsiClient::compute_best_bid_ask(&orders, JITA_STATION);
    println!("Bid: {:?} (vol {}), Ask: {:?} (vol {})", bid, bid_vol, ask, ask_vol);

    Ok(())
}
```

## OAuth / SSO

The client supports EVE SSO with PKCE for both web and native apps.

```rust
use nea_esi::EsiClient;
use secrecy::SecretString;

// Native (public) app — no client secret needed
let client = EsiClient::with_native_app("my-app", "your-client-id");

// Web (confidential) app
let client = EsiClient::with_web_app("my-app", "your-client-id", secret);

// 1. Generate authorize URL
let challenge = client.authorize_url(
    "http://localhost:8080/callback",
    &["esi-wallet.read_character_wallet.v1"],
)?;
println!("Open in browser: {}", challenge.authorize_url);

// 2. Exchange the code after callback
let tokens = client.exchange_code(&code, &challenge.code_verifier, redirect_uri).await?;

// 3. Authenticated endpoints now work automatically
let assets = client.character_assets(character_id).await?;
```

Token management methods: `set_tokens()`, `get_tokens()`, `clear_tokens()`, `refresh_token()`. The client auto-refreshes tokens when they're within 60 seconds of expiry and retries once on 401.

## API reference

### Client construction

| Constructor | Description |
|---|---|
| `EsiClient::new()` | Default user agent, 30s timeout |
| `EsiClient::with_user_agent(ua)` | Custom user agent, 30s timeout |
| `EsiClient::with_native_app(ua, client_id)` | Native app with PKCE (public client) |
| `EsiClient::with_web_app(ua, client_id, secret)` | Web app with PKCE (confidential client) |
| `EsiClient::default()` | Same as `new()` |
| `.credentials(creds)` | Set app credentials (builder pattern) |
| `.with_cache()` | Enable ETag response caching (builder pattern) |

ESI requires a descriptive User-Agent. Format: `app-name (contact; +repo_url; eve:CharacterName)`.

### Methods

All methods are `async` and return `nea_esi::Result<T>`.

**Market data:**

```rust
// Daily OHLCV data for a type in a region
client.market_history(region_id: i32, type_id: i32) -> Vec<EsiMarketHistoryEntry>

// All orders for a type in a region (handles pagination automatically)
client.market_orders(region_id: i32, type_id: i32) -> Vec<EsiMarketOrder>

// Global average and adjusted prices for all types
client.market_prices() -> Vec<EsiMarketPrice>

// Filter orders to a station, return (best_bid, best_ask, bid_volume, ask_volume)
// This is a static method — no &self, no async
EsiClient::compute_best_bid_ask(orders: &[EsiMarketOrder], station_id: i64)
    -> (Option<f64>, Option<f64>, i64, i64)
```

**Killmails:**

```rust
// Raw JSON — useful when you need fields not in the typed struct
client.get_killmail(killmail_id: i64, killmail_hash: &str) -> serde_json::Value

// Typed — parses into EsiKillmail with victim, attackers, items
client.get_killmail_typed(killmail_id: i64, killmail_hash: &str) -> EsiKillmail
```

**Entity lookups:**

```rust
client.get_character(character_id: i64) -> EsiCharacterInfo     // name, corp, alliance
client.get_corporation(corporation_id: i64) -> EsiCorporationInfo // name, alliance, member_count
client.get_alliance(alliance_id: i64) -> EsiAllianceInfo         // name, ticker
client.get_structure(structure_id: i64) -> EsiStructureInfo      // name, owner, system (authenticated)
```

**Universe / names:**

```rust
// Resolve IDs to names and categories (auto-chunks at 1000 per request)
client.resolve_names(ids: &[i64]) -> Vec<EsiResolvedName>
```

**Assets (authenticated):**

```rust
// All assets for a character (handles pagination automatically)
client.character_assets(character_id: i64) -> Vec<EsiAssetItem>
```

**Generic pagination helpers:**

```rust
// Paginated GET — fetches page 1, reads x-pages, spawns concurrent tasks for the rest
client.get_paginated::<T>(base_url: &str) -> Vec<T>

// Paginated POST — same pattern, serializes body once and clones into each task
client.post_paginated::<T, B>(base_url: &str, body: &B) -> Vec<T>
```

**Low-level:**

```rust
// Rate-limited GET with retry on 502/503/504
client.request(url: &str) -> reqwest::Response

// Rate-limited POST with retry on 502/503/504
client.request_post(url: &str, body: &impl Serialize) -> reqwest::Response

// GET with ETag caching (returns raw bytes; requires .with_cache())
client.request_cached(url: &str) -> Vec<u8>

// Current error budget (starts at 100, updated from X-ESI-Error-Limit-Remain header)
client.error_budget() -> i32

// Clear cached ETag responses
client.clear_cache()
```

### Constants

| Constant | Value | Notes |
|---|---|---|
| `BASE_URL` | `https://esi.evetech.net/latest` | All endpoints built from this |
| `THE_FORGE` | `10000002` | Region ID — Jita's region |
| `DOMAIN` | `10000043` | Region ID — Amarr's region |
| `SINQ_LAISON` | `10000032` | Region ID — Dodixie's region |
| `HEIMATAR` | `10000030` | Region ID — Rens's region |
| `METROPOLIS` | `10000042` | Region ID — Hek's region |
| `JITA_STATION` | `60003760` | Station ID — Jita 4-4 CNAP |
| `AMARR_STATION` | `60008494` | Station ID — Amarr VIII |
| `DODIXIE_STATION` | `60011866` | Station ID — Dodixie IX |
| `RENS_STATION` | `60004588` | Station ID — Rens VI |
| `HEK_STATION` | `60005686` | Station ID — Hek VIII |
| `DEFAULT_USER_AGENT` | `nea-esi (https://github.com/...)` | Used by `EsiClient::new()` |

### Response types

**`EsiMarketHistoryEntry`** — `date: String`, `average: f64`, `highest: f64`, `lowest: f64`, `volume: i64`, `order_count: i64`

**`EsiMarketOrder`** — `order_id: i64`, `type_id: i32`, `location_id: i64`, `price: f64`, `volume_remain: i64`, `is_buy_order: bool`, `issued: String`, `duration: i32`, `min_volume: i32`, `range: String`

**`EsiMarketPrice`** — `type_id: i32`, `average_price: Option<f64>`, `adjusted_price: Option<f64>`

**`EsiKillmail`** — `killmail_id: i64`, `killmail_time: String`, `solar_system_id: i32`, `victim: EsiKillmailVictim`, `attackers: Vec<EsiKillmailAttacker>`

**`EsiKillmailVictim`** — `ship_type_id: i32`, `character_id: Option<i64>`, `corporation_id: Option<i64>`, `alliance_id: Option<i64>`, `items: Vec<EsiKillmailItem>`

**`EsiKillmailAttacker`** — `character_id: Option<i64>`, `corporation_id: Option<i64>`, `alliance_id: Option<i64>`, `ship_type_id: i32`, `weapon_type_id: i32`, `damage_done: i32`, `final_blow: bool`

**`EsiKillmailItem`** — `item_type_id: i32`, `quantity_destroyed: Option<i64>`, `quantity_dropped: Option<i64>`, `flag: i32`, `singleton: i32`

**`EsiCharacterInfo`** — `name: String`, `corporation_id: Option<i64>`, `alliance_id: Option<i64>`

**`EsiCorporationInfo`** — `name: String`, `alliance_id: Option<i64>`, `member_count: Option<i32>`

**`EsiAllianceInfo`** — `name: String`, `ticker: Option<String>`

**`EsiAssetItem`** — `item_id: i64`, `type_id: i32`, `location_id: i64`, `location_type: String`, `location_flag: String`, `quantity: i32`, `is_singleton: bool`, `is_blueprint_copy: Option<bool>`

**`EsiResolvedName`** — `id: i64`, `name: String`, `category: String`

**`EsiStructureInfo`** — `name: String`, `owner_id: i64`, `solar_system_id: i32`, `type_id: Option<i32>`

**`EsiTokens`** — `access_token: SecretString`, `refresh_token: SecretString`, `expires_at: DateTime<Utc>`

### Error handling

```rust
use nea_esi::EsiError;

match client.market_history(THE_FORGE, 34).await {
    Ok(entries) => { /* use entries */ }
    Err(EsiError::RateLimited) => { /* budget exhausted, wait and retry */ }
    Err(EsiError::Api { status, message }) => { /* ESI returned an error status */ }
    Err(EsiError::Http(e)) => { /* network/connection error */ }
    Err(EsiError::Deserialize(msg)) => { /* response didn't match expected shape */ }
    Err(EsiError::Internal(msg)) => { /* semaphore closed or similar */ }
    Err(EsiError::Auth(msg)) => { /* missing credentials or SSO config error */ }
    Err(EsiError::TokenRefresh(msg)) => { /* token exchange/refresh failed */ }
}
```

## Rate limiting behavior

The client manages ESI's error budget automatically:

1. **Concurrency cap**: Max 20 in-flight requests (tokio semaphore).
2. **Budget tracking**: After each response, `X-ESI-Error-Limit-Remain` header updates the internal budget (starts at 100).
3. **Smart backoff**: When budget drops below 20, each request sleeps until the `X-ESI-Error-Limit-Reset` window (falls back to 60s if no reset header received).
4. **Budget exhausted**: When budget hits 0, requests immediately return `EsiError::RateLimited` without making a network call.
5. **Transient retry**: 502, 503, and 504 responses are retried up to 3 times with exponential backoff (1s base, doubled each attempt, plus random 0–500ms jitter).

Check the budget at any time with `client.error_budget()`. The budget is shared across clones of the client's internal `Arc<AtomicI32>` — concurrent tasks spawned by pagination all share the same budget.

## Pagination

`market_orders()`, `character_assets()`, and any endpoint using `get_paginated()` / `post_paginated()` handle pagination transparently. They read the `x-pages` header from the first response, then fetch remaining pages concurrently via `tokio::spawn`. All pages share the same rate limiter.

## ETag caching

Enable with `.with_cache()`:

```rust
let client = EsiClient::with_user_agent("my-app").with_cache();

// First call: normal GET, caches response + ETag
let bytes = client.request_cached(&url).await?;

// Subsequent calls: sends If-None-Match, returns cached body on 304
let bytes = client.request_cached(&url).await?;

// Free memory when needed
client.clear_cache();
```

Best for endpoints that change infrequently: `market_prices`, `get_character`, `get_corporation`, `get_alliance`, etc. Not used automatically for paginated endpoints.

## Logging

The crate uses `tracing` for structured logging:
- `debug!` on every successful request (URL, status, elapsed_ms, error_budget)
- `warn!` on low budget backoff, API errors, and transient retries

Wire up a `tracing_subscriber` in the consuming binary to see these.

## Common type IDs

This library doesn't bundle a type/region database. Look up IDs at the [ESI Swagger UI](https://esi.evetech.net/ui/) or use the search endpoints. Some commonly used ones:

- **Tritanium**: type_id `34`
- **PLEX**: type_id `44992`

Region and station IDs are available as constants (see table above).

## License

MIT
