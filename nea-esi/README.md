# nea-esi

Rate-limited Rust client for EVE Online's [ESI](https://esi.evetech.net/ui/) (EVE Swagger Interface) API. This library handles concurrency, pagination, retry, ETag caching, OAuth/SSO, and ESI's error budget system so callers don't have to.

## Adding to a project

```toml
[dependencies]
nea-esi = { path = "../nea-esi" }  # or wherever this lives relative to the consuming crate
tokio = { version = "1", features = ["full"] }
```

The crate re-exports everything from `lib.rs` — there are no feature flags.

## Quick start

```rust
use nea_esi::{EsiClient, compute_best_bid_ask, THE_FORGE, JITA_STATION};

#[tokio::main]
async fn main() -> nea_esi::Result<()> {
    let client = EsiClient::with_user_agent(
        "my-app (contact@example.com; +https://github.com/me/my-app; eve:MyCharacter)",
    );

    // Market history for Tritanium (type_id 34) in The Forge
    let history = client.market_history(THE_FORGE, 34).await?;
    println!("Got {} daily entries", history.len());

    // Market orders + best bid/ask at Jita 4-4
    let orders = client.market_orders(THE_FORGE, 34).await?;
    let (bid, ask, bid_vol, ask_vol) = compute_best_bid_ask(&orders, JITA_STATION);
    println!("Bid: {:?} (vol {}), Ask: {:?} (vol {})", bid, bid_vol, ask, ask_vol);

    Ok(())
}
```

## OAuth / SSO

The client supports EVE SSO with PKCE for both web and native apps.

```rust
use nea_esi::EsiClient;
use secrecy::SecretString;

// Native (public) app — no client secret needed
let client = EsiClient::with_native_app("my-app", "your-client-id");

// Web (confidential) app
let client = EsiClient::with_web_app("my-app", "your-client-id", secret);

// 1. Generate authorize URL
let challenge = client.authorize_url(
    "http://localhost:8080/callback",
    &["esi-wallet.read_character_wallet.v1"],
)?;
println!("Open in browser: {}", challenge.authorize_url);

// 2. Exchange the code after callback
let tokens = client.exchange_code(&code, &challenge.code_verifier, redirect_uri).await?;

// 3. Authenticated endpoints now work automatically
let assets = client.character_assets(character_id).await?;
```

Token management methods: `set_tokens()`, `get_tokens()`, `clear_tokens()`, `refresh_token()`. The client auto-refreshes tokens when they're within 60 seconds of expiry and retries once on 401.

## API reference

### Client construction

| Constructor | Description |
|---|---|
| `EsiClient::new()` | Default user agent, 30s timeout |
| `EsiClient::with_user_agent(ua)` | Custom user agent, 30s timeout |
| `EsiClient::with_native_app(ua, client_id)` | Native app with PKCE (public client) |
| `EsiClient::with_web_app(ua, client_id, secret)` | Web app with PKCE (confidential client) |
| `EsiClient::default()` | Same as `new()` |
| `.credentials(creds)` | Set app credentials (builder pattern) |
| `.with_base_url(url)` | Override base URL, e.g. for testing with mock servers (builder pattern) |
| `.with_cache()` | Enable ETag response caching (builder pattern) |

ESI requires a descriptive User-Agent. Format: `app-name (contact; +repo_url; eve:CharacterName)`.

### Methods

All methods are `async` and return `nea_esi::Result<T>`.

**Market data:**

```rust
// Daily OHLCV data for a type in a region
client.market_history(region_id: i32, type_id: i32) -> Vec<EsiMarketHistoryEntry>

// All orders for a type in a region (handles pagination automatically)
client.market_orders(region_id: i32, type_id: i32) -> Vec<EsiMarketOrder>

// Global average and adjusted prices for all types
client.market_prices() -> Vec<EsiMarketPrice>

// List all type IDs with active orders in a region (paginated)
client.market_type_ids(region_id: i32) -> Vec<i32>

// List all market group IDs
client.market_group_ids() -> Vec<i32>

// Fetch market group info
client.get_market_group(market_group_id: i32) -> EsiMarketGroupInfo

// Filter orders to a station, return (best_bid, best_ask, bid_volume, ask_volume)
// This is a free function — no &self, no async
compute_best_bid_ask(orders: &[EsiMarketOrder], station_id: i64)
    -> (Option<f64>, Option<f64>, i64, i64)
```

**Killmails:**

```rust
// Raw JSON — useful when you need fields not in the typed struct
client.get_killmail(killmail_id: i64, killmail_hash: &str) -> serde_json::Value

// Typed — parses into EsiKillmail with victim, attackers, items
client.get_killmail_typed(killmail_id: i64, killmail_hash: &str) -> EsiKillmail

// Recent killmails for a character (authenticated, paginated)
client.character_killmails(character_id: i64) -> Vec<EsiKillmailRef>

// Recent killmails for a corporation (authenticated, paginated)
client.corporation_killmails(corporation_id: i64) -> Vec<EsiKillmailRef>
```

