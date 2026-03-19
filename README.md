# nea-esi

Rate-limited Rust client for EVE Online's [ESI](https://esi.evetech.net/ui/) (EVE Swagger Interface) API. This library handles concurrency, pagination, and ESI's error budget system so callers don't have to.

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

## API reference

### Client construction

| Constructor | Description |
|---|---|
| `EsiClient::new()` | Default user agent, 30s timeout |
| `EsiClient::with_user_agent(ua)` | Custom user agent, 30s timeout |
| `EsiClient::default()` | Same as `new()` |

ESI requires a descriptive User-Agent. Format: `app-name (contact; +repo_url; eve:CharacterName)`.

### Methods

All methods are `async` and return `nea_esi::Result<T>`.

**Market data:**

```rust
// Daily OHLCV data for a type in a region
client.market_history(region_id: i32, type_id: i32) -> Vec<EsiMarketHistoryEntry>

// All orders for a type in a region (handles pagination automatically)
client.market_orders(region_id: i32, type_id: i32) -> Vec<EsiMarketOrder>

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
```

**Low-level:**

```rust
// Rate-limited GET to any ESI URL — use for endpoints not wrapped above
client.request(url: &str) -> reqwest::Response

// Current error budget (starts at 100, updated from X-ESI-Error-Limit-Remain header)
client.error_budget() -> i32
```

### Constants

| Constant | Value | Notes |
|---|---|---|
| `BASE_URL` | `https://esi.evetech.net/latest` | All endpoints built from this |
| `THE_FORGE` | `10000002` | Region ID — Jita's region |
| `JITA_STATION` | `60003760` | Station ID — Jita 4-4 CNAP |
| `DEFAULT_USER_AGENT` | `nea-esi (https://github.com/...)` | Used by `EsiClient::new()` |

### Response types

**`EsiMarketHistoryEntry`** — `date: String`, `average: f64`, `highest: f64`, `lowest: f64`, `volume: i64`, `order_count: i64`

**`EsiMarketOrder`** — `order_id: i64`, `type_id: i32`, `location_id: i64`, `price: f64`, `volume_remain: i64`, `is_buy_order: bool`, `issued: String`, `duration: i32`, `min_volume: i32`, `range: String`

**`EsiKillmail`** — `killmail_id: i64`, `killmail_time: String`, `solar_system_id: i32`, `victim: EsiKillmailVictim`, `attackers: Vec<EsiKillmailAttacker>`

**`EsiKillmailVictim`** — `ship_type_id: i32`, `character_id: Option<i64>`, `corporation_id: Option<i64>`, `alliance_id: Option<i64>`, `items: Vec<EsiKillmailItem>`

**`EsiKillmailAttacker`** — `character_id: Option<i64>`, `corporation_id: Option<i64>`, `alliance_id: Option<i64>`, `ship_type_id: i32`, `weapon_type_id: i32`, `damage_done: i32`, `final_blow: bool`

**`EsiKillmailItem`** — `item_type_id: i32`, `quantity_destroyed: Option<i64>`, `quantity_dropped: Option<i64>`, `flag: i32`, `singleton: i32`

**`EsiCharacterInfo`** — `name: String`, `corporation_id: Option<i64>`, `alliance_id: Option<i64>`

**`EsiCorporationInfo`** — `name: String`, `alliance_id: Option<i64>`, `member_count: Option<i32>`

**`EsiAllianceInfo`** — `name: String`, `ticker: Option<String>`

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
}
```

## Rate limiting behavior

The client manages ESI's error budget automatically:

1. **Concurrency cap**: Max 20 in-flight requests (tokio semaphore).
2. **Budget tracking**: After each response, `X-ESI-Error-Limit-Remain` header updates the internal budget (starts at 100).
3. **Low budget backoff**: When budget drops below 20, each request sleeps 1s before firing.
4. **Budget exhausted**: When budget hits 0, requests immediately return `EsiError::RateLimited` without making a network call.

Check the budget at any time with `client.error_budget()`. The budget is shared across clones of the client's internal `Arc<AtomicI32>` — concurrent tasks spawned by `market_orders` pagination all share the same budget.

## Pagination

`market_orders()` handles pagination transparently. It reads the `x-pages` header from the first response, then fetches remaining pages concurrently via `tokio::spawn`. All pages share the same rate limiter.

## Logging

The crate uses `tracing` for structured logging:
- `debug!` on every successful request (URL, status, elapsed_ms, error_budget)
- `warn!` on low budget backoff and API errors

Wire up a `tracing_subscriber` in the consuming binary to see these.

## Common type IDs and region IDs

This library doesn't bundle a type/region database. Look up IDs at the [ESI Swagger UI](https://esi.evetech.net/ui/) or use the search endpoints. Some commonly used ones:

- **Tritanium**: type_id `34`
- **PLEX**: type_id `44992`
- **The Forge** (Jita's region): region_id `10000002` (`THE_FORGE` constant)
- **Domain** (Amarr's region): region_id `10000043`
- **Jita 4-4**: station_id `60003760` (`JITA_STATION` constant)
- **Amarr**: station_id `60008494`

## License

MIT
