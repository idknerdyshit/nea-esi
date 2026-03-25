use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCharacterFleet {
    pub fleet_id: i64,
    pub role: String,
    pub squad_id: i64,
    pub wing_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFleetInfo {
    pub fleet_id: i64,
    #[serde(default)]
    pub is_free_move: bool,
    #[serde(default)]
    pub is_registered: bool,
    #[serde(default)]
    pub is_voice_enabled: bool,
    #[serde(default)]
    pub motd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFleetMember {
    pub character_id: i64,
    pub join_time: DateTime<Utc>,
    pub role: String,
    pub role_name: String,
    pub ship_type_id: i32,
    pub solar_system_id: i32,
    pub squad_id: i64,
    pub takes_fleet_warp: bool,
    pub wing_id: i64,
    #[serde(default)]
    pub station_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFleetWing {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub squads: Vec<EsiFleetSquad>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFleetSquad {
    pub id: i64,
    pub name: String,
}

/// Body for updating fleet settings.
#[derive(Debug, Clone, Serialize)]
pub struct EsiFleetUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_free_move: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motd: Option<String>,
}

/// Body for inviting a character to a fleet.
#[derive(Debug, Clone, Serialize)]
pub struct EsiFleetInvitation {
    pub character_id: i64,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squad_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wing_id: Option<i64>,
}

/// Body for moving a fleet member.
#[derive(Debug, Clone, Serialize)]
pub struct EsiFleetMovement {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub squad_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wing_id: Option<i64>,
}

/// Response from creating a fleet wing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFleetWingCreated {
    pub wing_id: i64,
}

/// Response from creating a fleet squad.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFleetSquadCreated {
    pub squad_id: i64,
}

/// Body for naming a fleet wing or squad.
#[derive(Debug, Clone, Serialize)]
pub struct EsiFleetNaming {
    pub name: String,
}
