// nea-esi: Client for the EVE Swagger Interface (ESI) API.

pub mod auth;
mod endpoints;

use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Utc};
use rand::RngExt;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT as USER_AGENT_HEADER};
use secrecy::{ExposeSecret, SecretString};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, warn};

pub use auth::{EsiAppCredentials, EsiTokens, PkceChallenge};
pub use endpoints::compute_best_bid_ask;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const BASE_URL: &str = "https://esi.evetech.net/latest";
pub const THE_FORGE: i32 = 10000002;
pub const DOMAIN: i32 = 10000043;
pub const SINQ_LAISON: i32 = 10000032;
pub const HEIMATAR: i32 = 10000030;
pub const METROPOLIS: i32 = 10000042;
pub const JITA_STATION: i64 = 60003760;
pub const AMARR_STATION: i64 = 60008494;
pub const DODIXIE_STATION: i64 = 60011866;
pub const RENS_STATION: i64 = 60004588;
pub const HEK_STATION: i64 = 60005686;
pub const DEFAULT_USER_AGENT: &str =
    "nea-esi (https://github.com/idknerdyshit/new-eden-analytics)";

const MAX_RETRIES: u32 = 3;
const RETRY_BASE_MS: u64 = 1000;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum EsiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("Rate limited – error budget exhausted")]
    RateLimited,

    #[error("Deserialization error: {0}")]
    Deserialize(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Token refresh error: {0}")]
    TokenRefresh(String),
}

pub type Result<T> = std::result::Result<T, EsiError>;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct EsiMarketHistoryEntry {
    pub date: NaiveDate,
    pub average: f64,
    pub highest: f64,
    pub lowest: f64,
    pub volume: i64,
    pub order_count: i64,
}

// ---------------------------------------------------------------------------
// Killmail types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct EsiKillmail {
    pub killmail_id: i64,
    pub killmail_time: DateTime<Utc>,
    pub solar_system_id: i32,
    pub victim: EsiKillmailVictim,
    #[serde(default)]
    pub attackers: Vec<EsiKillmailAttacker>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiKillmailAttacker {
    #[serde(default)]
    pub character_id: Option<i64>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    pub ship_type_id: i32,
    pub weapon_type_id: i32,
    pub damage_done: i32,
    pub final_blow: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiCharacterInfo {
    pub name: String,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiCorporationInfo {
    pub name: String,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub member_count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiAllianceInfo {
    pub name: String,
    #[serde(default)]
    pub ticker: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiKillmailVictim {
    pub ship_type_id: i32,
    #[serde(default)]
    pub character_id: Option<i64>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub items: Vec<EsiKillmailItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiKillmailItem {
    pub item_type_id: i32,
    #[serde(default)]
    pub quantity_destroyed: Option<i64>,
    #[serde(default)]
    pub quantity_dropped: Option<i64>,
    pub flag: i32,
    pub singleton: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EsiMarketOrder {
    pub order_id: i64,
    pub type_id: i32,
    pub location_id: i64,
    pub price: f64,
    pub volume_remain: i64,
    pub is_buy_order: bool,
    pub issued: DateTime<Utc>,
    pub duration: i32,
    pub min_volume: i32,
    pub range: String,
}

/// A single item in a character's asset list.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiAssetItem {
    pub item_id: i64,
    pub type_id: i32,
    pub location_id: i64,
    pub location_type: String,
    pub location_flag: String,
    pub quantity: i32,
    #[serde(default)]
    pub is_singleton: bool,
    #[serde(default)]
    pub is_blueprint_copy: Option<bool>,
}

/// Resolved name from POST /universe/names/.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiResolvedName {
    pub id: i64,
    pub name: String,
    pub category: String,
}

/// Player-owned structure info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiStructureInfo {
    pub name: String,
    pub owner_id: i64,
    pub solar_system_id: i32,
    #[serde(default)]
    pub type_id: Option<i32>,
}

/// Global average/adjusted price for a type.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiMarketPrice {
    pub type_id: i32,
    #[serde(default)]
    pub average_price: Option<f64>,
    #[serde(default)]
    pub adjusted_price: Option<f64>,
}

// ---------------------------------------------------------------------------
// Universe types (Phase 1)
// ---------------------------------------------------------------------------

/// Detailed information about an inventory type.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiTypeInfo {
    pub type_id: i32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub group_id: i32,
    #[serde(default)]
    pub market_group_id: Option<i32>,
    #[serde(default)]
    pub mass: Option<f64>,
    #[serde(default)]
    pub volume: Option<f64>,
    #[serde(default)]
    pub packaged_volume: Option<f64>,
    #[serde(default)]
    pub capacity: Option<f64>,
    pub published: bool,
    #[serde(default)]
    pub portion_size: Option<i32>,
    #[serde(default)]
    pub icon_id: Option<i32>,
    #[serde(default)]
    pub graphic_id: Option<i32>,
}

/// Inventory group info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiGroupInfo {
    pub group_id: i32,
    pub name: String,
    pub category_id: i32,
    pub published: bool,
    #[serde(default)]
    pub types: Vec<i32>,
}

/// Inventory category info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiCategoryInfo {
    pub category_id: i32,
    pub name: String,
    pub published: bool,
    #[serde(default)]
    pub groups: Vec<i32>,
}

/// Solar system info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiSolarSystemInfo {
    pub system_id: i32,
    pub name: String,
    pub constellation_id: i32,
    pub security_status: f64,
    #[serde(default)]
    pub security_class: Option<String>,
    #[serde(default)]
    pub star_id: Option<i32>,
    #[serde(default)]
    pub stargates: Vec<i32>,
    #[serde(default)]
    pub stations: Vec<i32>,
    #[serde(default)]
    pub planets: Vec<EsiSystemPlanet>,
}

/// A planet within a solar system.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiSystemPlanet {
    pub planet_id: i32,
    #[serde(default)]
    pub moons: Vec<i32>,
    #[serde(default)]
    pub asteroid_belts: Vec<i32>,
}

/// Constellation info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiConstellationInfo {
    pub constellation_id: i32,
    pub name: String,
    pub region_id: i32,
    #[serde(default)]
    pub systems: Vec<i32>,
}

/// Region info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiRegionInfo {
    pub region_id: i32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub constellations: Vec<i32>,
}

/// NPC station info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiStationInfo {
    pub station_id: i32,
    pub name: String,
    pub system_id: i32,
    pub type_id: i32,
    #[serde(default)]
    pub owner: Option<i64>,
    #[serde(default)]
    pub race_id: Option<i32>,
    #[serde(default)]
    pub reprocessing_efficiency: Option<f64>,
    #[serde(default)]
    pub reprocessing_stations_take: Option<f64>,
    #[serde(default)]
    pub office_rental_cost: Option<f64>,
}

/// Stargate info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiStargateInfo {
    pub stargate_id: i32,
    pub name: String,
    pub system_id: i32,
    pub type_id: i32,
    #[serde(default)]
    pub destination: Option<EsiStargateDestination>,
}

/// Stargate destination.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiStargateDestination {
    pub stargate_id: i32,
    pub system_id: i32,
}

/// Result of POST /universe/ids/ — names resolved to IDs.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EsiResolvedIds {
    #[serde(default)]
    pub characters: Vec<EsiIdEntry>,
    #[serde(default)]
    pub corporations: Vec<EsiIdEntry>,
    #[serde(default)]
    pub alliances: Vec<EsiIdEntry>,
    #[serde(default)]
    pub systems: Vec<EsiIdEntry>,
    #[serde(default)]
    pub constellations: Vec<EsiIdEntry>,
    #[serde(default)]
    pub regions: Vec<EsiIdEntry>,
    #[serde(default)]
    pub stations: Vec<EsiIdEntry>,
    #[serde(default)]
    pub inventory_types: Vec<EsiIdEntry>,
    #[serde(default)]
    pub factions: Vec<EsiIdEntry>,
    #[serde(default)]
    pub agents: Vec<EsiIdEntry>,
}

impl EsiResolvedIds {
    /// Merge another `EsiResolvedIds` into this one.
    pub fn merge(&mut self, other: EsiResolvedIds) {
        self.characters.extend(other.characters);
        self.corporations.extend(other.corporations);
        self.alliances.extend(other.alliances);
        self.systems.extend(other.systems);
        self.constellations.extend(other.constellations);
        self.regions.extend(other.regions);
        self.stations.extend(other.stations);
        self.inventory_types.extend(other.inventory_types);
        self.factions.extend(other.factions);
        self.agents.extend(other.agents);
    }
}

