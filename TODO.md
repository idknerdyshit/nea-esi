# nea-esi TODO — ESI Endpoint Coverage

## Already Implemented

- [x] **Market**: `market_history`, `market_orders`, `market_prices`, `compute_best_bid_ask`
- [x] **Killmails**: `get_killmail` (raw JSON), `get_killmail_typed`
- [x] **Characters**: `get_character`
- [x] **Corporations**: `get_corporation`
- [x] **Alliances**: `get_alliance`
- [x] **Assets**: `character_assets` (authenticated, paginated)
- [x] **Universe**: `resolve_names` (POST), `get_structure` (authenticated)
- [x] **Auth**: OAuth2/SSO with PKCE, token refresh, web + native app flows
- [x] **Infrastructure**: rate limiting, error budget, concurrency cap, pagination, `request`/`request_post`

---

## Phase 1 — Core Public Endpoints

High-value public endpoints that don't require authentication.

### Universe / Static Data
- [x] `GET /universe/types/{type_id}/` — type info (name, description, group, market_group, etc.)
- [x] `GET /universe/types/` — list all type IDs (paginated)
- [x] `GET /universe/groups/{group_id}/` — inventory group info
- [x] `GET /universe/categories/{category_id}/` — inventory category info
- [x] `GET /universe/systems/{system_id}/` — solar system info (name, constellation, security)
- [x] `GET /universe/constellations/{constellation_id}/` — constellation info
- [x] `GET /universe/regions/{region_id}/` — region info (name, constellations)
- [x] `GET /universe/stations/{station_id}/` — NPC station info
- [x] `GET /universe/stargates/{stargate_id}/` — stargate info + destination
- [x] `POST /universe/ids/` — resolve names to IDs (reverse of `resolve_names`)

### Market (additional)
- [x] `GET /markets/{region_id}/types/` — list all type IDs with active orders in a region
- [x] `GET /markets/groups/` — list market group IDs
- [x] `GET /markets/groups/{market_group_id}/` — market group info

### Search
- [x] `GET /search/` — search for entities by name (public, unauthenticated)

### Killmails (additional)
- [x] `GET /characters/{character_id}/killmails/recent/` — recent killmails (authenticated)
- [x] `GET /corporations/{corporation_id}/killmails/recent/` — corp killmails (authenticated)

### Sovereignty
- [x] `GET /sovereignty/map/` — sovereignty map (who owns what)
- [x] `GET /sovereignty/campaigns/` — active sovereignty campaigns
- [x] `GET /sovereignty/structures/` — sovereignty structures

### Incursions
- [x] `GET /incursions/` — active incursions

### Status
- [x] `GET /status/` — server status (player count, server version)

---

## Phase 2 — Character Endpoints (Authenticated)

Personal character data — all require SSO tokens.

### Wallet
- [x] `GET /characters/{character_id}/wallet/` — ISK balance
- [x] `GET /characters/{character_id}/wallet/journal/` — wallet journal (paginated)
- [x] `GET /characters/{character_id}/wallet/transactions/` — wallet transactions

### Skills
- [x] `GET /characters/{character_id}/skills/` — trained skills
- [x] `GET /characters/{character_id}/skillqueue/` — skill queue
- [x] `GET /characters/{character_id}/attributes/` — character attributes

### Industry
- [x] `GET /characters/{character_id}/industry/jobs/` — industry jobs
- [x] `GET /characters/{character_id}/blueprints/` — owned blueprints

### Contracts
- [x] `GET /characters/{character_id}/contracts/` — personal contracts
- [x] `GET /characters/{character_id}/contracts/{contract_id}/items/` — contract items
- [x] `GET /characters/{character_id}/contracts/{contract_id}/bids/` — auction bids

### Orders
- [x] `GET /characters/{character_id}/orders/` — active market orders
- [x] `GET /characters/{character_id}/orders/history/` — expired/cancelled orders

### Fittings
- [x] `GET /characters/{character_id}/fittings/` — saved ship fittings
- [x] `POST /characters/{character_id}/fittings/` — save a fitting
- [x] `DELETE /characters/{character_id}/fittings/{fitting_id}/` — delete a fitting

### Location / Ship
- [x] `GET /characters/{character_id}/location/` — current location
- [x] `GET /characters/{character_id}/ship/` — current ship
- [x] `GET /characters/{character_id}/online/` — online status

### Mail
- [x] `GET /characters/{character_id}/mail/` — mail headers
- [x] `GET /characters/{character_id}/mail/{mail_id}/` — mail body
- [x] `POST /characters/{character_id}/mail/` — send mail
- [x] `GET /characters/{character_id}/mail/labels/` — mail labels

### Notifications
- [x] `GET /characters/{character_id}/notifications/` — notifications

### Contacts
- [x] `GET /characters/{character_id}/contacts/` — contact list
- [x] `GET /characters/{character_id}/contacts/labels/` — contact labels