**Entity lookups:**

```rust
client.get_character(character_id: i64) -> EsiCharacterInfo     // name, corp, alliance
client.get_corporation(corporation_id: i64) -> EsiCorporationInfo // name, alliance, member_count
client.get_alliance(alliance_id: i64) -> EsiAllianceInfo         // name, ticker
client.get_structure(structure_id: i64) -> EsiStructureInfo      // name, owner, system (authenticated)
```

**Universe:**

```rust
client.get_type(type_id: i32) -> EsiTypeInfo
client.list_type_ids() -> Vec<i32>                              // paginated
client.get_group(group_id: i32) -> EsiGroupInfo
client.get_category(category_id: i32) -> EsiCategoryInfo
client.get_system(system_id: i32) -> EsiSolarSystemInfo
client.get_constellation(constellation_id: i32) -> EsiConstellationInfo
client.get_region(region_id: i32) -> EsiRegionInfo
client.get_station(station_id: i32) -> EsiStationInfo
client.get_stargate(stargate_id: i32) -> EsiStargateInfo

// Resolve IDs to names and categories (auto-chunks at 1000 per request)
client.resolve_names(ids: &[i64]) -> Vec<EsiResolvedName>

// Resolve names to IDs (auto-chunks at 500 per request)
client.resolve_ids(names: &[String]) -> EsiResolvedIds

// Search for entities by name
client.search(search: &str, categories: &str, strict: bool) -> EsiSearchResult
```

**Sovereignty:**

```rust
client.sovereignty_map() -> Vec<EsiSovereigntyMap>
client.sovereignty_campaigns() -> Vec<EsiSovereigntyCampaign>
client.sovereignty_structures() -> Vec<EsiSovereigntyStructure>
```

**Character — Wallet:**

```rust
client.wallet_balance(character_id: i64) -> f64
client.wallet_journal(character_id: i64) -> Vec<EsiWalletJournalEntry>  // paginated
client.wallet_transactions(character_id: i64) -> Vec<EsiWalletTransaction>
```

**Character — Skills:**

```rust
client.character_skills(character_id: i64) -> EsiSkills
client.character_skillqueue(character_id: i64) -> Vec<EsiSkillqueueEntry>
client.character_attributes(character_id: i64) -> EsiAttributes
```

**Character — Industry:**

```rust
client.character_industry_jobs(character_id: i64) -> Vec<EsiIndustryJob>
client.character_blueprints(character_id: i64) -> Vec<EsiBlueprint>  // paginated
```

**Character — Contracts:**

```rust
client.character_contracts(character_id: i64) -> Vec<EsiContract>  // paginated
client.character_contract_items(character_id: i64, contract_id: i64) -> Vec<EsiContractItem>
client.character_contract_bids(character_id: i64, contract_id: i64) -> Vec<EsiContractBid>
```

**Character — Orders:**

```rust
client.character_orders(character_id: i64) -> Vec<EsiCharacterOrder>
client.character_order_history(character_id: i64) -> Vec<EsiCharacterOrder>  // paginated
```

**Character — Fittings:**

```rust
client.character_fittings(character_id: i64) -> Vec<EsiFitting>
client.create_fitting(character_id: i64, &EsiNewFitting) -> i64  // POST, returns fitting_id
client.delete_fitting(character_id: i64, fitting_id: i64) -> ()  // DELETE
```

**Character — Location:**

```rust
client.character_location(character_id: i64) -> EsiLocation
client.character_ship(character_id: i64) -> EsiShip
client.character_online(character_id: i64) -> EsiOnlineStatus
```

**Character — Mail:**

```rust
client.character_mail(character_id: i64) -> Vec<EsiMailHeader>
client.character_mail_before(character_id: i64, last_mail_id: i64) -> Vec<EsiMailHeader>
client.character_mail_body(character_id: i64, mail_id: i64) -> EsiMailBody
client.send_mail(character_id: i64, &EsiNewMail) -> i32  // POST, returns mail_id
client.character_mail_labels(character_id: i64) -> EsiMailLabels
```

**Character — Notifications & Contacts:**

```rust
client.character_notifications(character_id: i64) -> Vec<EsiNotification>
client.character_contacts(character_id: i64) -> Vec<EsiContact>  // paginated
client.character_contact_labels(character_id: i64) -> Vec<EsiContactLabel>
```

**Character — Calendar:**

```rust
client.character_calendar(character_id: i64) -> Vec<EsiCalendarEvent>
client.character_calendar_event(character_id: i64, event_id: i64) -> EsiCalendarEventDetail
```

**Character — Clones & Loyalty:**

