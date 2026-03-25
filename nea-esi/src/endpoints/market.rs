use crate::{EsiClient, EsiMarketGroupInfo, EsiMarketHistoryEntry, EsiMarketOrder, EsiMarketPrice, Result};

impl EsiClient {
    // -----------------------------------------------------------------------
    // Market endpoints
    // -----------------------------------------------------------------------

    /// Fetch market history for a type in a region.
    #[tracing::instrument(skip(self))]
    pub async fn market_history(
        &self,
        region_id: i32,
        type_id: i32,
    ) -> Result<Vec<EsiMarketHistoryEntry>> {
        self.get_json(&format!("/markets/{}/history/?type_id={}", region_id, type_id))
            .await
    }

    /// Fetch all market orders for a type in a region, handling pagination.
    ///
    /// `order_type` can be `"all"` (default), `"buy"`, or `"sell"`.
    #[tracing::instrument(skip(self))]
    pub async fn market_orders(
        &self,
        region_id: i32,
        type_id: i32,
        order_type: Option<&str>,
    ) -> Result<Vec<EsiMarketOrder>> {
        let ot = order_type.unwrap_or("all");
        self.get_paginated_json(&format!(
            "/markets/{}/orders/?type_id={}&order_type={}",
            region_id, type_id, ot
        ))
        .await
    }

    /// Fetch global average and adjusted prices for all types.
    #[tracing::instrument(skip(self))]
    pub async fn market_prices(&self) -> Result<Vec<EsiMarketPrice>> {
        self.get_json("/markets/prices/").await
    }

    /// List all type IDs with active market orders in a region (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn market_type_ids(&self, region_id: i32) -> Result<Vec<i32>> {
        self.get_paginated_json(&format!("/markets/{}/types/", region_id))
            .await
    }

    /// List all market group IDs.
    #[tracing::instrument(skip(self))]
    pub async fn market_group_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/markets/groups/").await
    }

    /// Fetch market group info.
    #[tracing::instrument(skip(self))]
    pub async fn get_market_group(&self, market_group_id: i32) -> Result<EsiMarketGroupInfo> {
        self.get_json(&format!("/markets/groups/{}/", market_group_id))
            .await
    }

    /// Fetch market orders at a structure (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn structure_orders(
        &self,
        structure_id: i64,
    ) -> Result<Vec<EsiMarketOrder>> {
        self.get_paginated_json(&format!(
            "/markets/structures/{}/",
            structure_id
        ))
        .await
    }
}

/// Given a slice of market orders, filter to a specific station and compute
/// best bid, best ask, total bid volume, and total ask volume.
///
/// Returns `(best_bid, best_ask, bid_volume, ask_volume)`.
pub fn compute_best_bid_ask(
    orders: &[EsiMarketOrder],
    station_id: i64,
) -> (Option<f64>, Option<f64>, i64, i64) {
    let mut best_bid: Option<f64> = None;
    let mut best_ask: Option<f64> = None;
    let mut bid_volume: i64 = 0;
    let mut ask_volume: i64 = 0;

    for order in orders.iter().filter(|o| o.location_id == station_id) {
        if order.is_buy_order {
            bid_volume += order.volume_remain;
            best_bid = Some(match best_bid {
                Some(current) => current.max(order.price),
                None => order.price,
            });
        } else {
            ask_volume += order.volume_remain;
            best_ask = Some(match best_ask {
                Some(current) => current.min(order.price),
                None => order.price,
            });
        }
    }

    (best_bid, best_ask, bid_volume, ask_volume)
}