/// A single resolved ID + name entry.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiIdEntry {
    pub id: i64,
    pub name: String,
}

// ---------------------------------------------------------------------------
// Market types (Phase 1)
// ---------------------------------------------------------------------------

/// Market group info.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiMarketGroupInfo {
    pub market_group_id: i32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent_group_id: Option<i32>,
    #[serde(default)]
    pub types: Vec<i32>,
}

// ---------------------------------------------------------------------------
// Search types (Phase 1)
// ---------------------------------------------------------------------------

/// Result of GET /search/.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EsiSearchResult {
    #[serde(default)]
    pub character: Vec<i64>,
    #[serde(default)]
    pub corporation: Vec<i64>,
    #[serde(default)]
    pub alliance: Vec<i64>,
    #[serde(default)]
    pub solar_system: Vec<i32>,
    #[serde(default)]
    pub constellation: Vec<i32>,
    #[serde(default)]
    pub region: Vec<i32>,
    #[serde(default)]
    pub station: Vec<i32>,
    #[serde(default)]
    pub inventory_type: Vec<i32>,
    #[serde(default)]
    pub agent: Vec<i64>,
    #[serde(default)]
    pub faction: Vec<i32>,
}

// ---------------------------------------------------------------------------
// Killmail ref types (Phase 1)
// ---------------------------------------------------------------------------

/// A killmail reference (ID + hash) from a character/corporation killmail listing.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiKillmailRef {
    pub killmail_id: i64,
    pub killmail_hash: String,
}

// ---------------------------------------------------------------------------
// Sovereignty types (Phase 1)
// ---------------------------------------------------------------------------

/// Sovereignty map entry — who owns each system.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiSovereigntyMap {
    pub system_id: i32,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub faction_id: Option<i32>,
}

/// Active sovereignty campaign.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiSovereigntyCampaign {
    pub campaign_id: i32,
    pub solar_system_id: i32,
    pub structure_id: i64,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub defender_id: Option<i64>,
    #[serde(default)]
    pub constellation_id: Option<i32>,
}

/// Sovereignty structure.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiSovereigntyStructure {
    #[serde(default)]
    pub alliance_id: Option<i64>,
    pub solar_system_id: i32,
    pub structure_id: i64,
    pub structure_type_id: i32,
    #[serde(default)]
    pub vulnerability_occupancy_level: Option<f64>,
    #[serde(default)]
    pub vulnerable_start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub vulnerable_end_time: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Incursion types (Phase 1)
// ---------------------------------------------------------------------------

/// An active incursion.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiIncursion {
    pub constellation_id: i32,
    #[serde(rename = "type", default)]
    pub incursion_type: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub staging_solar_system_id: Option<i32>,
    #[serde(default)]
    pub influence: Option<f64>,
    #[serde(default)]
    pub has_boss: bool,
    #[serde(default)]
    pub faction_id: Option<i32>,
    #[serde(default)]
    pub infested_solar_systems: Vec<i32>,
}

// ---------------------------------------------------------------------------
// Server status types (Phase 1)
// ---------------------------------------------------------------------------

/// Server status.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiServerStatus {
    pub players: i32,
    #[serde(default)]
    pub server_version: Option<String>,
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub vip: Option<bool>,
}

// ---------------------------------------------------------------------------
// Wallet types (Phase 2)
// ---------------------------------------------------------------------------

/// A single wallet journal entry.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiWalletJournalEntry {
    pub id: i64,
    pub date: DateTime<Utc>,
    pub ref_type: String,
    #[serde(default)]
    pub amount: Option<f64>,
    #[serde(default)]
    pub balance: Option<f64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub first_party_id: Option<i64>,
    #[serde(default)]
    pub second_party_id: Option<i64>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub context_id: Option<i64>,
    #[serde(default)]
    pub context_id_type: Option<String>,
    #[serde(default)]
    pub tax: Option<f64>,
    #[serde(default)]
    pub tax_receiver_id: Option<i64>,
}

/// A single wallet transaction.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiWalletTransaction {
    pub transaction_id: i64,
    pub date: DateTime<Utc>,
    pub type_id: i32,
    pub location_id: i64,
    pub unit_price: f64,
    pub quantity: i32,
    pub client_id: i64,
    pub is_buy: bool,
    pub is_personal: bool,
    pub journal_ref_id: i64,
}

// ---------------------------------------------------------------------------
// Skill types (Phase 2)
// ---------------------------------------------------------------------------

/// Character skills overview.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiSkills {
    #[serde(default)]
    pub skills: Vec<EsiSkill>,
    pub total_sp: i64,
    #[serde(default)]
    pub unallocated_sp: Option<i32>,
}

/// A single trained skill.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiSkill {
    pub skill_id: i32,
    pub trained_skill_level: i32,
    pub active_skill_level: i32,
    pub skillpoints_in_skill: i64,
}

/// A skill queue entry.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiSkillqueueEntry {
    pub skill_id: i32,
    pub finish_level: i32,
    pub queue_position: i32,
    #[serde(default)]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub finish_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub training_start_sp: Option<i32>,
    #[serde(default)]
    pub level_start_sp: Option<i32>,
    #[serde(default)]
    pub level_end_sp: Option<i32>,
}

/// Character attributes.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiAttributes {
    pub intelligence: i32,
    pub memory: i32,
    pub perception: i32,
    pub willpower: i32,
    pub charisma: i32,
    #[serde(default)]
    pub bonus_remaps: Option<i32>,
    #[serde(default)]
    pub last_remap_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub accrued_remap_cooldown_date: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Industry types (Phase 2)
// ---------------------------------------------------------------------------

/// A character industry job.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiIndustryJob {
    pub job_id: i32,
    pub installer_id: i64,
    pub facility_id: i64,
    pub activity_id: i32,
    pub blueprint_id: i64,
    pub blueprint_type_id: i32,
    pub blueprint_location_id: i64,
    pub output_location_id: i64,
    pub runs: i32,
    pub status: String,
    pub duration: i32,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    #[serde(default)]
    pub cost: Option<f64>,
    #[serde(default)]
    pub licensed_runs: Option<i32>,
    #[serde(default)]
    pub probability: Option<f64>,
    #[serde(default)]
    pub product_type_id: Option<i32>,
    #[serde(default)]
    pub pause_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub completed_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub completed_character_id: Option<i64>,
    #[serde(default)]
    pub successful_runs: Option<i32>,
    #[serde(default)]
    pub station_id: Option<i64>,
}

/// A character blueprint.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiBlueprint {
    pub item_id: i64,
    pub type_id: i32,
    pub location_id: i64,
    pub location_flag: String,
    pub quantity: i32,
    pub time_efficiency: i32,
    pub material_efficiency: i32,
    pub runs: i32,
}

// ---------------------------------------------------------------------------
// Contract types (Phase 2)
// ---------------------------------------------------------------------------

/// A character contract.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiContract {
    pub contract_id: i64,
    pub issuer_id: i64,
    pub issuer_corporation_id: i64,
    #[serde(default)]
    pub assignee_id: Option<i64>,
    #[serde(default)]
    pub acceptor_id: Option<i64>,
    #[serde(rename = "type")]
    pub contract_type: String,
    pub status: String,
    pub availability: String,
    pub date_issued: DateTime<Utc>,
    pub date_expired: DateTime<Utc>,
    pub for_corporation: bool,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub date_accepted: Option<DateTime<Utc>>,
    #[serde(default)]
    pub date_completed: Option<DateTime<Utc>>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub reward: Option<f64>,
    #[serde(default)]
    pub collateral: Option<f64>,
    #[serde(default)]
    pub buyout: Option<f64>,
    #[serde(default)]
    pub volume: Option<f64>,
    #[serde(default)]
    pub days_to_complete: Option<i32>,
    #[serde(default)]
    pub start_location_id: Option<i64>,
    #[serde(default)]
    pub end_location_id: Option<i64>,
}

/// An item in a contract.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiContractItem {
    pub record_id: i64,
    pub type_id: i32,
    pub quantity: i32,
    pub is_included: bool,
    #[serde(default)]
    pub is_singleton: Option<bool>,
    #[serde(default)]
    pub raw_quantity: Option<i32>,
}