```rust
client.character_clones(character_id: i64) -> EsiClones
client.character_implants(character_id: i64) -> Vec<i32>
client.character_loyalty_points(character_id: i64) -> Vec<EsiLoyaltyPoints>
client.loyalty_store_offers(corporation_id: i64) -> Vec<EsiLoyaltyStoreOffer>  // public
```

**Character — Planetary Interaction:**

```rust
client.character_planets(character_id: i64) -> Vec<EsiPlanetSummary>
client.character_planet_detail(character_id: i64, planet_id: i32) -> EsiPlanetDetail
```

**Corporation — Wallet:**

```rust
client.corp_wallet_balances(corporation_id: i64) -> Vec<EsiCorpWalletDivision>
client.corp_wallet_journal(corporation_id: i64, division: i32) -> Vec<EsiWalletJournalEntry>  // paginated
client.corp_wallet_transactions(corporation_id: i64, division: i32) -> Vec<EsiWalletTransaction>
```

**Corporation — Assets:**

```rust
client.corp_assets(corporation_id: i64) -> Vec<EsiAssetItem>  // paginated
client.corp_asset_names(corporation_id: i64, item_ids: &[i64]) -> Vec<EsiAssetName>  // POST
client.corp_asset_locations(corporation_id: i64, item_ids: &[i64]) -> Vec<EsiAssetLocation>  // POST
```

**Corporation — Industry:**

```rust
client.corp_industry_jobs(corporation_id: i64) -> Vec<EsiIndustryJob>  // paginated
client.corp_blueprints(corporation_id: i64) -> Vec<EsiBlueprint>  // paginated
```

**Corporation — Contracts:**

```rust
client.corp_contracts(corporation_id: i64) -> Vec<EsiContract>  // paginated
```

**Corporation — Orders:**

```rust
client.corp_orders(corporation_id: i64) -> Vec<EsiCharacterOrder>
client.corp_order_history(corporation_id: i64) -> Vec<EsiCharacterOrder>  // paginated
```

**Corporation — Members:**

```rust
client.corp_members(corporation_id: i64) -> Vec<i64>
client.corp_member_titles(corporation_id: i64) -> Vec<EsiCorpMemberTitle>
client.corp_member_roles(corporation_id: i64) -> Vec<EsiCorpMemberRole>
client.corp_member_tracking(corporation_id: i64) -> Vec<EsiCorpMemberTracking>
```

**Corporation — Structures:**

```rust
client.corp_structures(corporation_id: i64) -> Vec<EsiCorpStructure>  // paginated
client.corp_starbases(corporation_id: i64) -> Vec<EsiCorpStarbase>
client.corp_starbase_detail(corporation_id: i64, starbase_id: i64, system_id: i32) -> EsiCorpStarbaseDetail
```

**Dogma:**

```rust
client.get_dogma_attribute(attribute_id: i32) -> EsiDogmaAttribute
client.get_dogma_effect(effect_id: i32) -> EsiDogmaEffect
client.get_dynamic_item(type_id: i32, item_id: i64) -> EsiDynamicItem
```

**Opportunities:**

```rust
client.opportunity_group_ids() -> Vec<i32>
client.opportunity_task_ids() -> Vec<i32>
client.character_opportunities(character_id: i64) -> Vec<EsiCompletedOpportunity>  // authenticated
```

**Fleet:**

```rust
client.character_fleet(character_id: i64) -> EsiCharacterFleet  // authenticated
client.get_fleet(fleet_id: i64) -> EsiFleetInfo
client.fleet_members(fleet_id: i64) -> Vec<EsiFleetMember>
client.fleet_wings(fleet_id: i64) -> Vec<EsiFleetWing>
```

**Wars:**

```rust
client.list_war_ids() -> Vec<i32>  // paginated
client.get_war(war_id: i32) -> EsiWar
client.war_killmails(war_id: i32) -> Vec<EsiKillmailRef>  // paginated
```

**Faction Warfare:**

```rust
client.fw_stats() -> Vec<EsiFwFactionStats>
client.fw_systems() -> Vec<EsiFwSystem>
client.fw_leaderboards() -> EsiFwLeaderboards
client.fw_wars() -> Vec<EsiFwWar>
```

**Insurance:**

```rust
client.insurance_prices() -> Vec<EsiInsurancePrice>
```

**Routes:**

```rust
client.get_route(origin: i32, destination: i32, flag: Option<&str>, avoid: &[i32]) -> Vec<i32>
```

**History:**

```rust
client.corp_alliance_history(corporation_id: i64) -> Vec<EsiAllianceHistoryEntry>
client.character_corporation_history(character_id: i64) -> Vec<EsiCorporationHistoryEntry>
```

**Other:**

```rust
client.incursions() -> Vec<EsiIncursion>
client.server_status() -> EsiServerStatus
client.character_assets(character_id: i64) -> Vec<EsiAssetItem>  // authenticated, paginated
```

