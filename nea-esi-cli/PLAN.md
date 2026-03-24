# nea-esi-cli Implementation Plan

## Context

`nea-esi` is a Rust library with ~203 typed ESI endpoint methods, OAuth2 PKCE auth, rate limiting, caching, and pagination. We need a CLI binary that exposes all endpoints for both scripting (subcommands) and interactive use (REPL).

## Project Structure

```
nea-esi-cli/
  src/
    main.rs              # Entry point: parse args, dispatch
    cli.rs               # Top-level clap #[derive(Parser)]
    config.rs            # TOML config loading/saving
    auth.rs              # Auth flow: browser-open, copy-paste, token persistence
    token_store.rs       # Read/write tokens to disk
    output.rs            # JSON / table / CSV formatting
    repl.rs              # Interactive REPL (rustyline)
    error.rs             # CLI error type wrapping EsiError + IO
    commands/
      mod.rs             # Re-exports, shared dispatch helpers
      auth_cmd.rs        # auth login/logout/status
      config_cmd.rs      # config init/show/set
      status.rs          # server status, incursions, insurance
      market.rs          # ~8 endpoints
      character.rs       # ~50+ endpoints
      corporation.rs     # ~30+ endpoints
      alliance.rs        # ~6 endpoints
      universe.rs        # ~40+ endpoints
      wallet.rs          # character + corp wallet
      assets.rs          # character + corp assets
      skills.rs          # skills, queue, attributes
      mail.rs            # mail CRUD
      fleet.rs           # fleet management
      industry.rs        # industry jobs, facilities
      contracts.rs       # public + character + corp
      killmails.rs       # killmail lookup
      search.rs          # entity search
      sovereignty.rs     # sov map/campaigns/structures
      wars.rs            # war lookup + killmails
      fw.rs              # faction warfare
      dogma.rs           # attributes/effects
      navigation.rs      # routes + UI commands
      contacts.rs        # character + corp + alliance
      fittings.rs        # fitting CRUD
      calendar.rs        # calendar events
      clones.rs          # clones + implants
      loyalty.rs         # LP + store offers
      pi.rs              # planetary interaction
      mining.rs          # mining ledger
```

## Dependencies

```toml
[dependencies]
nea-esi = { path = "../nea-esi" }
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive", "env"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
tabled = "0.17"
csv = "1"
chrono = { version = "0.4", features = ["serde"] }
secrecy = { version = "0.10", features = ["serde"] }
directories = "6"
open = "5"
tiny_http = "0.12"
dialoguer = "0.11"
rustyline = "15"
colored = "3"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

## CLI Argument Structure

```
nea-esi-cli [OPTIONS] <COMMAND>

Global options:
  --format <json|table|csv>   Output format [default: table for TTY, json otherwise]
  --config <PATH>             Config file override
  --character-id <ID>         Override default character ID
  --no-color                  Disable colored output
  -v, --verbose               Enable tracing output

Commands:
  auth          login / logout / status
  config        init / show / set
  interactive   Launch REPL
  status        Server status
  market        Market data (history, orders, prices, types, groups)
  character     Character endpoints (~50 subcommands)
  corporation   Corporation endpoints (~30 subcommands)
  alliance      Alliance endpoints
  universe      Universe data (types, systems, regions, stations, etc.)
  wallet        Wallet balance/journal/transactions
  skills        Skills, queue, attributes
  assets        Asset listing/names/locations
  ...           (one group per command module)
```

Nested example: `nea-esi-cli market orders --region 10000002 --type 34`

Required IDs (like `character_id`) fall back to config defaults when not provided.

## Authentication

Three strategies, auto-detected in order:

1. **Existing tokens** — Load from `~/.config/nea-esi/tokens.json`. Auto-refresh if expired.
2. **Browser-open** — Spawn `tiny_http` on localhost, call `open::that()` for the auth URL, capture callback automatically.
3. **Copy-paste fallback** — If browser-open fails (or `--headless` flag), print the URL, prompt user to paste back the auth code.

### Token Persistence

- Location: `~/.config/nea-esi/tokens.json` (via `directories` crate)
- File permissions: `0600` on Unix
- Format: JSON with `access_token`, `refresh_token`, `expires_at`
- `auth login` writes, `auth logout` deletes, `auth status` shows expiry
- Every authenticated command calls `load_or_refresh_tokens()` which reads from disk, refreshes if needed, writes back

### Scope Selection

`auth login` accepts `--scopes <comma-list>` or `--all-scopes`. Defaults stored in config.

## Config File

Location: `~/.config/nea-esi/config.toml`

```toml
[app]
client_id = "your-eve-app-client-id"
# client_secret = "optional-for-web-apps"
user_agent = "nea-esi-cli"

