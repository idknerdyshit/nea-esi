# AGENTS.md

## Scope

This file applies to the `nea-esi/` crate.

## Crate Summary

`nea-esi` is the async library crate for EVE Online ESI access. It owns:

- `EsiClient` construction and shared request logic
- OAuth2 PKCE and token refresh behavior
- error-budget tracking, retry, pagination, and optional ETag caching
- public request/response types re-exported from the crate root
- endpoint methods exposed as `impl EsiClient` blocks

## Structure

- `src/lib.rs`: crate root, `EsiClient`, error types, shared HTTP logic, constants, cache, and public re-exports
- `src/auth.rs`: app credentials, PKCE challenge generation, token exchange, token refresh, and auth helpers
- `src/endpoints/mod.rs`: shared endpoint helpers and domain module wiring
- `src/endpoints/*.rs`: endpoint-specific `impl EsiClient` blocks
- `src/types/mod.rs`: public type re-exports
- `src/types/*.rs`: response/request model groupings by domain
- `tests/auth_integration.rs`: mocked auth/token flows with `wiremock`
- `tests/endpoints_integration.rs`: mocked endpoint coverage with `wiremock`

Current endpoint modules:

- `alliance`
- `character`
- `character_social`
- `corporation`
- `fleet`
- `killmail`
- `market`
- `misc`
- `universe`

Current type modules:

- `alliance`
- `character`
- `common`
- `corporation`
- `fleet`
- `industry`
- `killmail`
- `market`
- `misc`
- `social`
- `universe`

## Development Conventions

- Keep new endpoint methods in `src/endpoints/` as `impl EsiClient` blocks.
- Keep public request/response models in `src/types/` and re-export them through the crate root unless there is a strong reason not to.
- Async library operations should continue returning `nea_esi::Result<T>` / `Result<T, EsiError>`, while constructors and pure helpers can remain synchronous.
- Shared HTTP behavior belongs in `src/lib.rs` or `src/auth.rs`; endpoint-shared helpers should stay centralized in `src/endpoints/mod.rs` instead of being duplicated across endpoint modules.
- Testing hooks such as `with_base_url()` and `with_sso_token_url()` are part of the intended design; use them instead of adding ad hoc test-only paths.

## Editing Guidance

- Extend nearby types and endpoint modules instead of creating large new catch-all files.
- Preserve existing pagination, retry, and auth behavior unless the task explicitly changes it.
- If you add an endpoint that needs auth scopes, verify the library requirement and consider whether the CLI’s default scopes should also change.
- If you change auth or token-refresh behavior, inspect unit tests in `src/auth.rs` and integration coverage in `tests/auth_integration.rs`.

## Verification

Run from the workspace root:

```bash
cargo test -p nea-esi
```

Use broader verification when appropriate:

- request/auth/pagination changes: update or add `wiremock` coverage in `tests/`
- public API changes: verify docs in `README.md` or `nea-esi/README.md`
- cross-crate changes: `cargo test --workspace`
