# AGENTS.md

## Scope

This file applies to the `nea-esi-cli/` crate.

## Crate Summary

`nea-esi-cli` is the binary crate that exposes the library through clap commands and an interactive REPL. It owns:

- CLI argument parsing and command dispatch
- config loading and saving
- token persistence
- browser-based and headless login UX
- output formatting for JSON, tables, and CSV
- REPL history and command execution

## Structure

- `src/main.rs`: entry point, config loading, client construction, token loading/saving, and top-level dispatch
- `src/cli.rs`: clap command tree and global options
- `src/commands/mod.rs`: command module exports and `ExecContext`
- `src/commands/*.rs`: command handlers mapped to ESI domains
- `src/auth.rs`: login flow, default scopes, local callback server, and copy-paste fallback
- `src/config.rs`: config schema, default paths, load/save logic
- `src/token_store.rs`: persisted OAuth token handling
- `src/output.rs`: output-format selection and rendering helpers
- `src/repl.rs`: interactive shell, completion, and history

Current command modules:

- `alliance`
- `assets`
- `auth_cmd`
- `calendar`
- `character`
- `clones`
- `config_cmd`
- `contacts`
- `contracts`
- `corporation`
- `dogma`
- `fittings`
- `fleet`
- `fw`
- `industry`
- `killmails`
- `loyalty`
- `mail`
- `market`
- `mining`
- `navigation`
- `pi`
- `resolve`
- `search`
- `skills`
- `sovereignty`
- `status`
- `universe`
- `wallet`
- `wars`

## Development Conventions

- The CLI should stay a thin layer over `nea-esi`; avoid reimplementing HTTP or auth protocol details here.
- Add new user-facing capabilities by wiring existing or newly added library methods into the clap tree and command handlers.
- Keep config, token persistence, REPL, auth UX, and rendering concerns separated in their existing modules.
- Default scopes are defined in `src/auth.rs`; update them deliberately when adding authenticated commands that should work after a standard login.

## Editing Guidance

- If you touch CLI parsing or dispatch, update both `src/cli.rs` and the relevant `src/commands/*.rs` module as needed.
- If you touch login behavior, inspect both `src/auth.rs` and the library crate’s `src/auth.rs`.
- Respect the current config directory and file layout implemented in `src/config.rs` and `src/token_store.rs`.
- Keep one-shot command behavior and REPL behavior aligned; the REPL dispatches through the same command surface.

## Verification

Run from the workspace root:

```bash
cargo build -p nea-esi-cli
cargo run -p nea-esi-cli -- --help
```

Use broader verification when appropriate:

- cross-cutting changes: `cargo test --workspace`
- auth-flow changes: also run `cargo test -p nea-esi`
- user-facing behavior changes: update `nea-esi-cli/README.md` if commands, options, or setup steps changed