[defaults]
character_id = 12345678
format = "table"
region_id = 10000002

[auth]
scopes = ["esi-wallet.read_character_wallet.v1", "..."]
headless = false
```

`config init` walks through setup interactively with dialoguer.

## Output Formatting

- **Auto**: Table if TTY, JSON otherwise
- **JSON**: `serde_json::to_string_pretty()`
- **Table**: `tabled` crate. Each command module defines thin display-adapter structs selecting/formatting key columns (ISK with commas, short datetimes, etc.). Full object goes to JSON/CSV.
- **CSV**: `csv::Writer` with serde serialization

## REPL Mode (`nea-esi-cli interactive`)

Built with `rustyline`:

- Prompt: `esi> ` (or `esi:market> ` in category context)
- Parses input as CLI args, reuses the same clap definitions
- **Tab completion** for commands/subcommands
- **Context navigation**: `cd market` scopes commands, `cd ..` resets
- **History**: persisted to `~/.config/nea-esi/history.txt`
- **Interactive param fill**: Missing required params prompt via `dialoguer::Input`
- **Built-ins**: `exit`, `quit`, `help`, `cd`, `set format <fmt>`

## Library Prerequisite (Phase 0)

All ~80 response structs in `nea-esi/src/lib.rs` only derive `Deserialize`. Must add `Serialize` to enable JSON/CSV output. This is a mechanical, non-breaking change.

**Files to modify:**
- `nea-esi/src/lib.rs` — Add `Serialize` to all `#[derive(..., Deserialize)]` on public response types
- `nea-esi/src/auth.rs` — Add `Serialize` to `EsiTokens` if not already present

## Implementation Phases

### Phase 0: Library — Add Serialize derives
- Add `Serialize` to all public response structs in `lib.rs` and `auth.rs`
- Verify `cargo test` passes

### Phase 1: CLI Scaffolding
- `Cargo.toml` with all deps
- `error.rs`, `config.rs`, `token_store.rs`, `output.rs` (JSON only first)
- `cli.rs` with top-level Parser and global options
- `main.rs` wiring: parse args → load config → create EsiClient → dispatch
- `commands/status.rs` as first smoke-test command (no auth needed)

### Phase 2: Auth Flow
- `auth.rs` — browser-open + copy-paste + token persistence
- `commands/auth_cmd.rs` — login/logout/status
- `commands/config_cmd.rs` — init/show/set
- End-to-end test: login → token file → authenticated request

### Phase 3: Core Command Groups
Implement in order of usefulness:
1. `market.rs` — public, high value
2. `universe.rs` — public lookups
3. `search.rs` — entity search
4. `character.rs` — the big one (50+ subcommands)
5. `wallet.rs`, `skills.rs`, `assets.rs`
6. `killmails.rs`

### Phase 4: Remaining Command Groups
- `corporation.rs`, `alliance.rs`, `fleet.rs`, `mail.rs`
- `industry.rs`, `contracts.rs`, `sovereignty.rs`
- `wars.rs`, `fw.rs`, `dogma.rs`, `navigation.rs`
- `contacts.rs`, `fittings.rs`, `calendar.rs`, `clones.rs`
- `loyalty.rs`, `pi.rs`, `mining.rs`

### Phase 5: Table & CSV Output
- Define `Tabled` display-adapter structs per command module
- Implement CSV writer
- Wire `--format` flag through all commands

### Phase 6: REPL Mode
- `repl.rs` with rustyline loop
- Tab completion, history, context navigation
- Interactive parameter prompting

### Phase 7: Polish
- `--verbose` tracing setup
- Colored errors
- Shell completion generation (`clap_complete`)
- Progress indicators for paginated fetches

## Verification

1. `cargo build` — workspace compiles
2. `cargo run -p nea-esi-cli -- status` — server status without auth
3. `cargo run -p nea-esi-cli -- auth login` — full OAuth flow
4. `cargo run -p nea-esi-cli -- character skills` — authenticated endpoint, table output
5. `cargo run -p nea-esi-cli -- market orders --region 10000002 --type 34 --format csv` — CSV output
6. `cargo run -p nea-esi-cli -- interactive` — REPL launches, tab completion works