/// A bid on an auction contract.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiContractBid {
    pub bid_id: i64,
    pub bidder_id: i64,
    pub date_bid: DateTime<Utc>,
    pub amount: f64,
}

// ---------------------------------------------------------------------------
// Character order types (Phase 2)
// ---------------------------------------------------------------------------

/// A character market order.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiCharacterOrder {
    pub order_id: i64,
    pub type_id: i32,
    pub region_id: i32,
    pub location_id: i64,
    pub range: String,
    pub is_buy_order: bool,
    pub price: f64,
    pub volume_total: i32,
    pub volume_remain: i32,
    pub issued: DateTime<Utc>,
    pub min_volume: i32,
    pub duration: i32,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub escrow: Option<f64>,
    #[serde(default)]
    pub is_corporation: Option<bool>,
}

// ---------------------------------------------------------------------------
// Fitting types (Phase 2)
// ---------------------------------------------------------------------------

/// A saved ship fitting.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiFitting {
    pub fitting_id: i64,
    pub name: String,
    pub description: String,
    pub ship_type_id: i32,
    #[serde(default)]
    pub items: Vec<EsiFittingItem>,
}

/// An item in a fitting (used for both GET and POST).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFittingItem {
    pub type_id: i32,
    pub flag: i32,
    pub quantity: i32,
}

/// Body for creating a new fitting.
#[derive(Debug, Clone, Serialize)]
pub struct EsiNewFitting {
    pub name: String,
    pub description: String,
    pub ship_type_id: i32,
    pub items: Vec<EsiFittingItem>,
}

/// Response from creating a fitting.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiNewFittingResponse {
    pub fitting_id: i64,
}

// ---------------------------------------------------------------------------
// Location types (Phase 2)
// ---------------------------------------------------------------------------

/// A character's current location.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiLocation {
    pub solar_system_id: i32,
    #[serde(default)]
    pub station_id: Option<i64>,
    #[serde(default)]
    pub structure_id: Option<i64>,
}

/// A character's current ship.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiShip {
    pub ship_type_id: i32,
    pub ship_item_id: i64,
    pub ship_name: String,
}

/// A character's online status.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiOnlineStatus {
    pub online: bool,
    #[serde(default)]
    pub last_login: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_logout: Option<DateTime<Utc>>,
    #[serde(default)]
    pub logins: Option<i32>,
}

// ---------------------------------------------------------------------------
// Mail types (Phase 2)
// ---------------------------------------------------------------------------

/// A mail header from a character's inbox.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiMailHeader {
    pub mail_id: i64,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub from: Option<i64>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub is_read: Option<bool>,
    #[serde(default)]
    pub labels: Vec<i32>,
    #[serde(default)]
    pub recipients: Vec<EsiMailRecipient>,
}

/// A mail recipient (used in both GET and POST).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMailRecipient {
    pub recipient_id: i64,
    pub recipient_type: String,
}

/// A mail body.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiMailBody {
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub from: Option<i64>,
    #[serde(default)]
    pub read: Option<bool>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    #[serde(default)]
    pub labels: Vec<i32>,
    #[serde(default)]
    pub recipients: Vec<EsiMailRecipient>,
}

/// Body for sending a new mail.
#[derive(Debug, Clone, Serialize)]
pub struct EsiNewMail {
    pub recipients: Vec<EsiMailRecipient>,
    pub subject: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_cost: Option<i64>,
}

/// Character mail labels.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiMailLabels {
    pub total_unread_count: i32,
    #[serde(default)]
    pub labels: Vec<EsiMailLabel>,
}

/// A single mail label.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiMailLabel {
    pub label_id: i32,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub unread_count: Option<i32>,
}

// ---------------------------------------------------------------------------
// Notification types (Phase 2)
// ---------------------------------------------------------------------------

/// A character notification.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiNotification {
    pub notification_id: i64,
    #[serde(rename = "type")]
    pub notification_type: String,
    pub sender_id: i64,
    pub sender_type: String,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub is_read: Option<bool>,
    #[serde(default)]
    pub text: Option<String>,
}

// ---------------------------------------------------------------------------
// Contact types (Phase 2)
// ---------------------------------------------------------------------------

/// A character contact.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiContact {
    pub contact_id: i64,
    pub contact_type: String,
    pub standing: f64,
    #[serde(default)]
    pub label_ids: Vec<i64>,
    #[serde(default)]
    pub is_watched: Option<bool>,
}

/// A contact label.
#[derive(Debug, Clone, Deserialize)]
pub struct EsiContactLabel {
    pub label_id: i64,
    pub label_name: String,
}

// ---------------------------------------------------------------------------
// ETag cache
// ---------------------------------------------------------------------------

struct CachedResponse {
    etag: String,
    body: Vec<u8>,
}

// ---------------------------------------------------------------------------
// EsiClient
// ---------------------------------------------------------------------------

pub struct EsiClient {
    pub(crate) client: reqwest::Client,
    pub(crate) semaphore: Arc<tokio::sync::Semaphore>,
    pub(crate) error_budget: Arc<AtomicI32>,
    /// Unix epoch (seconds) at which the error budget resets.
    pub(crate) error_budget_reset_at: Arc<AtomicU64>,
    pub(crate) tokens: Arc<tokio::sync::RwLock<Option<EsiTokens>>>,
    pub(crate) app_credentials: Option<EsiAppCredentials>,
    cache: Option<Arc<RwLock<HashMap<String, CachedResponse>>>>,
    base_url: String,
}

impl EsiClient {
    /// Create a new ESI client with the default User-Agent and 30-second timeout.
    pub fn new() -> Self {
        Self::with_user_agent(DEFAULT_USER_AGENT)
    }

    /// Create a new ESI client with a custom User-Agent string and 30-second timeout.
    ///
    /// ESI requires a descriptive User-Agent. Include your app name, contact info,
    /// and optionally your EVE character name. Example:
    ///
    /// ```text
    /// my-app (contact@example.com; +https://github.com/me/my-app; eve:MyCharacter)
    /// ```
    pub fn with_user_agent(user_agent: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT_HEADER,
            HeaderValue::from_str(user_agent).expect("invalid user-agent string"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            semaphore: Arc::new(tokio::sync::Semaphore::new(20)),
            error_budget: Arc::new(AtomicI32::new(100)),
            error_budget_reset_at: Arc::new(AtomicU64::new(0)),
            tokens: Arc::new(tokio::sync::RwLock::new(None)),
            app_credentials: None,
            cache: None,
            base_url: BASE_URL.to_string(),
        }
    }

    /// Create an ESI client configured for a web application (confidential client).
    pub fn with_web_app(user_agent: &str, client_id: &str, client_secret: SecretString) -> Self {
        let mut client = Self::with_user_agent(user_agent);
        client.app_credentials = Some(EsiAppCredentials::Web {
            client_id: client_id.to_string(),
            client_secret,
        });
        client
    }

    /// Create an ESI client configured for a native/desktop application (public client).
    pub fn with_native_app(user_agent: &str, client_id: &str) -> Self {
        let mut client = Self::with_user_agent(user_agent);
        client.app_credentials = Some(EsiAppCredentials::Native {
            client_id: client_id.to_string(),
        });
        client
    }

    /// Set app credentials (builder pattern).
    pub fn credentials(mut self, creds: EsiAppCredentials) -> Self {
        self.app_credentials = Some(creds);
        self
    }

    /// Override the base URL (builder pattern). Useful for testing with mock servers.
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Return the current error budget value.
    pub fn error_budget(&self) -> i32 {
        self.error_budget.load(Ordering::Relaxed)
    }

    /// Read `X-ESI-Error-Limit-Remain` and `X-ESI-Error-Limit-Reset` from
    /// response headers, updating the stored error budget and reset deadline.
    fn update_error_budget(&self, headers: &reqwest::header::HeaderMap) {
        if let Some(val) = headers.get("x-esi-error-limit-remain")
            && let Ok(s) = val.to_str()
            && let Ok(remain) = s.parse::<i32>()
        {
            self.error_budget.store(remain, Ordering::Relaxed);
        }
        if let Some(val) = headers.get("x-esi-error-limit-reset")
            && let Ok(s) = val.to_str()
            && let Ok(secs) = s.parse::<u64>()
        {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            self.error_budget_reset_at
                .store(now + secs, Ordering::Relaxed);
        }
    }