### Bookmarks
- [x] `GET /characters/{character_id}/bookmarks/` — personal bookmarks
- [x] `GET /characters/{character_id}/bookmarks/folders/` — bookmark folders

### Calendar
- [x] `GET /characters/{character_id}/calendar/` — upcoming events
- [x] `GET /characters/{character_id}/calendar/{event_id}/` — event details

### Clones
- [x] `GET /characters/{character_id}/clones/` — jump clones
- [x] `GET /characters/{character_id}/implants/` — active implants

### Loyalty Points
- [x] `GET /characters/{character_id}/loyalty/points/` — LP balances
- [x] `GET /loyalty/stores/{corporation_id}/offers/` — LP store offers (public)

### PI (Planetary Interaction)
- [x] `GET /characters/{character_id}/planets/` — list colonies
- [x] `GET /characters/{character_id}/planets/{planet_id}/` — colony layout

---

## Phase 3 — Corporation Endpoints (Authenticated)

Corporation-level data — requires director/CEO roles.

### Corp Wallet
- [ ] `GET /corporations/{corporation_id}/wallets/` — division balances
- [ ] `GET /corporations/{corporation_id}/wallets/{division}/journal/` — division journal
- [ ] `GET /corporations/{corporation_id}/wallets/{division}/transactions/` — division transactions

### Corp Assets
- [ ] `GET /corporations/{corporation_id}/assets/` — corp assets (paginated)
- [ ] `POST /corporations/{corporation_id}/assets/names/` — name asset items
- [ ] `POST /corporations/{corporation_id}/assets/locations/` — asset locations

### Corp Industry
- [ ] `GET /corporations/{corporation_id}/industry/jobs/` — corp industry jobs
- [ ] `GET /corporations/{corporation_id}/blueprints/` — corp blueprints

### Corp Contracts
- [ ] `GET /corporations/{corporation_id}/contracts/` — corp contracts

### Corp Orders
- [ ] `GET /corporations/{corporation_id}/orders/` — corp market orders
- [ ] `GET /corporations/{corporation_id}/orders/history/` — corp order history

### Corp Members
- [ ] `GET /corporations/{corporation_id}/members/` — member list
- [ ] `GET /corporations/{corporation_id}/members/titles/` — member titles
- [ ] `GET /corporations/{corporation_id}/roles/` — member roles
- [ ] `GET /corporations/{corporation_id}/membertracking/` — member tracking

### Corp Structures
- [ ] `GET /corporations/{corporation_id}/structures/` — owned structures
- [ ] `GET /corporations/{corporation_id}/starbases/` — POSes
- [ ] `GET /corporations/{corporation_id}/starbases/{starbase_id}/` — POS config

---

## Phase 4 — Supplementary & Niche Endpoints

Lower-priority or niche endpoints.

### Dogma (item attributes)
- [ ] `GET /dogma/attributes/{attribute_id}/` — attribute info
- [ ] `GET /dogma/effects/{effect_id}/` — effect info
- [ ] `GET /dogma/dynamic/items/{type_id}/{item_id}/` — mutated item stats

### Opportunities
- [ ] `GET /opportunities/groups/` — opportunity groups
- [ ] `GET /opportunities/tasks/` — opportunity tasks
- [ ] `GET /characters/{character_id}/opportunities/` — completed tasks

### Fleet
- [ ] `GET /characters/{character_id}/fleet/` — current fleet
- [ ] `GET /fleets/{fleet_id}/` — fleet info
- [ ] `GET /fleets/{fleet_id}/members/` — fleet members
- [ ] `GET /fleets/{fleet_id}/wings/` — fleet wings

### Wars
- [ ] `GET /wars/` — list active wars
- [ ] `GET /wars/{war_id}/` — war details
- [ ] `GET /wars/{war_id}/killmails/` — war killmails

### Faction Warfare
- [ ] `GET /fw/stats/` — faction warfare stats
- [ ] `GET /fw/systems/` — contested systems
- [ ] `GET /fw/leaderboards/` — leaderboards
- [ ] `GET /fw/wars/` — faction warfare wars

### Insurance
- [ ] `GET /insurance/prices/` — insurance prices for all ships

### Routes
- [ ] `GET /route/{origin}/{destination}/` — shortest/safest/insecure route

### Corporation Public Info (additional)
- [ ] `GET /corporations/{corporation_id}/alliancehistory/` — alliance history
- [ ] `GET /characters/{character_id}/corporationhistory/` — corp history

---

## Infrastructure / Library Improvements

- [x] Generic paginated GET helper (`get_paginated<T>`)
- [x] Generic paginated POST helper (`post_paginated<T, B>`)
- [x] Caching layer (ETag / `If-None-Match` support via `with_cache()` + `request_cached()`)
- [x] Retry with exponential backoff on 502/503/504 (up to 3 attempts)
- [x] Batch ID resolution (`resolve_names` auto-chunks at 1000 IDs)
- [x] More station/region constants (Amarr, Dodixie, Rens, Hek + their regions)
