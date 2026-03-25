# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

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
