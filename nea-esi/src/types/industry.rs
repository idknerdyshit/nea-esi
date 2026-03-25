use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A character industry job.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A public industry facility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiIndustryFacility {
    pub facility_id: i64,
    pub owner_id: i64,
    pub region_id: i32,
    pub solar_system_id: i32,
    pub type_id: i32,
    #[serde(default)]
    pub tax: Option<f64>,
}

/// Industry system cost indices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiIndustrySystem {
    pub solar_system_id: i32,
    #[serde(default)]
    pub cost_indices: Vec<EsiCostIndex>,
}

/// A cost index for an activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiCostIndex {
    pub activity: String,
    pub cost_index: f64,
}