**Generic pagination helpers:**

```rust
// Paginated GET — fetches page 1, reads x-pages, spawns concurrent tasks for the rest
client.get_paginated::<T>(base_url: &str) -> Vec<T>

// Paginated POST — same pattern, serializes body once and clones into each task
client.post_paginated::<T, B>(base_url: &str, body: &B) -> Vec<T>
```

**Low-level:**

```rust
// Rate-limited GET with retry on 502/503/504 and network errors
client.request(url: &str) -> reqwest::Response

// Rate-limited POST with retry on 502/503/504 and network errors
client.request_post(url: &str, body: &impl Serialize) -> reqwest::Response

// Rate-limited DELETE with retry on 502/503/504 and network errors
client.request_delete(url: &str) -> reqwest::Response

// GET with ETag caching (returns raw bytes; requires .with_cache())
client.request_cached(url: &str) -> Vec<u8>

// Current error budget (starts at 100, updated from X-ESI-Error-Limit-Remain header)
client.error_budget() -> i32

// Clear cached ETag responses (async)
client.clear_cache().await
```

### Constants

| Constant | Value | Notes |
|---|---|---|
| `BASE_URL` | `https://esi.evetech.net/latest` | Default base URL (overridable via `with_base_url`) |
| `THE_FORGE` | `10000002` | Region ID — Jita's region |
| `DOMAIN` | `10000043` | Region ID — Amarr's region |
| `SINQ_LAISON` | `10000032` | Region ID — Dodixie's region |
| `HEIMATAR` | `10000030` | Region ID — Rens's region |
| `METROPOLIS` | `10000042` | Region ID — Hek's region |
| `JITA_STATION` | `60003760` | Station ID — Jita 4-4 CNAP |
| `AMARR_STATION` | `60008494` | Station ID — Amarr VIII |
| `DODIXIE_STATION` | `60011866` | Station ID — Dodixie IX |
| `RENS_STATION` | `60004588` | Station ID — Rens VI |
| `HEK_STATION` | `60005686` | Station ID — Hek VIII |
| `DEFAULT_USER_AGENT` | `nea-esi (https://github.com/...)` | Used by `EsiClient::new()` |

### Response types

**`EsiMarketHistoryEntry`** — `date: NaiveDate`, `average: f64`, `highest: f64`, `lowest: f64`, `volume: i64`, `order_count: i64`

**`EsiMarketOrder`** — `order_id: i64`, `type_id: i32`, `location_id: i64`, `price: f64`, `volume_remain: i64`, `is_buy_order: bool`, `issued: DateTime<Utc>`, `duration: i32`, `min_volume: i32`, `range: String`

**`EsiMarketPrice`** — `type_id: i32`, `average_price: Option<f64>`, `adjusted_price: Option<f64>`

**`EsiKillmail`** — `killmail_id: i64`, `killmail_time: DateTime<Utc>`, `solar_system_id: i32`, `victim: EsiKillmailVictim`, `attackers: Vec<EsiKillmailAttacker>`

**`EsiKillmailVictim`** — `ship_type_id: i32`, `character_id: Option<i64>`, `corporation_id: Option<i64>`, `alliance_id: Option<i64>`, `items: Vec<EsiKillmailItem>`

**`EsiKillmailAttacker`** — `character_id: Option<i64>`, `corporation_id: Option<i64>`, `alliance_id: Option<i64>`, `ship_type_id: i32`, `weapon_type_id: i32`, `damage_done: i32`, `final_blow: bool`

**`EsiKillmailItem`** — `item_type_id: i32`, `quantity_destroyed: Option<i64>`, `quantity_dropped: Option<i64>`, `flag: i32`, `singleton: i32`

**`EsiCharacterInfo`** — `name: String`, `corporation_id: Option<i64>`, `alliance_id: Option<i64>`

**`EsiCorporationInfo`** — `name: String`, `alliance_id: Option<i64>`, `member_count: Option<i32>`

**`EsiAllianceInfo`** — `name: String`, `ticker: Option<String>`

**`EsiAssetItem`** — `item_id: i64`, `type_id: i32`, `location_id: i64`, `location_type: String`, `location_flag: String`, `quantity: i32`, `is_singleton: bool`, `is_blueprint_copy: Option<bool>`

**`EsiResolvedName`** — `id: i64`, `name: String`, `category: String`

**`EsiStructureInfo`** — `name: String`, `owner_id: i64`, `solar_system_id: i32`, `type_id: Option<i32>`

**`EsiServerStatus`** — `players: i32`, `server_version: Option<String>`, `start_time: Option<DateTime<Utc>>`, `vip: Option<bool>`

