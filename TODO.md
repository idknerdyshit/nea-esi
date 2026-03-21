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
- [x] `GET /corporations/{corporation_id}/wallets/` — division balances
- [x] `GET /corporations/{corporation_id}/wallets/{division}/journal/` — division journal
- [x] `GET /corporations/{corporation_id}/wallets/{division}/transactions/` — division transactions

### Corp Assets
- [x] `GET /corporations/{corporation_id}/assets/` — corp assets (paginated)
- [x] `POST /corporations/{corporation_id}/assets/names/` — name asset items
- [x] `POST /corporations/{corporation_id}/assets/locations/` — asset locations

### Corp Industry
- [x] `GET /corporations/{corporation_id}/industry/jobs/` — corp industry jobs
- [x] `GET /corporations/{corporation_id}/blueprints/` — corp blueprints

### Corp Contracts
- [x] `GET /corporations/{corporation_id}/contracts/` — corp contracts

### Corp Orders
- [x] `GET /corporations/{corporation_id}/orders/` — corp market orders
- [x] `GET /corporations/{corporation_id}/orders/history/` — corp order history

### Corp Members
- [x] `GET /corporations/{corporation_id}/members/` — member list
- [x] `GET /corporations/{corporation_id}/members/titles/` — member titles
- [x] `GET /corporations/{corporation_id}/roles/` — member roles
- [x] `GET /corporations/{corporation_id}/membertracking/` — member tracking

### Corp Structures
- [x] `GET /corporations/{corporation_id}/structures/` — owned structures
- [x] `GET /corporations/{corporation_id}/starbases/` — POSes
- [x] `GET /corporations/{corporation_id}/starbases/{starbase_id}/` — POS config

---

## Phase 4 — Supplementary & Niche Endpoints

Lower-priority or niche endpoints.

### Dogma (item attributes)
- [x] `GET /dogma/attributes/{attribute_id}/` — attribute info
- [x] `GET /dogma/effects/{effect_id}/` — effect info
- [x] `GET /dogma/dynamic/items/{type_id}/{item_id}/` — mutated item stats

### Opportunities
- [x] `GET /opportunities/groups/` — opportunity groups
- [x] `GET /opportunities/tasks/` — opportunity tasks
- [x] `GET /characters/{character_id}/opportunities/` — completed tasks

### Fleet
- [x] `GET /characters/{character_id}/fleet/` — current fleet
- [x] `GET /fleets/{fleet_id}/` — fleet info
- [x] `GET /fleets/{fleet_id}/members/` — fleet members
- [x] `GET /fleets/{fleet_id}/wings/` — fleet wings

### Wars
- [x] `GET /wars/` — list active wars
- [x] `GET /wars/{war_id}/` — war details
- [x] `GET /wars/{war_id}/killmails/` — war killmails

### Faction Warfare
- [x] `GET /fw/stats/` — faction warfare stats
- [x] `GET /fw/systems/` — contested systems
- [x] `GET /fw/leaderboards/` — leaderboards
- [x] `GET /fw/wars/` — faction warfare wars

### Insurance
- [x] `GET /insurance/prices/` — insurance prices for all ships

### Routes
- [x] `GET /route/{origin}/{destination}/` — shortest/safest/insecure route

### Corporation Public Info (additional)
- [x] `GET /corporations/{corporation_id}/alliancehistory/` — alliance history
- [x] `GET /characters/{character_id}/corporationhistory/` — corp history

---

## Phase 5 — Remaining Endpoints

All remaining ESI endpoints, implemented in v0.7.0.

### Alliance
- [x] `GET /alliances/` — list all alliance IDs
- [x] `GET /alliances/{alliance_id}/corporations/` — alliance member corporation IDs
- [x] `GET /alliances/{alliance_id}/icons/` — alliance icon URLs
- [x] `GET /alliances/{alliance_id}/contacts/` — alliance contacts (auth, paginated)
- [x] `GET /alliances/{alliance_id}/contacts/labels/` — alliance contact labels (auth)

### Character — Info & History
- [x] `POST /characters/affiliation/` — bulk character affiliation lookup; req: `characters` body (max 1000)
- [x] `GET /characters/{character_id}/portrait/` — character portrait URLs
- [x] `GET /characters/{character_id}/roles/` — character roles (auth)
- [x] `GET /characters/{character_id}/titles/` — character titles (auth)
- [x] `GET /characters/{character_id}/standings/` — character standings (auth)
- [x] `GET /characters/{character_id}/medals/` — character medals (auth)
- [x] `GET /characters/{character_id}/agents_research/` — agent research info (auth)
- [x] `GET /characters/{character_id}/fatigue/` — jump fatigue (auth)
- [x] `GET /characters/{character_id}/fw/stats/` — character FW stats (auth)
- [x] `POST /characters/{character_id}/cspa/` — CSPA charge cost; req: `characters` body

