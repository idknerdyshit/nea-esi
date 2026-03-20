// ESI endpoint methods.

use tracing::debug;

use crate::{
    EsiAllianceInfo, EsiAssetItem, EsiCharacterInfo, EsiClient, EsiCorporationInfo, EsiError,
    EsiKillmail, EsiMarketHistoryEntry, EsiMarketOrder, EsiMarketPrice, EsiResolvedName,
    EsiStructureInfo, Result, BASE_URL,
};

const RESOLVE_NAMES_CHUNK_SIZE: usize = 1000;

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
        let url = format!(
            "{}/markets/{}/history/?type_id={}",
            BASE_URL, region_id, type_id
        );
        let resp = self.request(&url).await?;
        let entries: Vec<EsiMarketHistoryEntry> = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(entries = entries.len(), "market_history complete");
        Ok(entries)
    }

    /// Fetch all market orders for a type in a region, handling pagination.
    #[tracing::instrument(skip(self))]
    pub async fn market_orders(
        &self,
        region_id: i32,
        type_id: i32,
    ) -> Result<Vec<EsiMarketOrder>> {
        let url = format!(
            "{}/markets/{}/orders/?type_id={}&order_type=all",
            BASE_URL, region_id, type_id
        );
        let orders = self.get_paginated::<EsiMarketOrder>(&url).await?;
        debug!(total_orders = orders.len(), "market_orders complete");
        Ok(orders)
    }

    // -----------------------------------------------------------------------
    // Killmail endpoint
    // -----------------------------------------------------------------------

    /// Fetch a single killmail by ID and hash, returning the raw JSON value.
    #[tracing::instrument(skip(self))]
    pub async fn get_killmail(
        &self,
        killmail_id: i64,
        killmail_hash: &str,
    ) -> Result<serde_json::Value> {
        let url = format!(
            "{}/killmails/{}/{}/",
            BASE_URL, killmail_id, killmail_hash
        );
        let resp = self.request(&url).await?;
        let value: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!("get_killmail complete");
        Ok(value)
    }

    /// Fetch a single killmail by ID and hash, returning a typed struct.
    #[tracing::instrument(skip(self))]
    pub async fn get_killmail_typed(
        &self,
        killmail_id: i64,
        killmail_hash: &str,
    ) -> Result<EsiKillmail> {
        let url = format!(
            "{}/killmails/{}/{}/",
            BASE_URL, killmail_id, killmail_hash
        );
        let resp = self.request(&url).await?;
        let km: EsiKillmail = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!("get_killmail_typed complete");
        Ok(km)
    }

    // -----------------------------------------------------------------------
    // Character endpoint
    // -----------------------------------------------------------------------

    /// Fetch character info from ESI.
    #[tracing::instrument(skip(self))]
    pub async fn get_character(&self, character_id: i64) -> Result<EsiCharacterInfo> {
        let url = format!("{}/characters/{}/", BASE_URL, character_id);
        let resp = self.request(&url).await?;
        let info: EsiCharacterInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(character_id, name = %info.name, "get_character complete");
        Ok(info)
    }

    // -----------------------------------------------------------------------
    // Corporation endpoint
    // -----------------------------------------------------------------------

    /// Fetch corporation info from ESI.
    #[tracing::instrument(skip(self))]
    pub async fn get_corporation(&self, corporation_id: i64) -> Result<EsiCorporationInfo> {
        let url = format!("{}/corporations/{}/", BASE_URL, corporation_id);
        let resp = self.request(&url).await?;
        let info: EsiCorporationInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(corporation_id, name = %info.name, "get_corporation complete");
        Ok(info)
    }

    // -----------------------------------------------------------------------
    // Alliance endpoint
    // -----------------------------------------------------------------------

    /// Fetch alliance info from ESI.
    #[tracing::instrument(skip(self))]
    pub async fn get_alliance(&self, alliance_id: i64) -> Result<EsiAllianceInfo> {
        let url = format!("{}/alliances/{}/", BASE_URL, alliance_id);
        let resp = self.request(&url).await?;
        let info: EsiAllianceInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(alliance_id, name = %info.name, "get_alliance complete");
        Ok(info)
    }

    // -----------------------------------------------------------------------
    // Character assets endpoint (authenticated, paginated)
    // -----------------------------------------------------------------------

    /// Fetch all assets for a character, handling pagination.
    #[tracing::instrument(skip(self))]
    pub async fn character_assets(&self, character_id: i64) -> Result<Vec<EsiAssetItem>> {
        let url = format!("{}/characters/{}/assets/", BASE_URL, character_id);
        let items = self.get_paginated::<EsiAssetItem>(&url).await?;
        debug!(total_items = items.len(), "character_assets complete");
        Ok(items)
    }

    // -----------------------------------------------------------------------
    // Universe names endpoint (public, POST)
    // -----------------------------------------------------------------------

    /// Resolve a set of IDs to names and categories.
    ///
    /// Automatically chunks requests into batches of 1000 (the ESI limit).
    #[tracing::instrument(skip(self, ids))]
    pub async fn resolve_names(&self, ids: &[i64]) -> Result<Vec<EsiResolvedName>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}/universe/names/", BASE_URL);
        let mut all_names = Vec::with_capacity(ids.len());

        for chunk in ids.chunks(RESOLVE_NAMES_CHUNK_SIZE) {
            let resp = self.request_post(&url, &chunk).await?;
            let names: Vec<EsiResolvedName> = resp
                .json()
                .await
                .map_err(|e| EsiError::Deserialize(e.to_string()))?;
            all_names.extend(names);
        }

        debug!(count = all_names.len(), "resolve_names complete");
        Ok(all_names)
    }

    // -----------------------------------------------------------------------
    // Structure endpoint (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch info about a player-owned structure.
    #[tracing::instrument(skip(self))]
    pub async fn get_structure(&self, structure_id: i64) -> Result<EsiStructureInfo> {
        let url = format!("{}/universe/structures/{}/", BASE_URL, structure_id);
        let resp = self.request(&url).await?;
        let info: EsiStructureInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(structure_id, name = %info.name, "get_structure complete");
        Ok(info)
    }

    // -----------------------------------------------------------------------
    // Market prices endpoint (public)
    // -----------------------------------------------------------------------

    /// Fetch global average and adjusted prices for all types.
    #[tracing::instrument(skip(self))]
    pub async fn market_prices(&self) -> Result<Vec<EsiMarketPrice>> {
        let url = format!("{}/markets/prices/", BASE_URL);
        let resp = self.request(&url).await?;
        let prices: Vec<EsiMarketPrice> = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(count = prices.len(), "market_prices complete");
        Ok(prices)
    }

    // -----------------------------------------------------------------------
    // Utility
    // -----------------------------------------------------------------------

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
}