**`EsiSovereigntyStructure`** — `alliance_id: Option<i64>`, `solar_system_id: i32`, `structure_id: i64`, `structure_type_id: i32`, `vulnerability_occupancy_level: Option<f64>`, `vulnerable_start_time: Option<DateTime<Utc>>`, `vulnerable_end_time: Option<DateTime<Utc>>`

**`EsiWalletJournalEntry`** — `id: i64`, `date: DateTime<Utc>`, `ref_type: String`, optional: `amount`, `balance`, `description`, `first_party_id`, `second_party_id`, `reason`, `context_id`, `context_id_type`, `tax`, `tax_receiver_id`

**`EsiWalletTransaction`** — `transaction_id: i64`, `date: DateTime<Utc>`, `type_id: i32`, `location_id: i64`, `unit_price: f64`, `quantity: i32`, `client_id: i64`, `is_buy: bool`, `is_personal: bool`, `journal_ref_id: i64`

**`EsiSkills`** — `skills: Vec<EsiSkill>`, `total_sp: i64`, optional: `unallocated_sp`

**`EsiSkill`** — `skill_id: i32`, `trained_skill_level: i32`, `active_skill_level: i32`, `skillpoints_in_skill: i64`

**`EsiSkillqueueEntry`** — `skill_id: i32`, `finish_level: i32`, `queue_position: i32`, optional: `start_date`, `finish_date`, `training_start_sp`, `level_start_sp`, `level_end_sp`

**`EsiAttributes`** — `intelligence/memory/perception/willpower/charisma: i32`, optional: `bonus_remaps`, `last_remap_date`, `accrued_remap_cooldown_date`

**`EsiIndustryJob`** — `job_id: i32`, `installer_id: i64`, `facility_id: i64`, `activity_id: i32`, `blueprint_id: i64`, `blueprint_type_id: i32`, `blueprint_location_id: i64`, `output_location_id: i64`, `runs: i32`, `status: String`, `duration: i32`, `start_date/end_date: DateTime<Utc>`, optional: `cost`, `licensed_runs`, `probability`, `product_type_id`, `pause_date`, `completed_date`, `completed_character_id`, `successful_runs`, `station_id`

**`EsiBlueprint`** — `item_id: i64`, `type_id: i32`, `location_id: i64`, `location_flag: String`, `quantity: i32`, `time_efficiency: i32`, `material_efficiency: i32`, `runs: i32`

**`EsiContract`** — `contract_id: i64`, `issuer_id/issuer_corporation_id: i64`, `contract_type: String`, `status/availability: String`, `date_issued/date_expired: DateTime<Utc>`, `for_corporation: bool`, optional: `assignee_id`, `acceptor_id`, `title`, `date_accepted`, `date_completed`, `price`, `reward`, `collateral`, `buyout`, `volume`, `days_to_complete`, `start_location_id`, `end_location_id`

**`EsiContractItem`** — `record_id: i64`, `type_id: i32`, `quantity: i32`, `is_included: bool`, optional: `is_singleton`, `raw_quantity`

**`EsiContractBid`** — `bid_id: i64`, `bidder_id: i64`, `date_bid: DateTime<Utc>`, `amount: f64`

**`EsiCharacterOrder`** — `order_id: i64`, `type_id: i32`, `region_id: i32`, `location_id: i64`, `range: String`, `is_buy_order: bool`, `price: f64`, `volume_total/volume_remain: i32`, `issued: DateTime<Utc>`, `min_volume: i32`, `duration: i32`, optional: `state`, `escrow`, `is_corporation`

**`EsiFitting`** — `fitting_id: i64`, `name: String`, `description: String`, `ship_type_id: i32`, `items: Vec<EsiFittingItem>`

**`EsiFittingItem`** — `type_id: i32`, `flag: i32`, `quantity: i32`

**`EsiLocation`** — `solar_system_id: i32`, optional: `station_id`, `structure_id`

**`EsiShip`** — `ship_type_id: i32`, `ship_item_id: i64`, `ship_name: String`

**`EsiOnlineStatus`** — `online: bool`, optional: `last_login`, `last_logout`, `logins`

**`EsiMailHeader`** — `mail_id: i64`, `timestamp: DateTime<Utc>`, optional: `from`, `subject`, `is_read`, default-vec: `labels`, `recipients`

**`EsiMailBody`** — optional: `body`, `from`, `read`, `subject`, `timestamp`, default-vec: `labels`, `recipients`

**`EsiMailLabels`** — `total_unread_count: i32`, `labels: Vec<EsiMailLabel>`

**`EsiNotification`** — `notification_id: i64`, `notification_type: String`, `sender_id: i64`, `sender_type: String`, `timestamp: DateTime<Utc>`, optional: `is_read`, `text`

**`EsiContact`** — `contact_id: i64`, `contact_type: String`, `standing: f64`, default-vec: `label_ids`, optional: `is_watched`

