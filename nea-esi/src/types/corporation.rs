use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::misc::EsiFwTotals;
use super::universe::EsiPosition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorporationInfo {
    pub name: String,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub member_count: Option<i32>,
}

/// A corporation wallet division balance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpWalletDivision {
    pub division: i32,
    pub balance: f64,
}

/// An asset name (from POST /corporations/{}/assets/names/).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiAssetName {
    pub item_id: i64,
    pub name: String,
}

/// An asset location (from POST /corporations/{}/assets/locations/).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiAssetLocation {
    pub item_id: i64,
    pub position: EsiPosition,
}

/// Corporation member titles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpMemberTitle {
    pub character_id: i64,
    #[serde(default)]
    pub titles: Vec<i32>,
}

/// Corporation member roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpMemberRole {
    pub character_id: i64,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub roles_at_hq: Vec<String>,
    #[serde(default)]
    pub roles_at_base: Vec<String>,
    #[serde(default)]
    pub roles_at_other: Vec<String>,
}

/// Corporation member tracking info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpMemberTracking {
    pub character_id: i64,
    #[serde(default)]
    pub location_id: Option<i64>,
    #[serde(default)]
    pub logon_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub logoff_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ship_type_id: Option<i32>,
    #[serde(default)]
    pub start_date: Option<DateTime<Utc>>,
}

/// A corporation-owned structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpStructure {
    pub structure_id: i64,
    pub corporation_id: i64,
    pub system_id: i32,
    pub type_id: i32,
    pub state: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub profile_id: Option<i32>,
    #[serde(default)]
    pub fuel_expires: Option<DateTime<Utc>>,
    #[serde(default)]
    pub state_timer_start: Option<DateTime<Utc>>,
    #[serde(default)]
    pub state_timer_end: Option<DateTime<Utc>>,
    #[serde(default)]
    pub unanchors_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub reinforce_hour: Option<i32>,
    #[serde(default)]
    pub services: Vec<EsiCorpStructureService>,
}

/// A service running on a corporation structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpStructureService {
    pub name: String,
    pub state: String,
}

/// A corporation starbase (POS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpStarbase {
    pub starbase_id: i64,
    pub system_id: i32,
    pub type_id: i32,
    pub state: String,
    #[serde(default)]
    pub moon_id: Option<i32>,
    #[serde(default)]
    pub onlined_since: Option<DateTime<Utc>>,
    #[serde(default)]
    pub reinforced_until: Option<DateTime<Utc>>,
    #[serde(default)]
    pub unanchor_at: Option<DateTime<Utc>>,
}

/// Detailed configuration of a corporation starbase (POS).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpStarbaseDetail {
    pub state: String,
    #[serde(default)]
    pub allow_alliance_members: bool,
    #[serde(default)]
    pub allow_corporation_members: bool,
    #[serde(default)]
    pub use_alliance_standings: bool,
    #[serde(default)]
    pub anchor: Option<String>,
    #[serde(default)]
    pub attack_if_at_war: bool,
    #[serde(default)]
    pub attack_if_other_security_status_dropping: bool,
    #[serde(default)]
    pub attack_security_status_threshold: Option<f64>,
    #[serde(default)]
    pub attack_standing_threshold: Option<f64>,
    #[serde(default)]
    pub fuel_bay_take: Option<String>,
    #[serde(default)]
    pub fuel_bay_view: Option<String>,
    #[serde(default)]
    pub offline: Option<String>,
    #[serde(default)]
    pub online: Option<String>,
    #[serde(default)]
    pub unanchor: Option<String>,
    #[serde(default)]
    pub fuels: Vec<EsiStarbaseFuel>,
}

/// A fuel entry for a starbase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiStarbaseFuel {
    pub type_id: i32,
    pub quantity: i32,
}

/// A contact notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiContactNotification {
    pub notification_id: i64,
    pub sender_character_id: i64,
    pub send_date: DateTime<Utc>,
    pub standing_level: f64,
    #[serde(default)]
    pub message: Option<String>,
}

/// Corporation container audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiContainerLog {
    pub logged_at: DateTime<Utc>,
    pub container_id: i64,
    pub container_type_id: i32,
    pub character_id: i64,
    pub action: String,
    pub location_flag: String,
    pub location_id: i64,
    #[serde(default)]
    pub new_config_bitmask: Option<i32>,
    #[serde(default)]
    pub old_config_bitmask: Option<i32>,
    #[serde(default)]
    pub password_type: Option<String>,
    #[serde(default)]
    pub quantity: Option<i32>,
    #[serde(default)]
    pub type_id: Option<i32>,
}