### Character — Asset Details
- [x] `POST /characters/{character_id}/assets/locations/` — character asset locations; req: `item_ids` body (max 1000)
- [x] `POST /characters/{character_id}/assets/names/` — character asset names; req: `item_ids` body (max 1000)

### Character — Contact CRUD
- [x] `POST /characters/{character_id}/contacts/` — add contacts; req: `standing` query + `contact_ids` body (max 100); opt: `label_ids`, `watched`
- [x] `PUT /characters/{character_id}/contacts/` — edit contacts; req: `standing` query + `contact_ids` body (max 100); opt: `label_ids`, `watched`
- [x] `DELETE /characters/{character_id}/contacts/` — delete contacts; req: `contact_ids` query (max 20)

### Character — Calendar Write
- [x] `PUT /characters/{character_id}/calendar/{event_id}/` — set event response; req: `response` body
- [x] `GET /characters/{character_id}/calendar/{event_id}/attendees/` — event attendees (auth)

### Character — Mail Management
- [x] `POST /characters/{character_id}/mail/labels/` — create mail label; req: `label` body
- [x] `DELETE /characters/{character_id}/mail/labels/{label_id}/` — delete mail label
- [x] `GET /characters/{character_id}/mail/lists/` — mailing lists (auth)
- [x] `DELETE /characters/{character_id}/mail/{mail_id}/` — delete a mail
- [x] `PUT /characters/{character_id}/mail/{mail_id}/` — update mail metadata (read/labels); req: `contents` body

### Character — Mining
- [x] `GET /characters/{character_id}/mining/` — personal mining ledger (auth, paginated)

### Character — Notifications
- [x] `GET /characters/{character_id}/notifications/contacts/` — contact notifications (auth)

### Character — Search
- [x] `GET /characters/{character_id}/search/` — authenticated search; req: `categories` + `search` query; opt: `strict`

### Public Contracts
- [x] `GET /contracts/public/{region_id}/` — public contracts in a region (paginated)
- [x] `GET /contracts/public/bids/{contract_id}/` — public contract bids (paginated)
- [x] `GET /contracts/public/items/{contract_id}/` — public contract items (paginated)

### Corporation — Additional
- [x] `GET /corporations/npccorps/` — list NPC corporation IDs
- [x] `GET /corporations/{corporation_id}/contacts/` — corp contacts (auth, paginated)
- [x] `GET /corporations/{corporation_id}/contacts/labels/` — corp contact labels (auth)
- [x] `GET /corporations/{corporation_id}/containers/logs/` — container audit logs (auth, paginated)
- [x] `GET /corporations/{corporation_id}/contracts/{contract_id}/bids/` — corp contract bids (auth)
- [x] `GET /corporations/{corporation_id}/contracts/{contract_id}/items/` — corp contract items (auth)
- [x] `GET /corporations/{corporation_id}/customs_offices/` — customs offices (auth, paginated)
- [x] `GET /corporations/{corporation_id}/divisions/` — corp divisions (auth)
- [x] `GET /corporations/{corporation_id}/facilities/` — corp facilities (auth)
- [x] `GET /corporations/{corporation_id}/fw/stats/` — corp FW stats (auth)
- [x] `GET /corporations/{corporation_id}/icons/` — corp icon URLs
- [x] `GET /corporations/{corporation_id}/medals/` — corp medals (auth, paginated)
- [x] `GET /corporations/{corporation_id}/medals/issued/` — issued medals (auth, paginated)
- [x] `GET /corporations/{corporation_id}/members/limit/` — member limit (auth)
- [x] `GET /corporations/{corporation_id}/roles/history/` — role change history (auth, paginated)
- [x] `GET /corporations/{corporation_id}/shareholders/` — shareholders (auth, paginated)
- [x] `GET /corporations/{corporation_id}/standings/` — corp standings (auth, paginated)
- [x] `GET /corporations/{corporation_id}/titles/` — corp titles (auth)

### Corporation — Mining (note: singular `/corporation/` path)
- [x] `GET /corporation/{corporation_id}/mining/extractions/` — moon mining extractions (auth, paginated)
- [x] `GET /corporation/{corporation_id}/mining/observers/` — mining observers (auth, paginated)
- [x] `GET /corporation/{corporation_id}/mining/observers/{observer_id}/` — observer details (auth, paginated)

### Dogma — List Endpoints
- [x] `GET /dogma/attributes/` — list all dogma attribute IDs
- [x] `GET /dogma/effects/` — list all dogma effect IDs