    /// When the error budget is low, sleep until the reset window instead of a
    /// flat delay. Falls back to 60 s if no reset header was ever received.
    async fn wait_for_budget_reset(&self) {
        let budget = self.error_budget.load(Ordering::Relaxed);
        if budget < 20 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let reset_at = self.error_budget_reset_at.load(Ordering::Relaxed);
            let wait_secs = if reset_at > now {
                reset_at - now
            } else {
                // No reset header seen yet – fall back to a conservative wait.
                60
            };
            warn!(
                budget,
                wait_secs, "ESI error budget low – sleeping until reset"
            );
            tokio::time::sleep(Duration::from_secs(wait_secs)).await;
        }
    }

    /// Enable the ETag response cache (builder pattern).
    pub fn with_cache(mut self) -> Self {
        self.cache = Some(Arc::new(RwLock::new(HashMap::new())));
        self
    }

    /// Clear all cached ETag responses.
    pub async fn clear_cache(&self) {
        if let Some(ref cache) = self.cache {
            cache.write().await.clear();
        }
    }

    /// Create a lightweight clone that shares all Arc-wrapped state.
    pub(crate) fn clone_shared(&self) -> Self {
        Self {
            client: self.client.clone(),
            semaphore: Arc::clone(&self.semaphore),
            error_budget: Arc::clone(&self.error_budget),
            error_budget_reset_at: Arc::clone(&self.error_budget_reset_at),
            tokens: Arc::clone(&self.tokens),
            app_credentials: self.app_credentials.clone(),
            cache: self.cache.as_ref().map(Arc::clone),
            base_url: self.base_url.clone(),
        }
    }

    // -----------------------------------------------------------------------
    // Pagination
    // -----------------------------------------------------------------------

    /// Fetch all pages of a paginated GET endpoint and flatten into one Vec.
    pub async fn get_paginated<T: DeserializeOwned + Send + 'static>(
        &self,
        base_url: &str,
    ) -> Result<Vec<T>> {
        self.paginated_fetch(base_url, PageFetcher::Get).await
    }

    /// Fetch all pages of a paginated POST endpoint and flatten into one Vec.
    pub async fn post_paginated<T, B>(
        &self,
        base_url: &str,
        body: &B,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned + Send + 'static,
        B: Serialize + Sync,
    {
        let body_value = serde_json::to_value(body)
            .map_err(|e| EsiError::Internal(format!("failed to serialize body: {}", e)))?;
        self.paginated_fetch(base_url, PageFetcher::Post(Arc::new(body_value)))
            .await
    }

    /// Shared pagination logic for both GET and POST.
    async fn paginated_fetch<T: DeserializeOwned + Send + 'static>(
        &self,
        base_url: &str,
        fetcher: PageFetcher,
    ) -> Result<Vec<T>> {
        let separator = if base_url.contains('?') { '&' } else { '?' };
        let first_url = format!("{}{}page=1", base_url, separator);

        let resp = match &fetcher {
            PageFetcher::Get => self.request(&first_url).await?,
            PageFetcher::Post(body) => self.request_post(&first_url, body.as_ref()).await?,
        };

        let total_pages: i32 = resp
            .headers()
            .get("x-pages")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let mut items: Vec<T> = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;

        if total_pages > 1 {
            let mut handles = Vec::with_capacity((total_pages - 1) as usize);
            for page in 2..=total_pages {
                let url = format!("{}{}page={}", base_url, separator, page);
                let this = self.clone_shared();
                let fetcher = fetcher.clone();
                handles.push(tokio::spawn(async move {
                    let resp = match &fetcher {
                        PageFetcher::Get => this.request(&url).await?,
                        PageFetcher::Post(body) => {
                            this.request_post(&url, body.as_ref()).await?
                        }
                    };
                    let page_items: Vec<T> = resp
                        .json()
                        .await
                        .map_err(|e| EsiError::Deserialize(e.to_string()))?;
                    Ok::<_, EsiError>(page_items)
                }));
            }

            for handle in handles {
                let page_items = handle
                    .await
                    .map_err(|e| EsiError::Deserialize(e.to_string()))??;
                items.extend(page_items);
            }
        }

        Ok(items)
    }

    // -----------------------------------------------------------------------
    // ETag caching
    // -----------------------------------------------------------------------

    /// Make a GET request with ETag caching support.
    ///
    /// Uses `execute_request` internally for retry/401 handling. On 304,
    /// returns the cached body. On 200, caches the response.
    pub async fn request_cached(&self, url: &str) -> Result<Vec<u8>> {
        let cached_etag = if let Some(ref cache) = self.cache {
            let guard = cache.read().await;
            guard.get(url).map(|c| c.etag.clone())
        } else {
            None
        };

        let etag_clone = cached_etag.clone();
        let result = self
            .execute_request(url, move |client, url| {
                let mut req = client.get(url);
                if let Some(ref etag) = etag_clone {
                    req = req.header("If-None-Match", etag.as_str());
                }
                req
            })
            .await;

        // Handle 304 Not Modified by returning cached body.
        if let Err(EsiError::Api { status: 304, .. }) = &result
            && let Some(ref cache) = self.cache
        {
            let guard = cache.read().await;
            if let Some(cached) = guard.get(url) {
                debug!(url, "ETag cache hit (304)");
                return Ok(cached.body.clone());
            }
        }

        let response = result?;

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let body = response.bytes().await.map_err(EsiError::Http)?.to_vec();

        if let (Some(cache), Some(etag)) = (&self.cache, etag) {
            let mut guard = cache.write().await;
            guard.insert(
                url.to_string(),
                CachedResponse {
                    etag,
                    body: body.clone(),
                },
            );
        }

        Ok(body)
    }

    // -----------------------------------------------------------------------
    // Core request helpers
    // -----------------------------------------------------------------------

    /// Unified request executor with semaphore, budget check, auth, retry
    /// (502-504 and network errors), and 401 token refresh.
    async fn execute_request(
        &self,
        url: &str,
        build_request: impl Fn(&reqwest::Client, &str) -> reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| EsiError::Internal("rate-limit semaphore closed".into()))?;

        self.wait_for_budget_reset().await;

        if self.error_budget.load(Ordering::Relaxed) <= 0 {
            return Err(EsiError::RateLimited);
        }

        let token = self.ensure_valid_token().await?;
        let start = std::time::Instant::now();

        // Retry loop for transient 502/503/504 errors and network errors.
        let response = {
            let mut last_err = None;
            let mut resp = None;
            for attempt in 0..=MAX_RETRIES {
                let mut req = build_request(&self.client, url);
                if let Some(ref tok) = token {
                    req = req.bearer_auth(tok.expose_secret());
                }
                match req.send().await {
                    Ok(r) => {
                        self.update_error_budget(r.headers());
                        let status = r.status().as_u16();
                        if matches!(status, 502..=504) && attempt < MAX_RETRIES {
                            let jitter = rand::rng().random_range(0..500);
                            let delay = RETRY_BASE_MS * 2u64.pow(attempt) + jitter;
                            warn!(url, status, attempt, delay_ms = delay, "retrying transient error");
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                            continue;
                        }
                        resp = Some(r);
                        break;
                    }
                    Err(e) => {
                        if attempt < MAX_RETRIES {
                            let jitter = rand::rng().random_range(0..500);
                            let delay = RETRY_BASE_MS * 2u64.pow(attempt) + jitter;
                            warn!(url, attempt, delay_ms = delay, error = %e, "retrying network error");
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                            continue;
                        }
                        last_err = Some(e);
                        break;
                    }
                }
            }
            match resp {
                Some(r) => r,
                None => return Err(EsiError::Http(last_err.unwrap())),
            }
        };

        // If 401 and we have tokens, try refreshing once and retry.
        if response.status().as_u16() == 401 && token.is_some() {
            debug!("got 401, attempting token refresh and retry");
            let refreshed = self.refresh_token().await?;
            let retry_resp = build_request(&self.client, url)
                .bearer_auth(refreshed.access_token.expose_secret())
                .send()
                .await?;

            self.update_error_budget(retry_resp.headers());

            if !retry_resp.status().is_success() {
                let status = retry_resp.status().as_u16();
                let message = retry_resp.text().await.unwrap_or_default();
                warn!(url, status, "ESI API error after token refresh retry");
                return Err(EsiError::Api { status, message });
            }

            debug!(
                url,
                status = retry_resp.status().as_u16(),
                elapsed_ms = start.elapsed().as_millis() as u64,
                "ESI request (after 401 retry)"
            );

            return Ok(retry_resp);
        }

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            warn!(url, status, "ESI API error");
            return Err(EsiError::Api { status, message });
        }

        debug!(
            url,
            status = response.status().as_u16(),
            elapsed_ms = start.elapsed().as_millis() as u64,
            error_budget = self.error_budget.load(Ordering::Relaxed),
            "ESI request"
        );

        Ok(response)
    }

    /// Make a rate-limited GET request to the given URL.
    pub async fn request(&self, url: &str) -> Result<reqwest::Response> {
        self.execute_request(url, |client, url| client.get(url))
            .await
    }

    /// Make a rate-limited POST request with a JSON body.
    pub async fn request_post(
        &self,
        url: &str,
        body: &impl Serialize,
    ) -> Result<reqwest::Response> {
        let body_value = serde_json::to_value(body)
            .map_err(|e| EsiError::Internal(format!("failed to serialize body: {}", e)))?;
        self.execute_request(url, move |client, url| client.post(url).json(&body_value))
            .await
    }

    /// Make a rate-limited DELETE request.
    pub async fn request_delete(&self, url: &str) -> Result<reqwest::Response> {
        self.execute_request(url, |client, url| client.delete(url))
            .await
    }
}