**`EsiContactLabel`** — `label_id: i64`, `label_name: String`

**`EsiCalendarEvent`** — `event_id: i64`, `event_date: DateTime<Utc>`, `title: String`, optional: `importance`, `event_response`

**`EsiCalendarEventDetail`** — `event_id: i64`, `date: DateTime<Utc>`, `title: String`, `owner_id: i64`, `owner_name: String`, `owner_type: String`, `duration: i32`, optional: `text`, `importance`, `response`

**`EsiClones`** — optional: `home_location`, `last_clone_jump_date`, `last_station_change_date`, default-vec: `jump_clones`

**`EsiJumpClone`** — `jump_clone_id: i64`, `location_id: i64`, `location_type: String`, default-vec: `implants`, optional: `name`

**`EsiLoyaltyPoints`** — `corporation_id: i64`, `loyalty_points: i32`

**`EsiLoyaltyStoreOffer`** — `offer_id: i32`, `type_id: i32`, `quantity: i32`, `lp_cost: i32`, `isk_cost: i64`, optional: `ak_cost`, default-vec: `required_items`

**`EsiPlanetSummary`** — `solar_system_id: i32`, `planet_id: i32`, `planet_type: String`, `num_pins: i32`, `last_update: DateTime<Utc>`, `upgrade_level: i32`, optional: `owner_id`

**`EsiPlanetDetail`** — `links/pins/routes: Vec<serde_json::Value>` (complex nested PI structures; typed access deferred)

**`EsiCorpWalletDivision`** — `division: i32`, `balance: f64`

**`EsiAssetName`** — `item_id: i64`, `name: String`

**`EsiAssetLocation`** — `item_id: i64`, `position: EsiPosition`

**`EsiCorpMemberTitle`** — `character_id: i64`, `titles: Vec<i32>` (default)

**`EsiCorpMemberRole`** — `character_id: i64`, `roles/roles_at_hq/roles_at_base/roles_at_other: Vec<String>` (all default)

**`EsiCorpMemberTracking`** — `character_id: i64`, optional: `location_id`, `logon_date`, `logoff_date`, `ship_type_id`, `start_date`

**`EsiCorpStructure`** — `structure_id: i64`, `corporation_id: i64`, `system_id: i32`, `type_id: i32`, `state: String`, optional: `name`, `profile_id`, `fuel_expires`, timers, `reinforce_hour`, default-vec: `services`

**`EsiCorpStructureService`** — `name: String`, `state: String`

**`EsiCorpStarbase`** — `starbase_id: i64`, `system_id: i32`, `type_id: i32`, `state: String`, optional: `moon_id`, `onlined_since`, `reinforced_until`, `unanchor_at`

**`EsiCorpStarbaseDetail`** — `state: String`, boolean flags, optional role access fields, optional thresholds, default-vec: `fuels`

**`EsiStarbaseFuel`** — `type_id: i32`, `quantity: i32`

**`EsiDogmaAttribute`** — `attribute_id: i32`, `name: String`, `published: bool`, optional/default: `description`, `icon_id`, `default_value: f64`, `display_name`, `unit_id`, `stackable: bool`, `high_is_good: bool`

**`EsiDogmaEffect`** — `effect_id: i32`, `name: String`, `published: bool`, optional/default: `description`, `icon_id`, `display_name`, `effect_category`, booleans (`is_assistance`, `is_offensive`, `is_warp_safe`), attribute IDs, `modifiers: Vec<EsiDogmaModifier>`

**`EsiDogmaModifier`** — all optional: `domain`, `effect_id`, `func`, `modified_attribute_id`, `modifying_attribute_id`, `operator`

**`EsiDynamicItem`** — `created_by: i64`, `mutator_type_id: i32`, `source_type_id: i32`, default-vec: `dogma_attributes`, `dogma_effects`

**`EsiDogmaAttributeValue`** — `attribute_id: i32`, `value: f64`

**`EsiDogmaEffectRef`** — `effect_id: i32`, `is_default: bool`

**`EsiCompletedOpportunity`** — `opportunity_id: i32`, `completed_at: DateTime<Utc>`

**`EsiCharacterFleet`** — `fleet_id: i64`, `role: String`, `squad_id: i64`, `wing_id: i64`

**`EsiFleetInfo`** — `fleet_id: i64`, default: `is_free_move/is_registered/is_voice_enabled: bool`, optional: `motd`

**`EsiFleetMember`** — `character_id: i64`, `join_time: DateTime<Utc>`, `role/role_name: String`, `ship_type_id: i32`, `solar_system_id: i32`, `squad_id/wing_id: i64`, `takes_fleet_warp: bool`, optional: `station_id`

**`EsiFleetWing`** — `id: i64`, `name: String`, default-vec: `squads: Vec<EsiFleetSquad>`

