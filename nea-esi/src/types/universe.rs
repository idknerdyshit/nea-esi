use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Player-owned structure info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiStructureInfo {
    pub name: String,
    pub owner_id: i64,
    pub solar_system_id: i32,
    #[serde(default)]
    pub type_id: Option<i32>,
}

/// Detailed information about an inventory type.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiGroupInfo {
    pub group_id: i32,
    pub name: String,
    pub category_id: i32,
    pub published: bool,
    #[serde(default)]
    pub types: Vec<i32>,
}

/// Inventory category info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCategoryInfo {
    pub category_id: i32,
    pub name: String,
    pub published: bool,
    #[serde(default)]
    pub groups: Vec<i32>,
}

/// Solar system info.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiSystemPlanet {
    pub planet_id: i32,
    #[serde(default)]
    pub moons: Vec<i32>,
    #[serde(default)]
    pub asteroid_belts: Vec<i32>,
}

/// Constellation info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiConstellationInfo {
    pub constellation_id: i32,
    pub name: String,
    pub region_id: i32,
    #[serde(default)]
    pub systems: Vec<i32>,
}

/// Region info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiRegionInfo {
    pub region_id: i32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub constellations: Vec<i32>,
}

/// NPC station info.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiStargateInfo {
    pub stargate_id: i32,
    pub name: String,
    pub system_id: i32,
    pub type_id: i32,
    #[serde(default)]
    pub destination: Option<EsiStargateDestination>,
}

/// Stargate destination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiStargateDestination {
    pub stargate_id: i32,
    pub system_id: i32,
}

/// Sovereignty map entry — who owns each system.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// An active incursion.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Server status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiServerStatus {
    pub players: i32,
    #[serde(default)]
    pub server_version: Option<String>,
    #[serde(default)]
    pub start_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub vip: Option<bool>,
}

/// An ancestry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiAncestry {
    pub id: i32,
    pub name: String,
    pub bloodline_id: i32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub short_description: Option<String>,
    #[serde(default)]
    pub icon_id: Option<i32>,
}

/// An asteroid belt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiAsteroidBelt {
    pub name: String,
    pub system_id: i32,
    #[serde(default)]
    pub position: Option<EsiPosition>,
}

/// A 3D position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A bloodline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiBloodline {
    pub bloodline_id: i32,
    pub name: String,
    pub race_id: i32,
    pub corporation_id: i64,
    pub ship_type_id: i32,
    pub charisma: i32,
    pub intelligence: i32,
    pub memory: i32,
    pub perception: i32,
    pub willpower: i32,
    #[serde(default)]
    pub description: Option<String>,
}

/// A faction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFaction {
    pub faction_id: i32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
    #[serde(default)]
    pub militia_corporation_id: Option<i64>,
    #[serde(default)]
    pub solar_system_id: Option<i32>,
    #[serde(default)]
    pub size_factor: Option<f64>,
    #[serde(default)]
    pub station_count: Option<i32>,
    #[serde(default)]
    pub station_system_count: Option<i32>,
    #[serde(default)]
    pub is_unique: bool,
}

/// A graphic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiGraphic {
    pub graphic_id: i32,
    #[serde(default)]
    pub collision_file: Option<String>,
    #[serde(default)]
    pub graphic_file: Option<String>,
    #[serde(default)]
    pub icon_folder: Option<String>,
    #[serde(default)]
    pub sof_dna: Option<String>,
    #[serde(default)]
    pub sof_fation_name: Option<String>,
    #[serde(default)]
    pub sof_hull_name: Option<String>,
    #[serde(default)]
    pub sof_race_name: Option<String>,
}

/// A moon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMoon {
    pub moon_id: i32,
    pub name: String,
    pub system_id: i32,
    #[serde(default)]
    pub position: Option<EsiPosition>,
}

/// A planet (universe data, not PI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiPlanet {
    pub planet_id: i32,
    pub name: String,
    pub system_id: i32,
    pub type_id: i32,
    #[serde(default)]
    pub position: Option<EsiPosition>,
}

/// A race.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiRace {
    pub race_id: i32,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub alliance_id: Option<i64>,
}

/// A PI schematic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiSchematic {
    pub schematic_id: i32,
    pub schematic_name: String,
    pub cycle_time: i32,
}

/// A star.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiStar {
    pub name: String,
    pub solar_system_id: i32,
    pub type_id: i32,
    pub age: i64,
    pub luminosity: f64,
    pub radius: i64,
    pub spectral_class: String,
    pub temperature: i32,
}

/// System jump statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiSystemJumps {
    pub system_id: i32,
    pub ship_jumps: i32,
}

/// System kill statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiSystemKills {
    pub system_id: i32,
    #[serde(default)]
    pub npc_kills: i32,
    #[serde(default)]
    pub pod_kills: i32,
    #[serde(default)]
    pub ship_kills: i32,
}