/// Internal enum to dispatch between GET and POST in paginated fetches.
#[derive(Clone)]
enum PageFetcher {
    Get,
    Post(Arc<serde_json::Value>),
}

impl Default for EsiClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_order(
        order_id: i64,
        location_id: i64,
        price: f64,
        volume_remain: i64,
        is_buy: bool,
    ) -> EsiMarketOrder {
        EsiMarketOrder {
            order_id,
            type_id: 34,
            location_id,
            price,
            volume_remain,
            is_buy_order: is_buy,
            issued: "2026-01-01T00:00:00Z".parse().unwrap(),
            duration: 90,
            min_volume: 1,
            range: "station".to_string(),
        }
    }

    #[test]
    fn test_compute_best_bid_ask_empty() {
        let (bid, ask, bv, av) = compute_best_bid_ask(&[], JITA_STATION);
        assert_eq!((bid, ask, bv, av), (None, None, 0, 0));
    }

    #[test]
    fn test_compute_best_bid_ask_wrong_station() {
        let orders = vec![make_order(1, 99999, 10.0, 100, true)];
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!((bid, ask, bv, av), (None, None, 0, 0));
    }

    #[test]
    fn test_compute_best_bid_ask_buys_only() {
        let orders = vec![
            make_order(1, JITA_STATION, 10.0, 100, true),
            make_order(2, JITA_STATION, 12.0, 200, true),
        ];
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!(bid, Some(12.0));
        assert_eq!(ask, None);
        assert_eq!(bv, 300);
        assert_eq!(av, 0);
    }

    #[test]
    fn test_compute_best_bid_ask_sells_only() {
        let orders = vec![
            make_order(1, JITA_STATION, 15.0, 50, false),
            make_order(2, JITA_STATION, 13.0, 75, false),
        ];
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!(bid, None);
        assert_eq!(ask, Some(13.0));
        assert_eq!(bv, 0);
        assert_eq!(av, 125);
    }

    #[test]
    fn test_compute_best_bid_ask_mixed() {
        let orders = vec![
            make_order(1, JITA_STATION, 10.0, 100, true),
            make_order(2, JITA_STATION, 12.0, 200, true),
            make_order(3, JITA_STATION, 15.0, 50, false),
            make_order(4, JITA_STATION, 13.0, 75, false),
        ];
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!(bid, Some(12.0));
        assert_eq!(ask, Some(13.0));
        assert_eq!(bv, 300);
        assert_eq!(av, 125);
    }

    #[test]
    fn test_compute_best_bid_ask_multi_station() {
        let amarr: i64 = 60008494;
        let orders = vec![
            make_order(1, JITA_STATION, 10.0, 100, true),
            make_order(2, amarr, 99.0, 999, true),
            make_order(3, JITA_STATION, 15.0, 50, false),
            make_order(4, amarr, 1.0, 999, false),
        ];
        let (bid, ask, bv, av) = compute_best_bid_ask(&orders, JITA_STATION);
        assert_eq!(bid, Some(10.0));
        assert_eq!(ask, Some(15.0));
        assert_eq!(bv, 100);
        assert_eq!(av, 50);
    }

    #[test]
    fn test_deserialize_esi_killmail() {
        let json = r#"{
            "killmail_id": 123456,
            "killmail_time": "2026-03-17T12:00:00Z",
            "solar_system_id": 30000142,
            "victim": {
                "ship_type_id": 587,
                "character_id": 91234567,
                "corporation_id": 98000001,
                "alliance_id": null,
                "items": [
                    {
                        "item_type_id": 2032,
                        "quantity_destroyed": 1,
                        "quantity_dropped": null,
                        "flag": 27,
                        "singleton": 0
                    },
                    {
                        "item_type_id": 3170,
                        "quantity_destroyed": null,
                        "quantity_dropped": 5,
                        "flag": 11,
                        "singleton": 0
                    }
                ]
            },
            "attackers": [
                {
                    "character_id": 95000001,
                    "corporation_id": 98000002,
                    "ship_type_id": 24690,
                    "weapon_type_id": 3170,
                    "damage_done": 5000,
                    "final_blow": true
                },
                {
                    "corporation_id": 1000125,
                    "ship_type_id": 0,
                    "weapon_type_id": 0,
                    "damage_done": 100,
                    "final_blow": false
                }
            ]
        }"#;

        let km: EsiKillmail = serde_json::from_str(json).unwrap();
        assert_eq!(km.killmail_id, 123456);
        assert_eq!(
            km.killmail_time,
            "2026-03-17T12:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
        assert_eq!(km.solar_system_id, 30000142);
        assert_eq!(km.victim.ship_type_id, 587);
        assert_eq!(km.victim.character_id, Some(91234567));
        assert_eq!(km.victim.alliance_id, None);
        assert_eq!(km.victim.items.len(), 2);
        assert_eq!(km.victim.items[0].item_type_id, 2032);
        assert_eq!(km.victim.items[0].quantity_destroyed, Some(1));
        assert_eq!(km.victim.items[1].item_type_id, 3170);
        assert_eq!(km.victim.items[1].quantity_dropped, Some(5));
        assert_eq!(km.attackers.len(), 2);
        assert_eq!(km.attackers[0].character_id, Some(95000001));
        assert_eq!(km.attackers[0].ship_type_id, 24690);
        assert_eq!(km.attackers[0].damage_done, 5000);
        assert!(km.attackers[0].final_blow);
        assert_eq!(km.attackers[1].character_id, None);
        assert!(!km.attackers[1].final_blow);
    }

    #[test]
    fn test_deserialize_esi_killmail_minimal() {
        let json = r#"{
            "killmail_id": 999,
            "killmail_time": "2026-01-01T00:00:00Z",
            "solar_system_id": 30000001,
            "victim": {
                "ship_type_id": 670
            }
        }"#;

        let km: EsiKillmail = serde_json::from_str(json).unwrap();
        assert_eq!(km.killmail_id, 999);
        assert_eq!(km.victim.ship_type_id, 670);
        assert!(km.victim.items.is_empty());
        assert_eq!(km.victim.character_id, None);
    }

    #[test]
    fn test_deserialize_market_history_entry() {
        let json = r#"{"date":"2026-03-01","average":5.25,"highest":5.27,"lowest":5.11,"volume":72016862,"order_count":2267}"#;
        let entry: EsiMarketHistoryEntry = serde_json::from_str(json).unwrap();
        assert_eq!(
            entry.date,
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
        );
        assert!((entry.average - 5.25).abs() < f64::EPSILON);
        assert_eq!(entry.volume, 72016862);
        assert_eq!(entry.order_count, 2267);
    }

    #[test]
    fn test_deserialize_esi_asset_item() {
        let json = r#"{
            "item_id": 1234567890,
            "type_id": 587,
            "location_id": 60003760,
            "location_type": "station",
            "location_flag": "Hangar",
            "quantity": 1,
            "is_singleton": true,
            "is_blueprint_copy": null
        }"#;
        let item: EsiAssetItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.item_id, 1234567890);
        assert_eq!(item.type_id, 587);
        assert_eq!(item.location_id, 60003760);
        assert_eq!(item.location_type, "station");
        assert_eq!(item.location_flag, "Hangar");
        assert_eq!(item.quantity, 1);
        assert!(item.is_singleton);
        assert_eq!(item.is_blueprint_copy, None);
    }

    #[test]
    fn test_deserialize_esi_resolved_name() {
        let json = r#"{"id": 95465499, "name": "CCP Bartender", "category": "character"}"#;
        let name: EsiResolvedName = serde_json::from_str(json).unwrap();
        assert_eq!(name.id, 95465499);
        assert_eq!(name.name, "CCP Bartender");
        assert_eq!(name.category, "character");
    }

    #[test]
    fn test_deserialize_esi_structure_info() {
        let json = r#"{
            "name": "My Citadel",
            "owner_id": 98000001,
            "solar_system_id": 30000142,
            "type_id": 35832
        }"#;
        let info: EsiStructureInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "My Citadel");
        assert_eq!(info.owner_id, 98000001);
        assert_eq!(info.solar_system_id, 30000142);
        assert_eq!(info.type_id, Some(35832));
    }

    #[test]
    fn test_deserialize_esi_market_price() {
        let json = r#"{"type_id": 34, "average_price": 5.25}"#;
        let price: EsiMarketPrice = serde_json::from_str(json).unwrap();
        assert_eq!(price.type_id, 34);
        assert!((price.average_price.unwrap() - 5.25).abs() < f64::EPSILON);
        assert_eq!(price.adjusted_price, None);
    }

    #[test]
    fn test_deserialize_market_order() {
        let json = r#"{"order_id":6789012345,"type_id":34,"location_id":60003760,"price":5.13,"volume_remain":250000,"is_buy_order":true,"issued":"2026-03-10T08:15:00Z","duration":90,"min_volume":1,"range":"station"}"#;
        let order: EsiMarketOrder = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_id, 6789012345);
        assert_eq!(order.type_id, 34);
        assert!(order.is_buy_order);
        assert_eq!(order.location_id, JITA_STATION);
    }

    // -----------------------------------------------------------------------
    // Phase 1 deserialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_deserialize_type_info() {
        let json = r#"{
            "type_id": 587,
            "name": "Rifter",
            "description": "A Minmatar frigate.",
            "group_id": 25,
            "market_group_id": 61,
            "mass": 1067000.0,
            "volume": 27289.0,
            "packaged_volume": 2500.0,
            "capacity": 130.0,
            "published": true,
            "portion_size": 1,
            "icon_id": 587,
            "graphic_id": 46
        }"#;
        let info: EsiTypeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.type_id, 587);
        assert_eq!(info.name, "Rifter");
        assert_eq!(info.group_id, 25);
        assert_eq!(info.market_group_id, Some(61));
        assert!(info.published);
    }

    #[test]
    fn test_deserialize_type_info_minimal() {
        let json = r#"{"type_id": 34, "name": "Tritanium", "group_id": 18, "published": true}"#;
        let info: EsiTypeInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.type_id, 34);
        assert_eq!(info.name, "Tritanium");
        assert_eq!(info.group_id, 18);
        assert!(info.published);
        assert_eq!(info.market_group_id, None);
    }

    #[test]
    fn test_deserialize_group_info() {
        let json = r#"{
            "group_id": 25,
            "name": "Frigate",
            "category_id": 6,
            "published": true,
            "types": [587, 603, 608]
        }"#;
        let info: EsiGroupInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.group_id, 25);
        assert_eq!(info.name, "Frigate");
        assert_eq!(info.category_id, 6);
        assert_eq!(info.types.len(), 3);
    }

    #[test]
    fn test_deserialize_category_info() {
        let json = r#"{
            "category_id": 6,
            "name": "Ship",
            "published": true,
            "groups": [25, 26, 27]
        }"#;
        let info: EsiCategoryInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.category_id, 6);
        assert_eq!(info.name, "Ship");
        assert_eq!(info.groups.len(), 3);
    }

    #[test]
    fn test_deserialize_solar_system_info() {
        let json = r#"{
            "system_id": 30000142,
            "name": "Jita",
            "constellation_id": 20000020,
            "security_status": 0.9459131,
            "security_class": "B",
            "star_id": 40009081,
            "stargates": [50001248, 50001249],
            "stations": [60003760],
            "planets": [
                {"planet_id": 40009082, "moons": [40009083], "asteroid_belts": []}
            ]
        }"#;
        let info: EsiSolarSystemInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.system_id, 30000142);
        assert_eq!(info.name, "Jita");
        assert!((info.security_status - 0.9459131).abs() < 0.0001);
        assert_eq!(info.stargates.len(), 2);
        assert_eq!(info.planets.len(), 1);
        assert_eq!(info.planets[0].planet_id, 40009082);
        assert_eq!(info.planets[0].moons, vec![40009083]);
    }

    #[test]
    fn test_deserialize_constellation_info() {
        let json = r#"{
            "constellation_id": 20000020,
            "name": "Kimotoro",
            "region_id": 10000002,
            "systems": [30000142, 30000143]
        }"#;
        let info: EsiConstellationInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.constellation_id, 20000020);
        assert_eq!(info.name, "Kimotoro");
        assert_eq!(info.systems.len(), 2);
    }

    #[test]
    fn test_deserialize_region_info() {
        let json = r#"{
            "region_id": 10000002,
            "name": "The Forge",
            "description": "Home of Jita",
            "constellations": [20000020, 20000021]
        }"#;
        let info: EsiRegionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.region_id, 10000002);
        assert_eq!(info.name, "The Forge");
        assert_eq!(info.constellations.len(), 2);
    }

    #[test]
    fn test_deserialize_station_info() {
        let json = r#"{
            "station_id": 60003760,
            "name": "Jita IV - Moon 4 - Caldari Navy Assembly Plant",
            "system_id": 30000142,
            "type_id": 52678,
            "owner": 1000035,
            "race_id": 1,
            "reprocessing_efficiency": 0.5,
            "reprocessing_stations_take": 0.05,
            "office_rental_cost": 1234567.89
        }"#;
        let info: EsiStationInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.station_id, 60003760);
        assert_eq!(info.system_id, 30000142);
        assert_eq!(info.owner, Some(1000035));
    }

    #[test]
    fn test_deserialize_stargate_info() {
        let json = r#"{
            "stargate_id": 50001248,
            "name": "Stargate (Perimeter)",
            "system_id": 30000142,
            "type_id": 29624,
            "destination": {"stargate_id": 50001249, "system_id": 30000144}
        }"#;
        let info: EsiStargateInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.stargate_id, 50001248);
        assert_eq!(info.destination.as_ref().unwrap().system_id, 30000144);
    }

    #[test]
    fn test_deserialize_resolved_ids() {
        let json = r#"{
            "characters": [{"id": 95465499, "name": "CCP Bartender"}],
            "systems": [{"id": 30000142, "name": "Jita"}]
        }"#;
        let resolved: EsiResolvedIds = serde_json::from_str(json).unwrap();
        assert_eq!(resolved.characters.len(), 1);
        assert_eq!(resolved.characters[0].id, 95465499);
        assert_eq!(resolved.systems.len(), 1);
        assert!(resolved.corporations.is_empty());
    }

    #[test]
    fn test_deserialize_market_group_info() {
        let json = r#"{
            "market_group_id": 61,
            "name": "Frigates",
            "description": "Small ships",
            "parent_group_id": 4,
            "types": [587, 603]
        }"#;
        let info: EsiMarketGroupInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.market_group_id, 61);
        assert_eq!(info.name, "Frigates");
        assert_eq!(info.parent_group_id, Some(4));
        assert_eq!(info.types.len(), 2);
    }

    #[test]
    fn test_deserialize_search_result() {
        let json = r#"{
            "solar_system": [30000142],
            "station": [60003760, 60003761]
        }"#;
        let result: EsiSearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.solar_system, vec![30000142]);
        assert_eq!(result.station.len(), 2);
        assert!(result.character.is_empty());
    }

    #[test]
    fn test_deserialize_killmail_ref() {
        let json = r#"{"killmail_id": 123456789, "killmail_hash": "abc123def456"}"#;
        let km: EsiKillmailRef = serde_json::from_str(json).unwrap();
        assert_eq!(km.killmail_id, 123456789);
        assert_eq!(km.killmail_hash, "abc123def456");
    }

    #[test]
    fn test_deserialize_sovereignty_map() {
        let json = r#"{"system_id": 30000001, "alliance_id": 99000001, "corporation_id": 98000001, "faction_id": null}"#;
        let entry: EsiSovereigntyMap = serde_json::from_str(json).unwrap();
        assert_eq!(entry.system_id, 30000001);
        assert_eq!(entry.alliance_id, Some(99000001));
        assert_eq!(entry.faction_id, None);
    }

    #[test]
    fn test_deserialize_sovereignty_campaign() {
        let json = r#"{"campaign_id": 1, "solar_system_id": 30000001, "structure_id": 1234567890, "event_type": "tcu_defense"}"#;
        let campaign: EsiSovereigntyCampaign = serde_json::from_str(json).unwrap();
        assert_eq!(campaign.campaign_id, 1);
        assert_eq!(campaign.event_type, Some("tcu_defense".to_string()));
    }

    #[test]
    fn test_deserialize_sovereignty_structure() {
        let json = r#"{"alliance_id": 99000001, "solar_system_id": 30000001, "structure_id": 1234567890, "structure_type_id": 32226}"#;
        let s: EsiSovereigntyStructure = serde_json::from_str(json).unwrap();
        assert_eq!(s.alliance_id, Some(99000001));
        assert_eq!(s.structure_type_id, 32226);
    }

    #[test]
    fn test_deserialize_incursion() {
        let json = r#"{
            "constellation_id": 20000020,
            "type": "Incursion",
            "state": "established",
            "staging_solar_system_id": 30000142,
            "influence": 0.5,
            "has_boss": true,
            "faction_id": 500019,
            "infested_solar_systems": [30000142, 30000143]
        }"#;
        let inc: EsiIncursion = serde_json::from_str(json).unwrap();
        assert_eq!(inc.constellation_id, 20000020);
        assert_eq!(inc.incursion_type, Some("Incursion".to_string()));
        assert_eq!(inc.state, Some("established".to_string()));
        assert!(inc.has_boss);
        assert_eq!(inc.infested_solar_systems.len(), 2);
    }

    #[test]
    fn test_deserialize_server_status() {
        let json = r#"{"players": 23456, "server_version": "2345678", "start_time": "2026-03-20T11:00:00Z", "vip": false}"#;
        let status: EsiServerStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.players, 23456);
        assert_eq!(status.server_version, Some("2345678".to_string()));
        assert_eq!(
            status.start_time,
            Some("2026-03-20T11:00:00Z".parse::<DateTime<Utc>>().unwrap())
        );
        assert_eq!(status.vip, Some(false));
    }

    #[test]
    fn test_deserialize_server_status_minimal() {
        let json = r#"{"players": 100}"#;
        let status: EsiServerStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.players, 100);
        assert_eq!(status.server_version, None);
        assert_eq!(status.vip, None);
    }

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Wallet
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Industry, Contracts, Orders
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Fittings, Location
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Mail, Notifications, Contacts
    // -----------------------------------------------------------------------

    #[test]
    fn test_deserialize_mail_header() {
        let json = r#"{
            "mail_id": 123456,
            "timestamp": "2026-03-15T10:30:00Z",
            "from": 91234567,
            "subject": "Hello",
            "is_read": false,
            "labels": [1, 3],
            "recipients": [{"recipient_id": 92345678, "recipient_type": "character"}]
        }"#;
        let header: EsiMailHeader = serde_json::from_str(json).unwrap();
        assert_eq!(header.mail_id, 123456);
        assert_eq!(header.from, Some(91234567));
        assert_eq!(header.subject, Some("Hello".to_string()));
        assert_eq!(header.labels, vec![1, 3]);
        assert_eq!(header.recipients.len(), 1);
    }

    #[test]
    fn test_deserialize_mail_body() {
        let json = r#"{
            "body": "<p>Hello world</p>",
            "from": 91234567,
            "read": true,
            "subject": "Hello",
            "timestamp": "2026-03-15T10:30:00Z",
            "labels": [1],
            "recipients": [{"recipient_id": 92345678, "recipient_type": "character"}]
        }"#;
        let body: EsiMailBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.body, Some("<p>Hello world</p>".to_string()));
        assert_eq!(body.read, Some(true));
    }

    #[test]
    fn test_deserialize_mail_labels() {
        let json = r##"{
            "total_unread_count": 5,
            "labels": [{"label_id": 1, "name": "Inbox", "color": "#ffffff", "unread_count": 3}]
        }"##;
        let labels: EsiMailLabels = serde_json::from_str(json).unwrap();
        assert_eq!(labels.total_unread_count, 5);
        assert_eq!(labels.labels.len(), 1);
        assert_eq!(labels.labels[0].name, "Inbox");
    }

    #[test]
    fn test_deserialize_notification() {
        let json = r#"{
            "notification_id": 999888,
            "type": "StructureUnderAttack",
            "sender_id": 1000125,
            "sender_type": "corporation",
            "timestamp": "2026-03-15T10:30:00Z",
            "is_read": false,
            "text": "structureID: 1234567890"
        }"#;
        let notif: EsiNotification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.notification_id, 999888);
        assert_eq!(notif.notification_type, "StructureUnderAttack");
        assert_eq!(notif.sender_type, "corporation");
        assert_eq!(notif.is_read, Some(false));
    }

    #[test]
    fn test_deserialize_contact() {
        let json = r#"{
            "contact_id": 91234567,
            "contact_type": "character",
            "standing": 10.0,
            "label_ids": [1, 2],
            "is_watched": true
        }"#;
        let contact: EsiContact = serde_json::from_str(json).unwrap();
        assert_eq!(contact.contact_id, 91234567);
        assert_eq!(contact.contact_type, "character");
        assert!((contact.standing - 10.0).abs() < f64::EPSILON);
        assert_eq!(contact.label_ids, vec![1, 2]);
        assert_eq!(contact.is_watched, Some(true));
    }

    #[test]
    fn test_deserialize_contact_label() {
        let json = r#"{"label_id": 1, "label_name": "Blues"}"#;
        let label: EsiContactLabel = serde_json::from_str(json).unwrap();
        assert_eq!(label.label_id, 1);
        assert_eq!(label.label_name, "Blues");
    }

    #[test]
    fn test_deserialize_fitting() {
        let json = r#"{
            "fitting_id": 12345,
            "name": "PvP Rifter",
            "description": "Standard PvP fit",
            "ship_type_id": 587,
            "items": [
                {"type_id": 2032, "flag": 11, "quantity": 1},
                {"type_id": 3170, "flag": 12, "quantity": 1}
            ]
        }"#;
        let fit: EsiFitting = serde_json::from_str(json).unwrap();
        assert_eq!(fit.fitting_id, 12345);
        assert_eq!(fit.name, "PvP Rifter");
        assert_eq!(fit.ship_type_id, 587);
        assert_eq!(fit.items.len(), 2);
        assert_eq!(fit.items[0].type_id, 2032);
    }

    #[test]
    fn test_deserialize_location() {
        let json = r#"{"solar_system_id": 30000142, "station_id": 60003760}"#;
        let loc: EsiLocation = serde_json::from_str(json).unwrap();
        assert_eq!(loc.solar_system_id, 30000142);
        assert_eq!(loc.station_id, Some(60003760));
        assert_eq!(loc.structure_id, None);
    }

    #[test]
    fn test_deserialize_ship() {
        let json = r#"{"ship_type_id": 587, "ship_item_id": 1234567890, "ship_name": "My Rifter"}"#;
        let ship: EsiShip = serde_json::from_str(json).unwrap();
        assert_eq!(ship.ship_type_id, 587);
        assert_eq!(ship.ship_name, "My Rifter");
    }

    #[test]
    fn test_deserialize_online_status() {
        let json = r#"{
            "online": true,
            "last_login": "2026-03-20T10:00:00Z",
            "last_logout": "2026-03-19T22:00:00Z",
            "logins": 500
        }"#;
        let status: EsiOnlineStatus = serde_json::from_str(json).unwrap();
        assert!(status.online);
        assert!(status.last_login.is_some());
        assert_eq!(status.logins, Some(500));
    }

    #[test]
    fn test_deserialize_industry_job() {
        let json = r#"{
            "job_id": 123,
            "installer_id": 91234567,
            "facility_id": 60003760,
            "activity_id": 1,
            "blueprint_id": 1234567890,
            "blueprint_type_id": 687,
            "blueprint_location_id": 60003760,
            "output_location_id": 60003760,
            "runs": 10,
            "status": "active",
            "duration": 3600,
            "start_date": "2026-03-15T10:00:00Z",
            "end_date": "2026-03-15T11:00:00Z",
            "cost": 1500.50,
            "product_type_id": 687
        }"#;
        let job: EsiIndustryJob = serde_json::from_str(json).unwrap();
        assert_eq!(job.job_id, 123);
        assert_eq!(job.activity_id, 1);
        assert_eq!(job.status, "active");
        assert_eq!(job.runs, 10);
        assert!((job.cost.unwrap() - 1500.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_blueprint() {
        let json = r#"{
            "item_id": 1234567890,
            "type_id": 687,
            "location_id": 60003760,
            "location_flag": "Hangar",
            "quantity": -2,
            "time_efficiency": 20,
            "material_efficiency": 10,
            "runs": 100
        }"#;
        let bp: EsiBlueprint = serde_json::from_str(json).unwrap();
        assert_eq!(bp.item_id, 1234567890);
        assert_eq!(bp.type_id, 687);
        assert_eq!(bp.quantity, -2);
        assert_eq!(bp.time_efficiency, 20);
        assert_eq!(bp.material_efficiency, 10);
    }

    #[test]
    fn test_deserialize_contract() {
        let json = r#"{
            "contract_id": 123456,
            "issuer_id": 91234567,
            "issuer_corporation_id": 98000001,
            "type": "item_exchange",
            "status": "outstanding",
            "availability": "personal",
            "date_issued": "2026-03-15T10:00:00Z",
            "date_expired": "2026-03-29T10:00:00Z",
            "for_corporation": false,
            "title": "Selling stuff",
            "price": 1000000.0,
            "start_location_id": 60003760,
            "end_location_id": 60003760
        }"#;
        let c: EsiContract = serde_json::from_str(json).unwrap();
        assert_eq!(c.contract_id, 123456);
        assert_eq!(c.contract_type, "item_exchange");
        assert_eq!(c.status, "outstanding");
        assert!(!c.for_corporation);
        assert_eq!(c.title, Some("Selling stuff".to_string()));
        assert!((c.price.unwrap() - 1000000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_contract_item() {
        let json = r#"{
            "record_id": 999,
            "type_id": 34,
            "quantity": 100000,
            "is_included": true,
            "is_singleton": false
        }"#;
        let item: EsiContractItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.record_id, 999);
        assert_eq!(item.type_id, 34);
        assert_eq!(item.quantity, 100000);
        assert!(item.is_included);
    }

    #[test]
    fn test_deserialize_contract_bid() {
        let json = r#"{
            "bid_id": 555,
            "bidder_id": 91234567,
            "date_bid": "2026-03-16T12:00:00Z",
            "amount": 5000000.0
        }"#;
        let bid: EsiContractBid = serde_json::from_str(json).unwrap();
        assert_eq!(bid.bid_id, 555);
        assert!((bid.amount - 5000000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_character_order() {
        let json = r#"{
            "order_id": 6789012345,
            "type_id": 34,
            "region_id": 10000002,
            "location_id": 60003760,
            "range": "station",
            "is_buy_order": true,
            "price": 5.13,
            "volume_total": 500000,
            "volume_remain": 250000,
            "issued": "2026-03-10T08:15:00Z",
            "min_volume": 1,
            "duration": 90,
            "escrow": 1282500.0,
            "is_corporation": false
        }"#;
        let order: EsiCharacterOrder = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_id, 6789012345);
        assert!(order.is_buy_order);
        assert_eq!(order.volume_total, 500000);
        assert_eq!(order.volume_remain, 250000);
        assert_eq!(order.state, None);
        assert!((order.escrow.unwrap() - 1282500.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_deserialize_wallet_journal_entry_full() {
        let json = r#"{
            "id": 123456789,
            "date": "2026-03-15T10:30:00Z",
            "ref_type": "market_transaction",
            "amount": -1500000.50,
            "balance": 98500000.00,
            "description": "Market: Tritanium",
            "first_party_id": 91234567,
            "second_party_id": 92345678,
            "reason": "For the lulz",
            "context_id": 6789012345,
            "context_id_type": "market_transaction_id",
            "tax": 15000.00,
            "tax_receiver_id": 1000035
        }"#;
        let entry: EsiWalletJournalEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, 123456789);
        assert_eq!(entry.ref_type, "market_transaction");
        assert!((entry.amount.unwrap() - (-1500000.50)).abs() < f64::EPSILON);
        assert!((entry.balance.unwrap() - 98500000.00).abs() < f64::EPSILON);
        assert_eq!(entry.description, Some("Market: Tritanium".to_string()));
        assert_eq!(entry.first_party_id, Some(91234567));
        assert_eq!(entry.second_party_id, Some(92345678));
        assert_eq!(entry.context_id_type, Some("market_transaction_id".to_string()));
        assert_eq!(entry.tax_receiver_id, Some(1000035));
    }

    #[test]
    fn test_deserialize_wallet_journal_entry_minimal() {
        let json = r#"{
            "id": 999,
            "date": "2026-01-01T00:00:00Z",
            "ref_type": "player_donation"
        }"#;
        let entry: EsiWalletJournalEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.id, 999);
        assert_eq!(entry.ref_type, "player_donation");
        assert_eq!(entry.amount, None);
        assert_eq!(entry.description, None);
    }

    // -----------------------------------------------------------------------
    // Phase 2 deserialization tests — Skills
    // -----------------------------------------------------------------------

    #[test]
    fn test_deserialize_skills() {
        let json = r#"{
            "skills": [
                {"skill_id": 3300, "trained_skill_level": 5, "active_skill_level": 5, "skillpoints_in_skill": 256000}
            ],
            "total_sp": 50000000,
            "unallocated_sp": 100000
        }"#;
        let skills: EsiSkills = serde_json::from_str(json).unwrap();
        assert_eq!(skills.total_sp, 50000000);
        assert_eq!(skills.unallocated_sp, Some(100000));
        assert_eq!(skills.skills.len(), 1);
        assert_eq!(skills.skills[0].skill_id, 3300);
        assert_eq!(skills.skills[0].trained_skill_level, 5);
    }

    #[test]
    fn test_deserialize_skillqueue_entry() {
        let json = r#"{
            "skill_id": 3300,
            "finish_level": 5,
            "queue_position": 0,
            "start_date": "2026-03-15T10:00:00Z",
            "finish_date": "2026-03-20T10:00:00Z",
            "training_start_sp": 45255,
            "level_start_sp": 45255,
            "level_end_sp": 256000
        }"#;
        let entry: EsiSkillqueueEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.skill_id, 3300);
        assert_eq!(entry.finish_level, 5);
        assert_eq!(entry.queue_position, 0);
        assert!(entry.start_date.is_some());
        assert_eq!(entry.level_end_sp, Some(256000));
    }

    #[test]
    fn test_deserialize_attributes() {
        let json = r#"{
            "intelligence": 20,
            "memory": 20,
            "perception": 20,
            "willpower": 20,
            "charisma": 19,
            "bonus_remaps": 1,
            "last_remap_date": "2025-01-01T00:00:00Z"
        }"#;
        let attrs: EsiAttributes = serde_json::from_str(json).unwrap();
        assert_eq!(attrs.intelligence, 20);
        assert_eq!(attrs.charisma, 19);
        assert_eq!(attrs.bonus_remaps, Some(1));
        assert!(attrs.last_remap_date.is_some());
        assert_eq!(attrs.accrued_remap_cooldown_date, None);
    }

    #[test]
    fn test_deserialize_wallet_transaction() {
        let json = r#"{
            "transaction_id": 5678901234,
            "date": "2026-03-15T10:30:00Z",
            "type_id": 34,
            "location_id": 60003760,
            "unit_price": 5.25,
            "quantity": 100000,
            "client_id": 91234567,
            "is_buy": true,
            "is_personal": true,
            "journal_ref_id": 123456789
        }"#;
        let tx: EsiWalletTransaction = serde_json::from_str(json).unwrap();
        assert_eq!(tx.transaction_id, 5678901234);
        assert_eq!(tx.type_id, 34);
        assert_eq!(tx.location_id, JITA_STATION);
        assert!((tx.unit_price - 5.25).abs() < f64::EPSILON);
        assert_eq!(tx.quantity, 100000);
        assert!(tx.is_buy);
        assert!(tx.is_personal);
    }
}