**`EsiFleetSquad`** — `id: i64`, `name: String`

**`EsiWar`** — `id: i32`, `declared: DateTime<Utc>`, `mutual/open_for_allies: bool`, `aggressor/defender: EsiWarParty`, optional: `started`, `finished`, `retracted`, default-vec: `allies`

**`EsiWarParty`** — `isk_destroyed: f64`, `ships_killed: i32`, optional: `alliance_id`, `corporation_id`

**`EsiWarAlly`** — optional: `alliance_id`, `corporation_id`

**`EsiFwFactionStats`** — `faction_id: i32`, `pilots: i32`, `systems_controlled: i32`, `kills/victory_points: EsiFwTotals`

**`EsiFwTotals`** — `last_week: i32`, `total: i32`, `yesterday: i32`

**`EsiFwSystem`** — `solar_system_id: i32`, `contested: String`, `occupier_faction_id/owner_faction_id: i32`, `victory_points/victory_points_threshold: i32`

**`EsiFwLeaderboards`** — `kills/victory_points: EsiFwLeaderboardCategory`

**`EsiFwLeaderboardCategory`** — default-vec: `active_total/last_week/yesterday: Vec<EsiFwLeaderboardEntry>`

**`EsiFwLeaderboardEntry`** — `amount: i32`, `id: i32`

**`EsiFwWar`** — `against_id: i32`, `faction_id: i32`

**`EsiInsurancePrice`** — `type_id: i32`, default-vec: `levels: Vec<EsiInsuranceLevel>`

**`EsiInsuranceLevel`** — `cost: f64`, `name: String`, `payout: f64`

**`EsiAllianceHistoryEntry`** — `record_id: i32`, `start_date: DateTime<Utc>`, optional: `alliance_id: i64`, default: `is_deleted: bool`

**`EsiCorporationHistoryEntry`** — `record_id: i32`, `start_date: DateTime<Utc>`, `corporation_id: i64`, default: `is_deleted: bool`

**`EsiTokens`** — `access_token: SecretString`, `refresh_token: SecretString`, `expires_at: DateTime<Utc>`

### Error handling

```rust
use nea_esi::EsiError;

match client.market_history(THE_FORGE, 34).await {
    Ok(entries) => { /* use entries */ }
    Err(EsiError::RateLimited) => { /* budget exhausted, wait and retry */ }
    Err(EsiError::Api { status, message }) => { /* ESI returned an error status */ }
    Err(EsiError::Http(e)) => { /* network/connection error */ }
    Err(EsiError::Deserialize(msg)) => { /* response didn't match expected shape */ }
    Err(EsiError::Internal(msg)) => { /* semaphore closed or similar */ }
    Err(EsiError::Auth(msg)) => { /* missing credentials or SSO config error */ }
    Err(EsiError::TokenRefresh(msg)) => { /* token exchange/refresh failed */ }
}
```

## Rate limiting behavior

The client manages ESI's error budget automatically:

1. **Concurrency cap**: Max 20 in-flight requests (tokio semaphore).
2. **Budget tracking**: After each response, `X-ESI-Error-Limit-Remain` header updates the internal budget (starts at 100).
3. **Smart backoff**: When budget drops below 20, each request sleeps until the `X-ESI-Error-Limit-Reset` window (falls back to 60s if no reset header received).
4. **Budget exhausted**: When budget hits 0, requests immediately return `EsiError::RateLimited` without making a network call.
5. **Transient retry**: 502, 503, 504 responses and network errors are retried up to 3 times with exponential backoff (1s base, doubled each attempt, plus random 0–500ms jitter).

Check the budget at any time with `client.error_budget()`. The budget is shared across clones of the client's internal `Arc<AtomicI32>` — concurrent tasks spawned by pagination all share the same budget.

## Pagination

`market_orders()`, `character_assets()`, and any endpoint using `get_paginated()` / `post_paginated()` handle pagination transparently. They read the `x-pages` header from the first response, then fetch remaining pages concurrently via `tokio::spawn`. All pages share the same rate limiter.

## ETag caching

Enable with `.with_cache()`:

```rust
let client = EsiClient::with_user_agent("my-app").with_cache();

// First call: normal GET, caches response + ETag
let bytes = client.request_cached(&url).await?;

// Subsequent calls: sends If-None-Match, returns cached body on 304
let bytes = client.request_cached(&url).await?;

// Free memory when needed
client.clear_cache().await;
```

Best for endpoints that change infrequently: `market_prices`, `get_character`, `get_corporation`, `get_alliance`, etc. Not used automatically for paginated endpoints.

## Migrating from 0.5.x to 0.6.0

No breaking changes. 0.6.0 adds 21 supplementary endpoints:

