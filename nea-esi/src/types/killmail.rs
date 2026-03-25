use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiKillmail {
    pub killmail_id: i64,
    pub killmail_time: DateTime<Utc>,
    pub solar_system_id: i32,
    pub victim: EsiKillmailVictim,
    #[serde(default)]
    pub attackers: Vec<EsiKillmailAttacker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiKillmailItem {
    pub item_type_id: i32,
    #[serde(default)]
    pub quantity_destroyed: Option<i64>,
    #[serde(default)]
    pub quantity_dropped: Option<i64>,
    pub flag: i32,
    pub singleton: i32,
}

/// A killmail reference (ID + hash) from a character/corporation killmail listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiKillmailRef {
    pub killmail_id: i64,
    pub killmail_hash: String,
}
