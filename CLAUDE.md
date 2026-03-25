# nea-esi Workspace

Rate-limited Rust client library and interactive CLI for EVE Online's ESI (EVE Swagger Interface) API.

## Workspace Structure

```
nea-esi/           # Workspace root
├── nea-esi/       # Library crate (v0.7.0) - async ESI client with OAuth2 PKCE, rate limiting, pagination
└── nea-esi-cli/   # Binary crate (v0.1.0) - interactive CLI with REPL, multiple output formats
```

## Build & Test

```bash
cargo build --workspace          # Build everything
cargo test --workspace           # Run all tests
cargo test -p nea-esi            # Library tests only
cargo test -p nea-esi-cli        # CLI tests only
cargo build -p nea-esi-cli       # Build CLI binary
cargo run -p nea-esi-cli -- ...  # Run CLI with args
```

- Rust edition 2024, resolver v3
- Tests use `wiremock` for HTTP mocking - no live ESI calls needed
- No feature flags; everything enabled by default

## Architecture Overview

### Library (nea-esi)

- **`lib.rs`**: Core `EsiClient` struct, all response types (100+), error types, constants (region/station IDs)
- **`auth.rs`**: Full OAuth2 PKCE flow (RFC 7636) - supports both native (public) and web (confidential) app credentials
- **`endpoints/mod.rs`**: HTTP helpers (`get_json`, `post_json`, `get_paginated_json`, etc.), chunking constants
- **`endpoints/*.rs`**: Domain-specific endpoint implementations as `impl EsiClient` blocks across 10 modules (character, corporation, market, universe, alliance, fleet, killmail, misc, character_social)

Key design patterns:
- **Rate limiting**: ESI error budget tracking (100 points, header-driven reset) + semaphore (max 20 concurrent requests) + exponential backoff with jitter on transient errors (502-504)
- **Token management**: Auto-refresh 60s before expiry, mutex-serialized refresh prevents concurrent storms, auto-retry on 401
- **Pagination**: Fetches page 1, reads `x-pages` header, parallelizes remaining pages with `tokio::spawn`
- **ETag caching**: Optional response cache via `.with_cache()`, serves cached body on 304
- **Security**: `SecretString` wraps credentials, PKCE + state parameter for OAuth

### CLI (nea-esi-cli)

- **`main.rs`**: Entry point, config loading, client construction, command dispatch
- **`cli.rs`**: clap derive command/subcommand definitions
- **`auth.rs`**: Browser-based SSO login flow with local HTTP callback server, scope management
- **`config.rs`**: `~/.config/nea-esi/config.toml` loading/saving (platform-specific via `directories` crate)
- **`token_store.rs`**: Token persistence to `tokens.json`
- **`output.rs`**: JSON/table/CSV output formatting with TTY auto-detection
- **`repl.rs`**: Interactive REPL with rustyline, tab-completion, history
- **`commands/`**: 31 command modules covering all ESI domains

## Conventions

- All endpoint methods are `async`, return `Result<T, EsiError>`, and use `#[tracing::instrument]`
- Response types are named `Esi{Domain}{Thing}` (e.g., `EsiMarketOrder`, `EsiCharacterInfo`)
- Endpoint modules extend `EsiClient` via `impl EsiClient` blocks - no traits
- Bulk operations auto-chunk at documented ESI limits (e.g., 1000 for affiliation, name resolution)
- CLI commands map 1:1 to library endpoint methods