- **Dogma**: `get_dogma_attribute`, `get_dogma_effect`, `get_dynamic_item`
- **Opportunities**: `opportunity_group_ids`, `opportunity_task_ids`, `character_opportunities`
- **Fleet**: `character_fleet`, `get_fleet`, `fleet_members`, `fleet_wings`
- **Wars**: `list_war_ids`, `get_war`, `war_killmails`
- **Faction Warfare**: `fw_stats`, `fw_systems`, `fw_leaderboards`, `fw_wars`
- **Insurance**: `insurance_prices`
- **Routes**: `get_route` (supports `flag` and `avoid` params)
- **History**: `corp_alliance_history`, `character_corporation_history`

## Migrating from 0.4.x to 0.5.0

No breaking changes. 0.5.0 adds 18 corporation-level endpoints (director/CEO role required):

- **Corp Wallet**: `corp_wallet_balances`, `corp_wallet_journal`, `corp_wallet_transactions`
- **Corp Assets**: `corp_assets`, `corp_asset_names`, `corp_asset_locations`
- **Corp Industry**: `corp_industry_jobs`, `corp_blueprints`
- **Corp Contracts**: `corp_contracts`
- **Corp Orders**: `corp_orders`, `corp_order_history`
- **Corp Members**: `corp_members`, `corp_member_titles`, `corp_member_roles`, `corp_member_tracking`
- **Corp Structures**: `corp_structures`, `corp_starbases`, `corp_starbase_detail`

## Migrating from 0.3.x to 0.4.0

No breaking changes. 0.4.0 adds comprehensive character-level endpoint coverage:

- **Wallet**: `wallet_balance`, `wallet_journal`, `wallet_transactions`
- **Skills**: `character_skills`, `character_skillqueue`, `character_attributes`
- **Industry**: `character_industry_jobs`, `character_blueprints`
- **Contracts**: `character_contracts`, `character_contract_items`, `character_contract_bids`
- **Orders**: `character_orders`, `character_order_history`
- **Fittings**: `character_fittings`, `create_fitting`, `delete_fitting`
- **Location**: `character_location`, `character_ship`, `character_online`
- **Mail**: `character_mail`, `character_mail_before`, `character_mail_body`, `send_mail`, `character_mail_labels`
- **Notifications**: `character_notifications`
- **Contacts**: `character_contacts`, `character_contact_labels`
- **Calendar**: `character_calendar`, `character_calendar_event`
- **Clones**: `character_clones`, `character_implants`
- **Loyalty**: `character_loyalty_points`, `loyalty_store_offers`
- **PI**: `character_planets`, `character_planet_detail`
- **Infrastructure**: `request_delete` for DELETE endpoints

## Migrating from 0.2.x to 0.3.0

### Breaking changes

- **`compute_best_bid_ask` is now a free function** — change `EsiClient::compute_best_bid_ask(...)` to `compute_best_bid_ask(...)` (re-exported from crate root).
- **`clear_cache()` is now async** — add `.await` to calls.
- **Timestamp fields are now typed** instead of `String`:
  - `EsiMarketHistoryEntry::date` → `NaiveDate`
  - `EsiKillmail::killmail_time` → `DateTime<Utc>`
  - `EsiMarketOrder::issued` → `DateTime<Utc>`
  - `EsiSovereigntyCampaign::start_time` → `Option<DateTime<Utc>>`
  - `EsiSovereigntyStructure::vulnerable_start_time` / `vulnerable_end_time` → `Option<DateTime<Utc>>`
  - `EsiServerStatus::start_time` → `Option<DateTime<Utc>>`
- **`EsiSovereigntyStructure::alliance_id`** changed from `i64` to `Option<i64>` (not all sov structures belong to an alliance).
- **Stricter serde deserialization** — `#[serde(default)]` removed from fields ESI always provides (`solar_system_id`, `ship_type_id`, `group_id`, `published`, `system_id`, `type_id`, `constellation_id`, `region_id`, `players`, etc.). JSON missing these fields will now fail to deserialize instead of silently defaulting to zero/false.

### New features

- **Configurable base URL** — `EsiClient::new().with_base_url("http://localhost:8080")` for testing with mock servers.
- **Network error retry** — transient network failures (not just 502-504) are now retried with exponential backoff.
- **`request_cached()` now has full retry/401 handling** — previously it skipped the retry loop and token refresh logic.

## Logging

The crate uses `tracing` for structured logging:
- `debug!` on every successful request (URL, status, elapsed_ms, error_budget)
- `warn!` on low budget backoff, API errors, and transient retries

Wire up a `tracing_subscriber` in the consuming binary to see these.

## Common type IDs

This library doesn't bundle a type/region database. Look up IDs at the [ESI Swagger UI](https://esi.evetech.net/ui/) or use the search endpoints. Some commonly used ones:

- **Tritanium**: type_id `34`
- **PLEX**: type_id `44992`

Region and station IDs are available as constants (see table above).

## License

MIT
