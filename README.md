# nea-esi

Rate-limited Rust client library and interactive CLI for EVE Online's [ESI](https://esi.evetech.net/) (EVE Swagger Interface) API.

## Features

- **Full OAuth2 PKCE** (RFC 7636) - supports both native (public) and web (confidential) app credentials
- **Automatic rate limiting** - ESI error budget tracking, concurrent request semaphore, exponential backoff with jitter
- **Token management** - auto-refresh before expiry, concurrent refresh prevention, auto-retry on 401
- **Parallel pagination** - fetches page 1, then spawns parallel requests for remaining pages
- **ETag caching** - optional response cache, serves cached body on 304
- **100+ endpoints** - characters, corporations, markets, universe, alliances, fleets, killmails, and more
- **Interactive CLI** - REPL with tab-completion, history, and JSON/table/CSV output formats

## Workspace Structure

| Crate | Version | Description |
|-------|---------|-------------|
| [`nea-esi`](nea-esi/) | 0.7.2 | Async ESI client library |
| [`nea-esi-cli`](nea-esi-cli/) | 0.1.2 | Interactive CLI |

## Quick Start

### Library

Add to your `Cargo.toml`:

```toml
[dependencies]
nea-esi = { git = "https://github.com/idknerdyshit/nea-esi" }
```

Basic usage:

```rust
use nea_esi::{EsiClient, EsiAppCredentials};
use secrecy::SecretString;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = EsiAppCredentials::Native {
        client_id: "your-client-id".to_string(),
    };
    let client = EsiClient::new(credentials, "my-app/1.0 contact@example.com");

    // Public endpoints work without authentication
    let prices = client.get_market_prices().await?;
    println!("Found {} market prices", prices.len());

    // Authenticated endpoints require completing the OAuth2 PKCE flow first
    // See the CLI for a full login example

    Ok(())
}
```

### CLI

```bash
# Build
cargo build -p nea-esi-cli

# Configure credentials
cargo run -p nea-esi-cli -- config set-client-id <your-client-id>

# Log in via EVE SSO
cargo run -p nea-esi-cli -- auth login

# Run commands
cargo run -p nea-esi-cli -- market prices
cargo run -p nea-esi-cli -- --format json character info
cargo run -p nea-esi-cli -- status

# Interactive REPL
cargo run -p nea-esi-cli -- interactive
```

Output formats (`--format`):
- **table** - ASCII table (default in TTY)
- **json** - pretty-printed JSON (default when piped)
- **csv** - CSV rows

## EVE Developer Setup

1. Register an application at [EVE Developers](https://developers.eveonline.com/)
2. For native apps, set the callback URL to `http://localhost/callback`
3. Copy your Client ID (and Client Secret for web apps)
4. Configure via the CLI or pass credentials directly to the library

## Building

```bash
cargo build --workspace    # Build everything
cargo test --workspace     # Run all tests
```

Rust edition 2024, resolver v3. Tests use [wiremock](https://crates.io/crates/wiremock) for HTTP mocking - no live ESI calls needed.

## License

MIT
