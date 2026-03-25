# AGENTS.md

## Scope

This file applies to the entire repository.

## Repository Summary

`nea-esi` is a Rust workspace with two crates:

- `nea-esi/`: async library crate for EVE Online ESI access, OAuth/SSO, pagination, retry, ETag caching, and rate-limit handling
- `nea-esi-cli/`: interactive CLI crate built on top of the library, with clap-based commands, REPL support, config persistence, and browser login flow

Workspace root:

- `Cargo.toml`: workspace manifest with resolver `3`

## Structure

- `nea-esi/src/lib.rs`: core `EsiClient`, response types, constants, shared request logic
- `nea-esi/src/auth.rs`: OAuth2 PKCE, token models, credential handling
- `nea-esi/src/endpoints/*.rs`: endpoint-specific `impl EsiClient` blocks
- `nea-esi/tests/auth_integration.rs`: auth/token integration coverage with `wiremock`
- `nea-esi/tests/endpoints_integration.rs`: endpoint integration coverage with `wiremock`
- `nea-esi-cli/src/main.rs`: CLI entry point and dispatch
- `nea-esi-cli/src/cli.rs`: clap command definitions
- `nea-esi-cli/src/auth.rs`: browser login and callback handling
- `nea-esi-cli/src/config.rs`: config file loading/saving
- `nea-esi-cli/src/token_store.rs`: token persistence
- `nea-esi-cli/src/repl.rs`: interactive REPL
- `nea-esi-cli/src/commands/*.rs`: command handlers mapped to ESI domains

## Build And Test

Run these from the workspace root:

```bash
cargo build --workspace
cargo test --workspace
cargo test -p nea-esi
cargo build -p nea-esi-cli
cargo run -p nea-esi-cli -- --help
```

Preferred verification after code changes:

- library changes: `cargo test -p nea-esi`
- CLI-only changes: `cargo build -p nea-esi-cli`
- cross-cutting changes: `cargo test --workspace`

## Development Conventions

- Rust edition is `2024` in both crates.
- Keep endpoint implementations in `nea-esi/src/endpoints/` as `impl EsiClient` blocks.
- Keep public response/request models in the library crate, primarily in `nea-esi/src/lib.rs` unless there is a strong reason to localize them.
- Library methods are async and should continue returning `nea_esi::Result<T>` / `Result<T, EsiError>`.
- The CLI generally mirrors library capabilities 1:1 through command modules.
- Tests use `wiremock`; prefer mocked HTTP flows over live ESI calls.
- There are many existing unit tests in `nea-esi/src/lib.rs` and `nea-esi/src/auth.rs`; extend nearby tests when changing related behavior.

## Editing Guidance

- Prefer small, targeted edits. This repo may already contain unrelated local changes.
- Do not revert or overwrite user changes outside the task you are handling.
- Keep new code aligned with the existing async/Tokio style and current module boundaries.
- For new ESI endpoints, add the library method first, then add or update the matching CLI command if appropriate.
- For auth changes, inspect both `nea-esi/src/auth.rs` and `nea-esi-cli/src/auth.rs`; behavior is split across the library and CLI.

## Verification Guidance

- If you touch request/auth/pagination logic, add or update `wiremock` tests.
- If you touch CLI parsing or dispatch, at minimum confirm the crate still builds with `cargo build -p nea-esi-cli`.
- If you add new public API surface, ensure the change is reflected in crate docs or README when warranted.

## Known Local State

No known dirty worktree state. Merge conflicts in `nea-esi/src/auth.rs` and `nea-esi-cli/src/auth.rs` have been resolved.
