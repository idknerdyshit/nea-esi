# nea-esi-cli

Interactive CLI for EVE Online's [ESI](https://esi.evetech.net/) API, built on the [`nea-esi`](../nea-esi/) library crate.

## Installation

```bash
cargo build -p nea-esi-cli
```

## Setup

1. Register an application at [EVE Developers](https://developers.eveonline.com/)
2. Configure your Client ID:
   ```bash
   nea-esi-cli config set-client-id <your-client-id>
   ```
3. Log in via EVE SSO (opens browser, receives callback on a local HTTP server):
   ```bash
   nea-esi-cli auth login
   ```

Config files are stored in a platform-specific directory:
- **macOS**: `~/Library/Application Support/nea-esi/`
- **Linux**: `~/.config/nea-esi/`
- **Windows**: `AppData\Local\nea-esi\`

Files:
| File | Purpose |
|------|---------|
| `config.toml` | App credentials and defaults |
| `tokens.json` | Persisted OAuth tokens |
| `history.txt` | REPL command history |

## Usage

```bash
# One-shot commands
nea-esi-cli status
nea-esi-cli market prices
nea-esi-cli character info
nea-esi-cli universe types --search "Tritanium"

# Output format (default: table for TTY, json when piped)
nea-esi-cli --format json market prices
nea-esi-cli --format csv wallet journal | head -20

# Override character
nea-esi-cli --character-id 12345 wallet balance

# Interactive REPL
nea-esi-cli interactive
```

## Output Formats

| Format | Flag | Description |
|--------|------|-------------|
| table | `--format table` | ASCII table (default in TTY) |
| json | `--format json` | Pretty-printed JSON (default when piped) |
| csv | `--format csv` | CSV rows |

## Global Options

| Option | Description |
|--------|-------------|
| `--format <FORMAT>` | Output format: json, table, csv |
| `--config <PATH>` | Override config file path |
| `--character-id <ID>` | Override default character ID |
| `--no-color` | Disable colored output |
| `-v, --verbose` | Enable verbose/debug output |

## Command Groups

| Command | Description |
|---------|-------------|
| `auth` | Authentication management (login, logout, status) |
| `config` | Configuration management |
| `interactive` | Launch interactive REPL |
| `status` | EVE server status |
| `market` | Market data (orders, history, prices) |
| `character` | Character info, location, ship |
| `corporation` | Corporation info, members, structures |
| `alliance` | Alliance info and members |
| `universe` | Types, systems, regions, stations |
| `wallet` | Wallet balance and journal |
| `skills` | Skills and skill queue |
| `assets` | Asset management |
| `mail` | EVE mail |
| `fleet` | Fleet management |
| `industry` | Industry jobs |
| `contracts` | Contracts |
| `killmails` | Killmail history |
| `search` | Search for entities |
| `sovereignty` | Sovereignty data |
| `wars` | War information |
| `fw` | Faction warfare |
| `dogma` | Dogma attributes and effects |
| `navigation` | Route planning and UI waypoints |
| `contacts` | Contact management |
| `fittings` | Ship fittings |
| `calendar` | Calendar events |
| `clones` | Clones and implants |
| `loyalty` | Loyalty points |
| `pi` | Planetary interaction |
| `mining` | Mining ledger |
| `resolve` | Name/ID resolution |

## Interactive REPL

The `interactive` command launches a REPL with:
- Tab-completion for commands and subcommands
- Persistent command history
- Shell-like argument splitting
- All commands available without the `nea-esi-cli` prefix

```
> status
> market prices
> character info
> set format json
```

## License

MIT
