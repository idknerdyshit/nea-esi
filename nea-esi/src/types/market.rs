use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMarketHistoryEntry {
    pub date: NaiveDate,
    pub average: f64,
    pub highest: f64,
    pub lowest: f64,
    pub volume: i64,
    pub order_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Global average/adjusted price for a type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiMarketPrice {
    pub type_id: i32,
    #[serde(default)]
    pub average_price: Option<f64>,
    #[serde(default)]
    pub adjusted_price: Option<f64>,
}

/// Market group info.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// A character market order.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
