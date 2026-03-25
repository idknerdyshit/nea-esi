use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::misc::EsiFwTotals;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCharacterInfo {
    pub name: String,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
}

/// Character skills overview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiSkills {
    #[serde(default)]
    pub skills: Vec<EsiSkill>,
    pub total_sp: i64,
    #[serde(default)]
    pub unallocated_sp: Option<i32>,
}

/// A single trained skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiSkill {
    pub skill_id: i32,
    pub trained_skill_level: i32,
    pub active_skill_level: i32,
    pub skillpoints_in_skill: i64,
}

/// A skill queue entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A character's current location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiLocation {
    pub solar_system_id: i32,
    #[serde(default)]
    pub station_id: Option<i64>,
    #[serde(default)]
    pub structure_id: Option<i64>,
}

/// A character's current ship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiShip {
    pub ship_type_id: i32,
    pub ship_item_id: i64,
    pub ship_name: String,
}

/// A character's online status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiOnlineStatus {
    pub online: bool,
    #[serde(default)]
    pub last_login: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_logout: Option<DateTime<Utc>>,
    #[serde(default)]
    pub logins: Option<i32>,
}

/// Character clones info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiClones {
    #[serde(default)]
    pub home_location: Option<EsiCloneLocation>,
    #[serde(default)]
    pub last_clone_jump_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_station_change_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub jump_clones: Vec<EsiJumpClone>,
}

/// A clone home location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCloneLocation {
    pub location_id: i64,
    pub location_type: String,
}

/// A jump clone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiJumpClone {
    pub jump_clone_id: i64,
    pub location_id: i64,
    pub location_type: String,
    #[serde(default)]
    pub implants: Vec<i32>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCompletedOpportunity {
    pub opportunity_id: i32,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiAllianceHistoryEntry {
    pub record_id: i32,
    pub start_date: DateTime<Utc>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCorporationHistoryEntry {
    pub record_id: i32,
    pub start_date: DateTime<Utc>,
    pub corporation_id: i64,
    #[serde(default)]
    pub is_deleted: bool,
}

/// Character affiliation (corporation, alliance, faction).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCharacterAffiliation {
    pub character_id: i64,
    pub corporation_id: i64,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub faction_id: Option<i32>,
}

/// Character portrait URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCharacterPortrait {
    #[serde(default)]
    pub px64: Option<String>,
    #[serde(default)]
    pub px128: Option<String>,
    #[serde(default)]
    pub px256: Option<String>,
    #[serde(default)]
    pub px512: Option<String>,
}

/// Character roles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCharacterRoles {
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub roles_at_hq: Vec<String>,
    #[serde(default)]
    pub roles_at_base: Vec<String>,
    #[serde(default)]
    pub roles_at_other: Vec<String>,
}

/// A character title.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCharacterTitle {
    pub title_id: i32,
    #[serde(default)]
    pub name: Option<String>,
}

/// A standing entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiStanding {
    pub from_id: i64,
    pub from_type: String,
    pub standing: f64,
}

/// A character medal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCharacterMedal {
    pub medal_id: i32,
    pub title: String,
    pub description: String,
    pub corporation_id: i64,
    pub issuer_id: i64,
    pub date: DateTime<Utc>,
    pub reason: String,
    pub status: String,
    #[serde(default)]
    pub graphics: Vec<EsiMedalGraphic>,
}

/// A medal graphic layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMedalGraphic {
    pub part: i32,
    pub layer: i32,
    #[serde(default)]
    pub graphic: Option<String>,
    #[serde(default)]
    pub color: Option<i32>,
}

/// Agent research info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiAgentResearch {
    pub agent_id: i64,
    pub skill_type_id: i32,
    pub started_at: DateTime<Utc>,
    pub points_per_day: f64,
    pub remainder_points: f64,
}

/// Jump fatigue info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFatigue {
    #[serde(default)]
    pub last_jump_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub jump_fatigue_expire_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_update_date: Option<DateTime<Utc>>,
}

/// Character FW stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCharacterFwStats {
    #[serde(default)]
    pub faction_id: Option<i32>,
    #[serde(default)]
    pub enlisted_on: Option<DateTime<Utc>>,
    #[serde(default)]
    pub current_rank: Option<i32>,
    #[serde(default)]
    pub highest_rank: Option<i32>,
    #[serde(default)]
    pub kills: Option<EsiFwTotals>,
    #[serde(default)]
    pub victory_points: Option<EsiFwTotals>,
}
