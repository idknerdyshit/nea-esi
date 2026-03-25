# nea-esi-cli

Interactive CLI for EVE Online's ESI API, built on the `nea-esi` library crate.

## Build & Run

```bash
cargo build -p nea-esi-cli
cargo run -p nea-esi-cli -- status                          # One-shot command
cargo run -p nea-esi-cli -- --format json market prices     # JSON output
cargo run -p nea-esi-cli -- interactive                     # Launch REPL
```

## Module Layout

- **`main.rs`** - Entry point: parse args, load config, build `EsiClient`, dispatch command or launch REPL
- **`cli.rs`** - All clap derive definitions (`Cli`, `Commands`, subcommand enums)
- **`auth.rs`** - SSO login: starts local HTTP server on random port, opens browser to EVE SSO, receives callback. Defines `DEFAULT_SCOPES` (~27 read-only) and `WRITE_SCOPES` (additional write scopes). Falls back to copy-paste if browser fails.
- **`config.rs`** - Loads/saves `config.toml` from platform config dir (`directories` crate). Fields: `client_id`, `client_secret`, `user_agent`, defaults, auth scopes.
- **`token_store.rs`** - Persists `EsiTokens` to `tokens.json` alongside config
- **`output.rs`** - Formats results as JSON (pretty), ASCII table (`tabled`), or CSV. Auto-detects TTY for default format.
- **`repl.rs`** - Interactive REPL via `rustyline` with tab-completion for commands/subcommands, persistent history, shell-like arg splitting (`shlex`)
- **`error.rs`** - CLI error types
- **`commands/`** - 31 modules, one per ESI domain. Each module has a public `run` or `handle` function dispatched from `main.rs`.

## Config Files

Platform-specific config directory (via `directories` crate):
- macOS: `~/Library/Application Support/nea-esi/`
- Linux: `~/.config/nea-esi/`
- Windows: `AppData\Local\nea-esi\`

Files:
- `config.toml` - App credentials and defaults
- `tokens.json` - Persisted OAuth tokens
- `history.txt` - REPL command history

## Adding a New Command

1. Create `commands/{domain}.rs` with a handler function
2. Add the subcommand enum variant in `cli.rs`
3. Wire dispatch in `main.rs`
4. Add the corresponding library endpoint method in `nea-esi` if it doesn't exist
5. The handler should call the library method and pass the result to `output::print_output()`

## Output Formats

`--format` flag (or `set format` in REPL):
- `json` - Pretty-printed JSON (default when stdout is not a TTY)
- `table` - ASCII table via `tabled` (default when stdout is a TTY)
- `csv` - CSV rows

## Command Families

auth, config, status, market, character, corporation, alliance, universe, wallet, skills, assets, mail, fleet, industry, contracts, killmails, search, sovereignty, wars, fw, dogma, navigation, contacts, fittings, calendar, clones, loyalty, pi, mining, resolve
