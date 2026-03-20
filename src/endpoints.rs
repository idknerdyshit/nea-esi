// ESI endpoint methods.

use tracing::debug;

use crate::{
    EsiAllianceInfo, EsiAssetItem, EsiCategoryInfo, EsiCharacterInfo, EsiClient,
    EsiConstellationInfo, EsiCorporationInfo, EsiError, EsiGroupInfo, EsiIncursion, EsiKillmail,
    EsiKillmailRef, EsiMarketGroupInfo, EsiMarketHistoryEntry, EsiMarketOrder, EsiMarketPrice,
    EsiRegionInfo, EsiResolvedIds, EsiResolvedName, EsiSearchResult, EsiServerStatus,
    EsiSolarSystemInfo, EsiSovereigntyCampaign, EsiSovereigntyMap, EsiSovereigntyStructure,
    EsiStargateInfo, EsiStationInfo, EsiStructureInfo, EsiTypeInfo, Result,
};

const RESOLVE_NAMES_CHUNK_SIZE: usize = 1000;
const RESOLVE_IDS_CHUNK_SIZE: usize = 500;

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
            self.base_url, region_id, type_id
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
            self.base_url, region_id, type_id
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
            self.base_url, killmail_id, killmail_hash
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
            self.base_url, killmail_id, killmail_hash
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
        let url = format!("{}/characters/{}/", self.base_url, character_id);
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
        let url = format!("{}/corporations/{}/", self.base_url, corporation_id);
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
        let url = format!("{}/alliances/{}/", self.base_url, alliance_id);
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
        let url = format!("{}/characters/{}/assets/", self.base_url, character_id);
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

        let url = format!("{}/universe/names/", self.base_url);
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
        let url = format!("{}/universe/structures/{}/", self.base_url, structure_id);
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
        let url = format!("{}/markets/prices/", self.base_url);
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

    // -----------------------------------------------------------------------
    // Universe endpoints (Phase 1)
    // -----------------------------------------------------------------------

    /// Fetch detailed information about an inventory type.
    #[tracing::instrument(skip(self))]
    pub async fn get_type(&self, type_id: i32) -> Result<EsiTypeInfo> {
        let url = format!("{}/universe/types/{}/", self.base_url, type_id);
        let resp = self.request(&url).await?;
        let info: EsiTypeInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(type_id, name = %info.name, "get_type complete");
        Ok(info)
    }

    /// List all type IDs (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn list_type_ids(&self) -> Result<Vec<i32>> {
        let url = format!("{}/universe/types/", self.base_url);
        let ids = self.get_paginated::<i32>(&url).await?;
        debug!(count = ids.len(), "list_type_ids complete");
        Ok(ids)
    }

    /// Fetch inventory group info.
    #[tracing::instrument(skip(self))]
    pub async fn get_group(&self, group_id: i32) -> Result<EsiGroupInfo> {
        let url = format!("{}/universe/groups/{}/", self.base_url, group_id);
        let resp = self.request(&url).await?;
        let info: EsiGroupInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(group_id, name = %info.name, "get_group complete");
        Ok(info)
    }

    /// Fetch inventory category info.
    #[tracing::instrument(skip(self))]
    pub async fn get_category(&self, category_id: i32) -> Result<EsiCategoryInfo> {
        let url = format!("{}/universe/categories/{}/", self.base_url, category_id);
        let resp = self.request(&url).await?;
        let info: EsiCategoryInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(category_id, name = %info.name, "get_category complete");
        Ok(info)
    }

    /// Fetch solar system info.
    #[tracing::instrument(skip(self))]
    pub async fn get_system(&self, system_id: i32) -> Result<EsiSolarSystemInfo> {
        let url = format!("{}/universe/systems/{}/", self.base_url, system_id);
        let resp = self.request(&url).await?;
        let info: EsiSolarSystemInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(system_id, name = %info.name, "get_system complete");
        Ok(info)
    }

    /// Fetch constellation info.
    #[tracing::instrument(skip(self))]
    pub async fn get_constellation(&self, constellation_id: i32) -> Result<EsiConstellationInfo> {
        let url = format!(
            "{}/universe/constellations/{}/",
            self.base_url, constellation_id
        );
        let resp = self.request(&url).await?;
        let info: EsiConstellationInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(constellation_id, name = %info.name, "get_constellation complete");
        Ok(info)
    }

    /// Fetch region info.
    #[tracing::instrument(skip(self))]
    pub async fn get_region(&self, region_id: i32) -> Result<EsiRegionInfo> {
        let url = format!("{}/universe/regions/{}/", self.base_url, region_id);
        let resp = self.request(&url).await?;
        let info: EsiRegionInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(region_id, name = %info.name, "get_region complete");
        Ok(info)
    }

    /// Fetch NPC station info.
    #[tracing::instrument(skip(self))]
    pub async fn get_station(&self, station_id: i32) -> Result<EsiStationInfo> {
        let url = format!("{}/universe/stations/{}/", self.base_url, station_id);
        let resp = self.request(&url).await?;
        let info: EsiStationInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(station_id, name = %info.name, "get_station complete");
        Ok(info)
    }

    /// Fetch stargate info.
    #[tracing::instrument(skip(self))]
    pub async fn get_stargate(&self, stargate_id: i32) -> Result<EsiStargateInfo> {
        let url = format!("{}/universe/stargates/{}/", self.base_url, stargate_id);
        let resp = self.request(&url).await?;
        let info: EsiStargateInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(stargate_id, name = %info.name, "get_stargate complete");
        Ok(info)
    }

    /// Resolve names to IDs (reverse of `resolve_names`).
    ///
    /// Automatically chunks requests into batches of 500 (the ESI limit).
    #[tracing::instrument(skip(self, names))]
    pub async fn resolve_ids(&self, names: &[String]) -> Result<EsiResolvedIds> {
        if names.is_empty() {
            return Ok(EsiResolvedIds::default());
        }

        let url = format!("{}/universe/ids/", self.base_url);
        let mut merged = EsiResolvedIds::default();

        for chunk in names.chunks(RESOLVE_IDS_CHUNK_SIZE) {
            let resp = self.request_post(&url, &chunk).await?;
            let resolved: EsiResolvedIds = resp
                .json()
                .await
                .map_err(|e| EsiError::Deserialize(e.to_string()))?;
            merged.merge(resolved);
        }

        debug!("resolve_ids complete");
        Ok(merged)
    }

    // -----------------------------------------------------------------------
    // Market endpoints (Phase 1 additions)
    // -----------------------------------------------------------------------

    /// List all type IDs with active market orders in a region (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn market_type_ids(&self, region_id: i32) -> Result<Vec<i32>> {
        let url = format!("{}/markets/{}/types/", self.base_url, region_id);
        let ids = self.get_paginated::<i32>(&url).await?;
        debug!(count = ids.len(), "market_type_ids complete");
        Ok(ids)
    }

    /// List all market group IDs.
    #[tracing::instrument(skip(self))]
    pub async fn market_group_ids(&self) -> Result<Vec<i32>> {
        let url = format!("{}/markets/groups/", self.base_url);
        let resp = self.request(&url).await?;
        let ids: Vec<i32> = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(count = ids.len(), "market_group_ids complete");
        Ok(ids)
    }

    /// Fetch market group info.
    #[tracing::instrument(skip(self))]
    pub async fn get_market_group(&self, market_group_id: i32) -> Result<EsiMarketGroupInfo> {
        let url = format!("{}/markets/groups/{}/", self.base_url, market_group_id);
        let resp = self.request(&url).await?;
        let info: EsiMarketGroupInfo = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(market_group_id, name = %info.name, "get_market_group complete");
        Ok(info)
    }

    // -----------------------------------------------------------------------
    // Search endpoint (Phase 1)
    // -----------------------------------------------------------------------

    /// Search for entities by name (public, unauthenticated).
    ///
    /// `categories` is a comma-separated list of categories to search
    /// (e.g. `"solar_system,station"`).
    #[tracing::instrument(skip(self))]
    pub async fn search(
        &self,
        search: &str,
        categories: &str,
        strict: bool,
    ) -> Result<EsiSearchResult> {
        let base = format!("{}/search/", self.base_url);
        let strict_str = strict.to_string();
        let url = url::Url::parse_with_params(
            &base,
            &[
                ("search", search),
                ("categories", categories),
                ("strict", &strict_str),
            ],
        )
        .map_err(|e| EsiError::Internal(format!("failed to build search URL: {}", e)))?;

        let resp = self.request(url.as_str()).await?;
        let result: EsiSearchResult = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!("search complete");
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // Killmail listing endpoints (Phase 1)
    // -----------------------------------------------------------------------

    /// Fetch recent killmails for a character (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn character_killmails(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiKillmailRef>> {
        let url = format!(
            "{}/characters/{}/killmails/recent/",
            self.base_url, character_id
        );
        let refs = self.get_paginated::<EsiKillmailRef>(&url).await?;
        debug!(count = refs.len(), "character_killmails complete");
        Ok(refs)
    }

    /// Fetch recent killmails for a corporation (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corporation_killmails(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiKillmailRef>> {
        let url = format!(
            "{}/corporations/{}/killmails/recent/",
            self.base_url, corporation_id
        );
        let refs = self.get_paginated::<EsiKillmailRef>(&url).await?;
        debug!(count = refs.len(), "corporation_killmails complete");
        Ok(refs)
    }

    // -----------------------------------------------------------------------
    // Sovereignty endpoints (Phase 1)
    // -----------------------------------------------------------------------

    /// Fetch the sovereignty map — who owns each system.
    #[tracing::instrument(skip(self))]
    pub async fn sovereignty_map(&self) -> Result<Vec<EsiSovereigntyMap>> {
        let url = format!("{}/sovereignty/map/", self.base_url);
        let resp = self.request(&url).await?;
        let entries: Vec<EsiSovereigntyMap> = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(count = entries.len(), "sovereignty_map complete");
        Ok(entries)
    }

    /// Fetch active sovereignty campaigns.
    #[tracing::instrument(skip(self))]
    pub async fn sovereignty_campaigns(&self) -> Result<Vec<EsiSovereigntyCampaign>> {
        let url = format!("{}/sovereignty/campaigns/", self.base_url);
        let resp = self.request(&url).await?;
        let entries: Vec<EsiSovereigntyCampaign> = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(count = entries.len(), "sovereignty_campaigns complete");
        Ok(entries)
    }

    /// Fetch sovereignty structures.
    #[tracing::instrument(skip(self))]
    pub async fn sovereignty_structures(&self) -> Result<Vec<EsiSovereigntyStructure>> {
        let url = format!("{}/sovereignty/structures/", self.base_url);
        let resp = self.request(&url).await?;
        let entries: Vec<EsiSovereigntyStructure> = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(count = entries.len(), "sovereignty_structures complete");
        Ok(entries)
    }

    // -----------------------------------------------------------------------
    // Incursions endpoint (Phase 1)
    // -----------------------------------------------------------------------

    /// Fetch active incursions.
    #[tracing::instrument(skip(self))]
    pub async fn incursions(&self) -> Result<Vec<EsiIncursion>> {
        let url = format!("{}/incursions/", self.base_url);
        let resp = self.request(&url).await?;
        let entries: Vec<EsiIncursion> = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(count = entries.len(), "incursions complete");
        Ok(entries)
    }

    // -----------------------------------------------------------------------
    // Status endpoint (Phase 1)
    // -----------------------------------------------------------------------

    /// Fetch server status (player count, server version).
    #[tracing::instrument(skip(self))]
    pub async fn server_status(&self) -> Result<EsiServerStatus> {
        let url = format!("{}/status/", self.base_url);
        let resp = self.request(&url).await?;
        let status: EsiServerStatus = resp
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))?;
        debug!(players = status.players, "server_status complete");
        Ok(status)
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
