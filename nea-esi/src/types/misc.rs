use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// A single item in a character's asset list.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A single wallet journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// LP balance with a corporation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiLoyaltyPoints {
    pub corporation_id: i64,
    pub loyalty_points: i32,
}

/// An LP store offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiLoyaltyStoreOffer {
    pub offer_id: i32,
    pub type_id: i32,
    pub quantity: i32,
    pub lp_cost: i32,
    pub isk_cost: i64,
    #[serde(default)]
    pub ak_cost: Option<i32>,
    #[serde(default)]
    pub required_items: Vec<EsiLoyaltyRequiredItem>,
}

/// A required item for an LP store offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiLoyaltyRequiredItem {
    pub type_id: i32,
    pub quantity: i32,
}

/// A planetary colony summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiPlanetSummary {
    pub solar_system_id: i32,
    pub planet_id: i32,
    pub planet_type: String,
    pub num_pins: i32,
    pub last_update: DateTime<Utc>,
    pub upgrade_level: i32,
    #[serde(default)]
    pub owner_id: Option<i64>,
}

/// Detailed planetary colony layout. Uses `serde_json::Value` for complex
/// nested PI structures; typed access is deferred to a future release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiPlanetDetail {
    #[serde(default)]
    pub links: Vec<serde_json::Value>,
    #[serde(default)]
    pub pins: Vec<serde_json::Value>,
    #[serde(default)]
    pub routes: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiDogmaAttribute {
    pub attribute_id: i32,
    pub name: String,
    pub published: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_id: Option<i32>,
    #[serde(default)]
    pub default_value: f64,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub unit_id: Option<i32>,
    #[serde(default)]
    pub stackable: bool,
    #[serde(default)]
    pub high_is_good: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiDogmaEffect {
    pub effect_id: i32,
    pub name: String,
    pub published: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_id: Option<i32>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub effect_category: Option<i32>,
    #[serde(default)]
    pub is_assistance: bool,
    #[serde(default)]
    pub is_offensive: bool,
    #[serde(default)]
    pub is_warp_safe: bool,
    #[serde(default)]
    pub pre_expression: Option<i32>,
    #[serde(default)]
    pub post_expression: Option<i32>,
    #[serde(default)]
    pub duration_attribute_id: Option<i32>,
    #[serde(default)]
    pub tracking_speed_attribute_id: Option<i32>,
    #[serde(default)]
    pub discharge_attribute_id: Option<i32>,
    #[serde(default)]
    pub range_attribute_id: Option<i32>,
    #[serde(default)]
    pub falloff_attribute_id: Option<i32>,
    #[serde(default)]
    pub modifiers: Vec<EsiDogmaModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiDogmaModifier {
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub effect_id: Option<i32>,
    #[serde(default)]
    pub func: Option<String>,
    #[serde(default)]
    pub modified_attribute_id: Option<i32>,
    #[serde(default)]
    pub modifying_attribute_id: Option<i32>,
    #[serde(default)]
    pub operator: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiDynamicItem {
    pub created_by: i64,
    pub mutator_type_id: i32,
    pub source_type_id: i32,
    #[serde(default)]
    pub dogma_attributes: Vec<EsiDogmaAttributeValue>,
    #[serde(default)]
    pub dogma_effects: Vec<EsiDogmaEffectRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiDogmaAttributeValue {
    pub attribute_id: i32,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiDogmaEffectRef {
    pub effect_id: i32,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiWar {
    pub id: i32,
    pub declared: DateTime<Utc>,
    pub mutual: bool,
    pub open_for_allies: bool,
    pub aggressor: EsiWarParty,
    pub defender: EsiWarParty,
    #[serde(default)]
    pub started: Option<DateTime<Utc>>,
    #[serde(default)]
    pub finished: Option<DateTime<Utc>>,
    #[serde(default)]
    pub retracted: Option<DateTime<Utc>>,
    #[serde(default)]
    pub allies: Vec<EsiWarAlly>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiWarParty {
    pub isk_destroyed: f64,
    pub ships_killed: i32,
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiWarAlly {
    #[serde(default)]
    pub alliance_id: Option<i64>,
    #[serde(default)]
    pub corporation_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFwFactionStats {
    pub faction_id: i32,
    pub pilots: i32,
    pub systems_controlled: i32,
    pub kills: EsiFwTotals,
    pub victory_points: EsiFwTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFwTotals {
    pub last_week: i32,
    pub total: i32,
    pub yesterday: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFwSystem {
    pub solar_system_id: i32,
    pub contested: String,
    pub occupier_faction_id: i32,
    pub owner_faction_id: i32,
    pub victory_points: i32,
    pub victory_points_threshold: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFwLeaderboards {
    pub kills: EsiFwLeaderboardCategory,
    pub victory_points: EsiFwLeaderboardCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFwLeaderboardCategory {
    #[serde(default)]
    pub active_total: Vec<EsiFwLeaderboardEntry>,
    #[serde(default)]
    pub last_week: Vec<EsiFwLeaderboardEntry>,
    #[serde(default)]
    pub yesterday: Vec<EsiFwLeaderboardEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFwLeaderboardEntry {
    pub amount: i32,
    pub id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFwWar {
    pub against_id: i32,
    pub faction_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiInsurancePrice {
    pub type_id: i32,
    #[serde(default)]
    pub levels: Vec<EsiInsuranceLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiInsuranceLevel {
    pub cost: f64,
    pub name: String,
    pub payout: f64,
}

/// A personal mining ledger entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMiningEntry {
    pub date: NaiveDate,
    pub solar_system_id: i32,
    pub type_id: i32,
    pub quantity: i64,
}

/// Character FW leaderboards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFwCharacterLeaderboards {
    pub kills: EsiFwLeaderboardCategory,
    pub victory_points: EsiFwLeaderboardCategory,
}

/// Corporation FW leaderboards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiFwCorporationLeaderboards {
    pub kills: EsiFwLeaderboardCategory,
    pub victory_points: EsiFwLeaderboardCategory,
}
