# nea-esi

Rate-limited Rust client for EVE Online's [ESI](https://esi.evetech.net/ui/) (EVE Swagger Interface) API.

## Features

- Market history (daily OHLCV per region/type)
- Market orders (paginated, with best bid/ask computation)
- Killmails (raw JSON or typed structs)
- Character, corporation, and alliance lookups
- Built-in rate limiting: semaphore-based concurrency control + ESI error budget feedback loop

## Usage

```rust
use nea_esi::EsiClient;

let client = EsiClient::with_user_agent(
    "my-app (contact@example.com; +https://github.com/me/my-app; eve:MyCharacter)",
);

// Fetch market history for Tritanium in The Forge
let history = client.market_history(nea_esi::THE_FORGE, 34).await?;

// Fetch market orders and compute best bid/ask at Jita 4-4
let orders = client.market_orders(nea_esi::THE_FORGE, 34).await?;
let (bid, ask, bid_vol, ask_vol) =
    EsiClient::compute_best_bid_ask(&orders, nea_esi::JITA_STATION);
```

## Rate Limiting

The client respects ESI's `X-ESI-Error-Limit-Remain` header:

- Max 20 concurrent requests (tokio semaphore)
- Budget < 20 remaining errors: 1s delay before each request
- Budget exhausted: requests return `EsiError::RateLimited`

## License

MIT
