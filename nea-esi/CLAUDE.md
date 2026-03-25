# nea-esi (library)

Async, rate-limited Rust client for EVE Online's ESI API with full OAuth2 PKCE support.

## Build & Test

```bash
cargo test -p nea-esi            # All library tests
cargo test -p nea-esi --lib      # Unit tests only
cargo test -p nea-esi --test auth_integration
cargo test -p nea-esi --test endpoints_integration
```

Tests use `wiremock` for HTTP mocking. No live ESI calls or credentials needed.

## Module Layout

- **`lib.rs`** (~1700 lines) - Core client and all public types:
  - `EsiClient` - Main client struct with rate limiting, caching, token management
  - `EsiError` - Error enum via `thiserror`
  - `EsiTokens` - Token pair with expiration tracking
  - `EsiAppCredentials` - Web (confidential) or Native (public) app credentials
  - `PkceChallenge` - OAuth flow state (authorize URL, code verifier, state)
  - 100+ response structs (`EsiMarketOrder`, `EsiCharacterInfo`, `EsiKillmail`, etc.)
  - Constants: `BASE_URL`, region IDs (`THE_FORGE`, etc.), station IDs (`JITA_STATION`, etc.)

- **`auth.rs`** - OAuth2 PKCE implementation (RFC 7636):
  - SSO endpoints: `login.eveonline.com/v2/oauth/authorize` and `/token`
  - PKCE: 96-byte verifier -> 128 base64url chars, S256 challenge, 32-byte state
  - Native apps: client_id in body. Web apps: Basic Auth with client_id:secret
  - Token exchange, refresh (mutex-serialized to prevent storms), auto-refresh 60s before expiry

- **`endpoints/mod.rs`** - HTTP helpers and pagination:
  - `get_json<T>()`, `post_json<T>()`, `post_json_void()`, `put_json()`, `delete_path()`
  - `get_paginated_json<T>()`, `post_paginated<T, B>()` - parallel page fetching
  - `build_url()`, `build_contact_url()` - URL construction with query params
  - Chunking constants: `RESOLVE_NAMES_CHUNK_SIZE=1000`, `ASSET_NAMES_CHUNK_SIZE=1000`

- **`endpoints/*.rs`** - Domain endpoints (10 modules):
  - `character.rs` - Info, assets, wallet, skills, roles, clones, implants, medals, etc.
  - `character_social.rs` - Mail, bookmarks, calendar, contacts, search
  - `corporation.rs` - Info, members, structures, starbases, divisions, etc.
  - `market.rs` - Orders, history, prices, types, groups + `compute_best_bid_ask()`
  - `universe.rs` - Types, systems, regions, stations, ID resolution (with chunking)
  - `alliance.rs`, `fleet.rs`, `killmail.rs`, `misc.rs`

## Key Design Decisions

**Rate Limiting** - Error budget system (not request counting):
- Tracks ESI's `X-ESI-Error-Limit-Remain` / `X-ESI-Error-Limit-Reset` headers
- Budget starts at 100, decrements on API errors
- Proactive sleep when budget < 20 until ESI reset window
- Semaphore caps at 20 concurrent requests
- Returns `EsiError::RateLimited` at budget <= 0

**Retry** (MAX_RETRIES=3):
- Transient errors (502-504, timeouts): exponential backoff + jitter (`1000ms * 2^attempt + rand(0-500ms)`)
- 401: auto-refresh token, retry once
- All other errors: fail immediately

**Pagination**:
- Fetch page 1, read `x-pages` header, spawn parallel fetches for pages 2+
- Results flattened into single `Vec<T>`

**Token Safety**:
- `SecretString` wraps all credentials (redacted in Debug output)
- Dedicated mutex serializes refresh operations, preventing concurrent refresh storms
- Double-check pattern: after acquiring mutex, re-checks if token was already refreshed

## Adding a New Endpoint

1. Add response struct(s) in `lib.rs` with `#[derive(Debug, Clone, Serialize, Deserialize)]`
2. Add the method in the appropriate `endpoints/*.rs` file as `impl EsiClient`
3. Use `#[tracing::instrument(skip(self))]` on the method
4. Use `self.get_json()`, `self.post_json()`, or paginated variants from `endpoints/mod.rs`
5. Add wiremock-based test in `tests/endpoints_integration.rs`