/// A customs office (POCO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCustomsOffice {
    pub office_id: i64,
    pub system_id: i32,
    #[serde(default)]
    pub reinforce_exit_start: Option<i32>,
    #[serde(default)]
    pub reinforce_exit_end: Option<i32>,
    #[serde(default)]
    pub alliance_tax_rate: Option<f64>,
    #[serde(default)]
    pub corporation_tax_rate: Option<f64>,
    #[serde(default)]
    pub standing_level: Option<String>,
    #[serde(default)]
    pub terrible_standing_tax_rate: Option<f64>,
    #[serde(default)]
    pub bad_standing_tax_rate: Option<f64>,
    #[serde(default)]
    pub neutral_standing_tax_rate: Option<f64>,
    #[serde(default)]
    pub good_standing_tax_rate: Option<f64>,
    #[serde(default)]
    pub excellent_standing_tax_rate: Option<f64>,
    #[serde(default)]
    pub allow_access_with_standings: bool,
    #[serde(default)]
    pub allow_alliance_access: bool,
}

/// Corporation divisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpDivisions {
    #[serde(default)]
    pub hangar: Vec<EsiCorpDivision>,
    #[serde(default)]
    pub wallet: Vec<EsiCorpDivision>,
}

/// A single division entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpDivision {
    pub division: i32,
    #[serde(default)]
    pub name: Option<String>,
}

/// A corporation facility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpFacility {
    pub facility_id: i64,
    pub system_id: i32,
    pub type_id: i32,
}

/// Corporation FW stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpFwStats {
    #[serde(default)]
    pub faction_id: Option<i32>,
    #[serde(default)]
    pub enlisted_on: Option<DateTime<Utc>>,
    #[serde(default)]
    pub pilots: Option<i32>,
    #[serde(default)]
    pub kills: Option<EsiFwTotals>,
    #[serde(default)]
    pub victory_points: Option<EsiFwTotals>,
}

/// Corporation icon URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpIcons {
    #[serde(default)]
    pub px64: Option<String>,
    #[serde(default)]
    pub px128: Option<String>,
    #[serde(default)]
    pub px256: Option<String>,
}

/// A corporation medal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpMedal {
    pub medal_id: i32,
    pub title: String,
    pub description: String,
    pub creator_id: i64,
    pub created_at: DateTime<Utc>,
}

/// An issued medal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiIssuedMedal {
    pub medal_id: i32,
    pub character_id: i64,
    pub issuer_id: i64,
    pub issued_at: DateTime<Utc>,
    pub reason: String,
    pub status: String,
}

/// A role change history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiRoleHistory {
    pub character_id: i64,
    pub changed_at: DateTime<Utc>,
    pub issuer_id: i64,
    pub role_type: String,
    #[serde(default)]
    pub before: Vec<String>,
    #[serde(default)]
    pub after: Vec<String>,
}

/// A shareholder entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiShareholder {
    pub shareholder_id: i64,
    pub shareholder_type: String,
    pub share_count: i64,
}

/// A corporation title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorpTitle {
    pub title_id: i32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub roles_at_hq: Vec<String>,
    #[serde(default)]
    pub roles_at_base: Vec<String>,
    #[serde(default)]
    pub roles_at_other: Vec<String>,
    #[serde(default)]
    pub grantable_roles: Vec<String>,
    #[serde(default)]
    pub grantable_roles_at_hq: Vec<String>,
    #[serde(default)]
    pub grantable_roles_at_base: Vec<String>,
    #[serde(default)]
    pub grantable_roles_at_other: Vec<String>,
}

/// A moon mining extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMiningExtraction {
    pub structure_id: i64,
    pub moon_id: i32,
    pub extraction_start_time: DateTime<Utc>,
    pub chunk_arrival_time: DateTime<Utc>,
    pub natural_decay_time: DateTime<Utc>,
}

/// A mining observer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMiningObserver {
    pub observer_id: i64,
    pub observer_type: String,
    pub last_updated: NaiveDate,
}

/// A mining observer entry (character mining at observer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMiningObserverEntry {
    pub character_id: i64,
    pub recorded_corporation_id: i64,
    pub type_id: i32,
    pub quantity: i64,
    pub last_updated: NaiveDate,
}