### Fleet — Write Endpoints
- [x] `PUT /fleets/{fleet_id}/` — update fleet settings; req: `new_settings` body
- [x] `POST /fleets/{fleet_id}/members/` — invite to fleet; req: `invitation` body
- [x] `DELETE /fleets/{fleet_id}/members/{member_id}/` — kick member
- [x] `PUT /fleets/{fleet_id}/members/{member_id}/` — move member; req: `movement` body
- [x] `POST /fleets/{fleet_id}/wings/` — create wing
- [x] `PUT /fleets/{fleet_id}/wings/{wing_id}/` — rename wing; req: `naming` body
- [x] `DELETE /fleets/{fleet_id}/wings/{wing_id}/` — delete wing
- [x] `POST /fleets/{fleet_id}/wings/{wing_id}/squads/` — create squad
- [x] `PUT /fleets/{fleet_id}/squads/{squad_id}/` — rename squad; req: `naming` body
- [x] `DELETE /fleets/{fleet_id}/squads/{squad_id}/` — delete squad

### Faction Warfare — Additional
- [x] `GET /fw/leaderboards/characters/` — character FW leaderboards
- [x] `GET /fw/leaderboards/corporations/` — corporation FW leaderboards

### Industry — Public
- [x] `GET /industry/facilities/` — public industry facilities
- [x] `GET /industry/systems/` — industry system cost indices

### Market — Structure Orders
- [x] `GET /markets/structures/{structure_id}/` — orders at a structure (auth, paginated)

### UI — In-Game Window Commands
- [x] `POST /ui/autopilot/waypoint/` — set autopilot waypoint; req: `add_to_beginning`, `clear_other_waypoints`, `destination_id` query params
- [x] `POST /ui/openwindow/contract/` — open contract window; req: `contract_id` query
- [x] `POST /ui/openwindow/information/` — open info window; req: `target_id` query
- [x] `POST /ui/openwindow/marketdetails/` — open market details; req: `type_id` query
- [x] `POST /ui/openwindow/newmail/` — open new mail window; req: `new_mail` body

### Universe — Additional
- [x] `GET /universe/ancestries/` — list ancestries
- [x] `GET /universe/asteroid_belts/{asteroid_belt_id}/` — asteroid belt info
- [x] `GET /universe/bloodlines/` — list bloodlines
- [x] `GET /universe/categories/` — list all category IDs
- [x] `GET /universe/constellations/` — list all constellation IDs
- [x] `GET /universe/factions/` — list factions
- [x] `GET /universe/graphics/` — list graphic IDs
- [x] `GET /universe/graphics/{graphic_id}/` — graphic info
- [x] `GET /universe/groups/` — list all group IDs (paginated)
- [x] `GET /universe/moons/{moon_id}/` — moon info
- [x] `GET /universe/planets/{planet_id}/` — planet info (public, not PI)
- [x] `GET /universe/races/` — list races
- [x] `GET /universe/regions/` — list all region IDs
- [x] `GET /universe/schematics/{schematic_id}/` — PI schematic info
- [x] `GET /universe/stars/{star_id}/` — star info
- [x] `GET /universe/structures/` — list all public structure IDs
- [x] `GET /universe/system_jumps/` — system jump statistics
- [x] `GET /universe/system_kills/` — system kill statistics
- [x] `GET /universe/systems/` — list all system IDs

---

## Optional Parameter Gaps in Existing Endpoints

- [x] `character_industry_jobs` — add `include_completed: bool` param to fetch completed jobs
- [x] `corp_industry_jobs` — add `include_completed: bool` param to fetch completed jobs
- [x] `character_mail` — add `labels: Option<&[i32]>` param to filter by label IDs
- [x] `wallet_transactions` — add `from_id: Option<i64>` param for cursor pagination through older transactions
- [x] `corp_wallet_transactions` — add `from_id: Option<i64>` param for cursor pagination
- [x] `character_calendar` — add `from_event: Option<i32>` param for cursor pagination
- [x] `get_route` — add `connections: Option<&[[i32; 2]]>` param for wormhole connections
- [x] `list_war_ids` — add `max_war_id: Option<i32>` param for backward pagination
- [x] `market_orders` — expose `order_type` param (previously hardcoded to `"all"`)

---

## Infrastructure / Library Improvements

- [x] Generic paginated GET helper (`get_paginated<T>`)
- [x] Generic paginated POST helper (`post_paginated<T, B>`)
- [x] Caching layer (ETag / `If-None-Match` support via `with_cache()` + `request_cached()`)
- [x] Retry with exponential backoff on 502/503/504 (up to 3 attempts)
- [x] Batch ID resolution (`resolve_names` auto-chunks at 1000 IDs)
- [x] More station/region constants (Amarr, Dodixie, Rens, Hek + their regions)
- [x] Generic PUT helper (`request_put`) for fleet/mail/calendar/contact write endpoints
- [x] Generic DELETE-with-query-params helper for contact/mail deletion
