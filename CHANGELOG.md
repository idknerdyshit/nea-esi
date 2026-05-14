# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.9.0] / [0.2.0] - 2026-05-14

### Added
- `Isk` newtype (`nea-esi/src/types/common.rs`) wrapping `rust_decimal::Decimal` for all monetary values. Deserialized through serde_json's `arbitrary_precision` path so large wallet balances and prices round-trip exactly with no f64 rounding. `Deref`s to `Decimal`, implements `Display`, `Ord`, and `From`/`Into<Decimal>`. (GitHub #2)
- Re-export `rust_decimal::Decimal` as `nea_esi::Decimal` so consumers can do exact ISK arithmetic without adding the dependency directly.

### Changed
- **BREAKING:** every monetary field across the API is now `Isk` (or `Option<Isk>`) instead of `f64`: `EsiMarketHistoryEntry` (`average`/`highest`/`lowest`), `EsiMarketOrder.price`, `EsiMarketPrice` (`average_price`/`adjusted_price`), `EsiCharacterOrder` (`price`/`escrow`), `EsiContract` (`price`/`reward`/`collateral`/`buyout`), `EsiContractBid.amount`, `EsiWalletJournalEntry` (`amount`/`balance`/`tax`), `EsiWalletTransaction.unit_price`, `EsiWarParty.isk_destroyed`, `EsiInsuranceLevel` (`cost`/`payout`), `EsiCorpWalletDivision.balance`, `EsiStationInfo.office_rental_cost`, `EsiIndustryJob.cost`, `EsiIndustryFacility.tax`. Non-monetary `f64` fields (standings, security status, mass/volume, dogma values, tax *rates*, cost indices, positions) are unchanged.
- **BREAKING:** `wallet_balance()` returns `Result<Isk>` instead of `Result<f64>`.
- **BREAKING:** `compute_best_bid_ask()` returns `(Option<Isk>, Option<Isk>, i64, i64)` instead of `(Option<f64>, Option<f64>, i64, i64)`.
- `serde_json` now built with `arbitrary_precision`; added `rust_decimal` dependency with `serde-with-arbitrary-precision`.
- CLI CSV output routes items through `serde_json::Value` rather than serializing structs directly, since the csv crate cannot consume serde_json's arbitrary-precision number representation.

## [0.8.1] - 2026-04-29

### Fixed
- Point `nea-esi` package `readme` at the in-package `README.md` instead of `../README.md`, removing the cargo-publish warning about a path outside the package. The published library README is unchanged.

## [0.8.0] / [0.1.3] - 2026-04-29

### Added
- Expose corp ticker on `EsiCorporationInfo` (c80b2e8)

### Fixed
- Rate-limit deadlock when error budget hits zero with no follow-up reset header (c80b2e8)
- `character_asset_locations` now chunks at 1000 IDs (previously unchunked, would 4xx above the limit) (c80b2e8)

### Changed
- **BREAKING:** `EsiCorporationInfo.ticker` is a required field; manually-constructed values must include it (c80b2e8)
- Add `post_chunked_ids_json` helper and route corp/character asset names + locations through it (c80b2e8)
- Expand `.gitignore` with grouped Rust/OS/editor/env sections (5faf2fe)
- Clarify why-style comments in `nea-esi-cli` auth, config, main, token_store (5faf2fe)
- Pin `nea-esi` dependency in CLI crate via explicit version (ef017f4)

## [0.7.2] / [0.1.2] - 2026-03-25

### Fixed
- CLI profile paths and wallet journal pagination (633e51e)
- Clippy needless borrow warnings in token_store
- Formatting inconsistencies in CLI sources

## [0.7.1] / [0.1.1] - 2026-03-25

### Added
- ~80 remaining endpoints, 9 optional param fixes (430a038)
- 21 supplementary endpoints (242e7cc)
- 18 corporation endpoints (fa254e7)
- Bookmark, calendar, clone, loyalty, and PI endpoints (98af102)
- Mail, notification, and contact endpoints (a6cde17)
- Fitting, location, ship, and online endpoints (f504eb9)
- Industry, contract, and order endpoints (018697c)
- Skill endpoints: skills, skillqueue, attributes (b4b175f)
- Wallet endpoints and request_delete infrastructure (53c8cfd)
- Retry, pagination helpers, ETag caching, station constants (750c0c6)
- Four new ESI endpoints: assets, names, structures, prices (e9b21e0)
- EVE Online OAuth/SSO support with PKCE (64a3732)
- README and CLAUDE.md docs (b6eaf6b)
- Interactive CLI with REPL, tab-completion, multiple output formats (86210c0)

### Fixed
- All compiler warnings in nea-esi-cli (800ad55)
- Clear_cache panic, dead code removal, RNG optimization (15e6261)
- Double serialization fix (b51d30e)

### Changed
- Resolve all clippy pedantic warnings (~445 warnings fixed)
- Extract type definitions from lib.rs into types/ module (d3117ae)
- Split into workspace crates and modularize endpoints (86210c0)
- Typed timestamps, stricter serde (c365f05)
- Reduce endpoint and auth boilerplate with shared helpers (8502664)
- Deduplicate request/pagination logic, add network error retry (af4fcca)
- Configurable base_url for testability with mock servers (c0d2f74)
- Smarter rate-limit backoff using X-ESI-Error-Limit-Reset header (fe6ac54)
- Update reqwest to 0.13 and rand to 0.10 (ecfafa0)
