# AGENTS.md

## Scope

This file applies to the entire repository.

Crate-specific guidance lives in:

- `nea-esi/AGENTS.md`
- `nea-esi-cli/AGENTS.md`

## Workspace Summary

`nea-esi` is a Rust workspace with resolver `3` and two members:

- `nea-esi/`: async library crate for EVE Online ESI access, OAuth/SSO, pagination, retry, ETag caching, and rate-limit handling
- `nea-esi-cli/`: interactive CLI crate built on top of the library with clap commands, config persistence, token storage, browser or headless login, and a REPL

Top-level files:

- `Cargo.toml`: workspace manifest
- `README.md`: workspace overview and quick-start docs for both crates
- `CHANGELOG.md`: release notes

## Workspace Structure

- `nea-esi/`: library crate; see `nea-esi/AGENTS.md`
- `nea-esi-cli/`: CLI crate; see `nea-esi-cli/AGENTS.md`

## Build And Test

Run these from the workspace root:

```bash
cargo build --workspace
cargo test --workspace
cargo test -p nea-esi
cargo build -p nea-esi-cli
cargo run -p nea-esi-cli -- --help
```

Preferred verification after changes:

- library-only changes: `cargo test -p nea-esi`
- CLI-only changes: `cargo build -p nea-esi-cli`
- cross-cutting changes: `cargo test --workspace`

## Development Conventions

- Rust edition is `2024` in both crates.
- Prefer small, targeted edits and preserve unrelated local changes.
- Keep shared API models and HTTP behavior in the library crate; keep CLI concerns in the CLI crate.
- For new end-to-end functionality, add or update the library surface first, then wire it into the CLI if needed.
- Prefer mocked HTTP coverage with `wiremock` over live ESI calls.
- If a change affects auth behavior, inspect both crates because auth responsibilities are split between library and CLI layers.

## Editing Guidance

- Do not revert or overwrite user changes outside the task you are handling.
- Keep module boundaries aligned with the existing crate layout instead of adding cross-cutting shortcuts at the workspace root.
- Update crate-local docs or READMEs when public behavior changes materially.
- When changing files inside a crate, follow the more specific instructions in that crate's `AGENTS.md`.

## Verification Guidance

- Request/auth/pagination changes should include or update mocked tests in `nea-esi/tests/`.
- CLI parsing, dispatch, or persistence changes should at minimum keep `nea-esi-cli` building.
- Cross-cutting changes should be validated at the workspace level.

## Repository State

- `git status` was clean when this file was updated.
- Avoid encoding transient local-state notes here unless they are still true and useful for future work.
