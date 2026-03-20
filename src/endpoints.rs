// ESI endpoint methods.

use serde::de::DeserializeOwned;
use tracing::debug;

use crate::{
    EsiAllianceInfo, EsiAssetItem, EsiCategoryInfo, EsiCharacterInfo, EsiClient,
    EsiConstellationInfo, EsiCorporationInfo, EsiError, EsiGroupInfo, EsiIncursion, EsiKillmail,
    EsiKillmailRef, EsiMarketGroupInfo, EsiMarketHistoryEntry, EsiMarketOrder, EsiMarketPrice,
    EsiRegionInfo, EsiResolvedIds, EsiResolvedName, EsiSearchResult, EsiServerStatus,
    EsiSolarSystemInfo, EsiSovereigntyCampaign, EsiSovereigntyMap, EsiSovereigntyStructure,
    EsiSkillqueueEntry, EsiSkills, EsiAttributes,
    EsiStargateInfo, EsiStationInfo, EsiStructureInfo, EsiTypeInfo, EsiWalletJournalEntry,
    EsiWalletTransaction, Result,
};

const RESOLVE_NAMES_CHUNK_SIZE: usize = 1000;
const RESOLVE_IDS_CHUNK_SIZE: usize = 500;

impl EsiClient {
    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// GET a path relative to `base_url` and deserialize the JSON response.
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.request(&url).await?;
        resp.json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))
    }

    /// GET a paginated path relative to `base_url` and collect all pages.
    async fn get_paginated_json<T: DeserializeOwned + Send + 'static>(
        &self,
        path: &str,
    ) -> Result<Vec<T>> {
        let url = format!("{}{}", self.base_url, path);
        self.get_paginated::<T>(&url).await
    }

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
    #[tracing::instrument(skip(self))]
    pub async fn market_orders(
        &self,
        region_id: i32,
        type_id: i32,
    ) -> Result<Vec<EsiMarketOrder>> {
        self.get_paginated_json(&format!(
            "/markets/{}/orders/?type_id={}&order_type=all",
            region_id, type_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Killmail endpoints
    // -----------------------------------------------------------------------

    /// Fetch a single killmail by ID and hash, returning the raw JSON value.
    #[tracing::instrument(skip(self))]
    pub async fn get_killmail(
        &self,
        killmail_id: i64,
        killmail_hash: &str,
    ) -> Result<serde_json::Value> {
        self.get_json(&format!("/killmails/{}/{}/", killmail_id, killmail_hash))
            .await
    }

    /// Fetch a single killmail by ID and hash, returning a typed struct.
    #[tracing::instrument(skip(self))]
    pub async fn get_killmail_typed(
        &self,
        killmail_id: i64,
        killmail_hash: &str,
    ) -> Result<EsiKillmail> {
        self.get_json(&format!("/killmails/{}/{}/", killmail_id, killmail_hash))
            .await
    }

    // -----------------------------------------------------------------------
    // Character / Corporation / Alliance endpoints
    // -----------------------------------------------------------------------

    /// Fetch character info from ESI.
    #[tracing::instrument(skip(self))]
    pub async fn get_character(&self, character_id: i64) -> Result<EsiCharacterInfo> {
        self.get_json(&format!("/characters/{}/", character_id)).await
    }

    /// Fetch corporation info from ESI.
    #[tracing::instrument(skip(self))]
    pub async fn get_corporation(&self, corporation_id: i64) -> Result<EsiCorporationInfo> {
        self.get_json(&format!("/corporations/{}/", corporation_id)).await
    }

    /// Fetch alliance info from ESI.
    #[tracing::instrument(skip(self))]
    pub async fn get_alliance(&self, alliance_id: i64) -> Result<EsiAllianceInfo> {
        self.get_json(&format!("/alliances/{}/", alliance_id)).await
    }

    // -----------------------------------------------------------------------
    // Character assets endpoint (authenticated, paginated)
    // -----------------------------------------------------------------------

    /// Fetch all assets for a character, handling pagination.
    #[tracing::instrument(skip(self))]
    pub async fn character_assets(&self, character_id: i64) -> Result<Vec<EsiAssetItem>> {
        self.get_paginated_json(&format!("/characters/{}/assets/", character_id))
            .await
    }

    // -----------------------------------------------------------------------
    // Wallet endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's ISK balance.
    #[tracing::instrument(skip(self))]
    pub async fn wallet_balance(&self, character_id: i64) -> Result<f64> {
        self.get_json(&format!("/characters/{}/wallet/", character_id))
            .await
    }

    /// Fetch a character's wallet journal (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn wallet_journal(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiWalletJournalEntry>> {
        self.get_paginated_json(&format!(
            "/characters/{}/wallet/journal/",
            character_id
        ))
        .await
    }

    /// Fetch a character's wallet transactions.
    #[tracing::instrument(skip(self))]
    pub async fn wallet_transactions(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiWalletTransaction>> {
        self.get_json(&format!(
            "/characters/{}/wallet/transactions/",
            character_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Skill endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's trained skills.
    #[tracing::instrument(skip(self))]
    pub async fn character_skills(&self, character_id: i64) -> Result<EsiSkills> {
        self.get_json(&format!("/characters/{}/skills/", character_id))
            .await
    }

    /// Fetch a character's skill queue.
    #[tracing::instrument(skip(self))]
    pub async fn character_skillqueue(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiSkillqueueEntry>> {
        self.get_json(&format!("/characters/{}/skillqueue/", character_id))
            .await
    }

    /// Fetch a character's attributes.
    #[tracing::instrument(skip(self))]
    pub async fn character_attributes(&self, character_id: i64) -> Result<EsiAttributes> {
        self.get_json(&format!("/characters/{}/attributes/", character_id))
            .await
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
        self.get_json(&format!("/universe/structures/{}/", structure_id))
            .await
    }

    // -----------------------------------------------------------------------
    // Market prices endpoint (public)
    // -----------------------------------------------------------------------

    /// Fetch global average and adjusted prices for all types.
    #[tracing::instrument(skip(self))]
    pub async fn market_prices(&self) -> Result<Vec<EsiMarketPrice>> {
        self.get_json("/markets/prices/").await
    }

    // -----------------------------------------------------------------------
    // Universe endpoints
    // -----------------------------------------------------------------------

    /// Fetch detailed information about an inventory type.
    #[tracing::instrument(skip(self))]
    pub async fn get_type(&self, type_id: i32) -> Result<EsiTypeInfo> {
        self.get_json(&format!("/universe/types/{}/", type_id)).await
    }

    /// List all type IDs (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn list_type_ids(&self) -> Result<Vec<i32>> {
        self.get_paginated_json("/universe/types/").await
    }

    /// Fetch inventory group info.
    #[tracing::instrument(skip(self))]
    pub async fn get_group(&self, group_id: i32) -> Result<EsiGroupInfo> {
        self.get_json(&format!("/universe/groups/{}/", group_id)).await
    }

    /// Fetch inventory category info.
    #[tracing::instrument(skip(self))]
    pub async fn get_category(&self, category_id: i32) -> Result<EsiCategoryInfo> {
        self.get_json(&format!("/universe/categories/{}/", category_id))
            .await
    }

    /// Fetch solar system info.
    #[tracing::instrument(skip(self))]
    pub async fn get_system(&self, system_id: i32) -> Result<EsiSolarSystemInfo> {
        self.get_json(&format!("/universe/systems/{}/", system_id))
            .await
    }

    /// Fetch constellation info.
    #[tracing::instrument(skip(self))]
    pub async fn get_constellation(&self, constellation_id: i32) -> Result<EsiConstellationInfo> {
        self.get_json(&format!("/universe/constellations/{}/", constellation_id))
            .await
    }

    /// Fetch region info.
    #[tracing::instrument(skip(self))]
    pub async fn get_region(&self, region_id: i32) -> Result<EsiRegionInfo> {
        self.get_json(&format!("/universe/regions/{}/", region_id))
            .await
    }

    /// Fetch NPC station info.
    #[tracing::instrument(skip(self))]
    pub async fn get_station(&self, station_id: i32) -> Result<EsiStationInfo> {
        self.get_json(&format!("/universe/stations/{}/", station_id))
            .await
    }

    /// Fetch stargate info.
    #[tracing::instrument(skip(self))]
    pub async fn get_stargate(&self, stargate_id: i32) -> Result<EsiStargateInfo> {
        self.get_json(&format!("/universe/stargates/{}/", stargate_id))
            .await
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
    // Market endpoints (additional)
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Search endpoint
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

        self.request(url.as_str())
            .await?
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))
    }

    // -----------------------------------------------------------------------
    // Killmail listing endpoints
    // -----------------------------------------------------------------------

    /// Fetch recent killmails for a character (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn character_killmails(&self, character_id: i64) -> Result<Vec<EsiKillmailRef>> {
        self.get_paginated_json(&format!(
            "/characters/{}/killmails/recent/",
            character_id
        ))
        .await
    }

    /// Fetch recent killmails for a corporation (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corporation_killmails(&self, corporation_id: i64) -> Result<Vec<EsiKillmailRef>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/killmails/recent/",
            corporation_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Sovereignty endpoints
    // -----------------------------------------------------------------------

    /// Fetch the sovereignty map — who owns each system.
    #[tracing::instrument(skip(self))]
    pub async fn sovereignty_map(&self) -> Result<Vec<EsiSovereigntyMap>> {
        self.get_json("/sovereignty/map/").await
    }

    /// Fetch active sovereignty campaigns.
    #[tracing::instrument(skip(self))]
    pub async fn sovereignty_campaigns(&self) -> Result<Vec<EsiSovereigntyCampaign>> {
        self.get_json("/sovereignty/campaigns/").await
    }

    /// Fetch sovereignty structures.
    #[tracing::instrument(skip(self))]
    pub async fn sovereignty_structures(&self) -> Result<Vec<EsiSovereigntyStructure>> {
        self.get_json("/sovereignty/structures/").await
    }

    // -----------------------------------------------------------------------
    // Incursions endpoint
    // -----------------------------------------------------------------------

    /// Fetch active incursions.
    #[tracing::instrument(skip(self))]
    pub async fn incursions(&self) -> Result<Vec<EsiIncursion>> {
        self.get_json("/incursions/").await
    }

    // -----------------------------------------------------------------------
    // Status endpoint
    // -----------------------------------------------------------------------

    /// Fetch server status (player count, server version).
    #[tracing::instrument(skip(self))]
    pub async fn server_status(&self) -> Result<EsiServerStatus> {
        self.get_json("/status/").await
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
