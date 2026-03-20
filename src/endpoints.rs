// ESI endpoint methods.

use serde::de::DeserializeOwned;
use tracing::debug;

use crate::{
    EsiAllianceInfo, EsiAssetItem, EsiAttributes, EsiBlueprint, EsiBookmark, EsiBookmarkFolder,
    EsiCalendarEvent, EsiCalendarEventDetail, EsiCategoryInfo, EsiCharacterInfo,
    EsiCharacterOrder, EsiClient, EsiClones, EsiConstellationInfo, EsiContact, EsiContactLabel,
    EsiContract, EsiContractBid, EsiContractItem, EsiCorporationInfo, EsiError, EsiFitting,
    EsiGroupInfo, EsiIncursion, EsiIndustryJob, EsiKillmail, EsiKillmailRef, EsiLocation,
    EsiLoyaltyPoints, EsiLoyaltyStoreOffer, EsiMailBody, EsiMailHeader, EsiMailLabels,
    EsiMarketGroupInfo, EsiMarketHistoryEntry, EsiMarketOrder, EsiMarketPrice, EsiNewFitting,
    EsiNewFittingResponse, EsiNewMail, EsiNotification, EsiOnlineStatus, EsiPlanetDetail,
    EsiPlanetSummary, EsiRegionInfo, EsiResolvedIds, EsiResolvedName, EsiSearchResult,
    EsiServerStatus, EsiShip, EsiSkillqueueEntry, EsiSkills, EsiSolarSystemInfo,
    EsiSovereigntyCampaign, EsiSovereigntyMap, EsiSovereigntyStructure, EsiStargateInfo,
    EsiStationInfo, EsiStructureInfo, EsiTypeInfo, EsiWalletJournalEntry, EsiWalletTransaction,
    Result,
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

    /// POST a path relative to `base_url` with a JSON body, deserialize the response.
    async fn post_json<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl serde::Serialize,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.request_post(&url, body).await?;
        resp.json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))
    }

    /// DELETE a path relative to `base_url`, discarding the response body.
    async fn delete_path(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let _resp = self.request_delete(&url).await?;
        Ok(())
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
    // Industry endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's industry jobs.
    #[tracing::instrument(skip(self))]
    pub async fn character_industry_jobs(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiIndustryJob>> {
        self.get_json(&format!(
            "/characters/{}/industry/jobs/",
            character_id
        ))
        .await
    }

    /// Fetch a character's blueprints (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn character_blueprints(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiBlueprint>> {
        self.get_paginated_json(&format!(
            "/characters/{}/blueprints/",
            character_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Contract endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's contracts (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn character_contracts(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiContract>> {
        self.get_paginated_json(&format!(
            "/characters/{}/contracts/",
            character_id
        ))
        .await
    }

    /// Fetch items in a specific contract.
    #[tracing::instrument(skip(self))]
    pub async fn character_contract_items(
        &self,
        character_id: i64,
        contract_id: i64,
    ) -> Result<Vec<EsiContractItem>> {
        self.get_json(&format!(
            "/characters/{}/contracts/{}/items/",
            character_id, contract_id
        ))
        .await
    }

    /// Fetch bids on a specific auction contract.
    #[tracing::instrument(skip(self))]
    pub async fn character_contract_bids(
        &self,
        character_id: i64,
        contract_id: i64,
    ) -> Result<Vec<EsiContractBid>> {
        self.get_json(&format!(
            "/characters/{}/contracts/{}/bids/",
            character_id, contract_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Character order endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's active market orders.
    #[tracing::instrument(skip(self))]
    pub async fn character_orders(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiCharacterOrder>> {
        self.get_json(&format!("/characters/{}/orders/", character_id))
            .await
    }

    /// Fetch a character's order history (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn character_order_history(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiCharacterOrder>> {
        self.get_paginated_json(&format!(
            "/characters/{}/orders/history/",
            character_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Fitting endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's saved fittings.
    #[tracing::instrument(skip(self))]
    pub async fn character_fittings(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiFitting>> {
        self.get_json(&format!("/characters/{}/fittings/", character_id))
            .await
    }

    /// Create a new fitting for a character. Returns the new fitting ID.
    #[tracing::instrument(skip(self, fitting))]
    pub async fn create_fitting(
        &self,
        character_id: i64,
        fitting: &EsiNewFitting,
    ) -> Result<i64> {
        let result: EsiNewFittingResponse = self
            .post_json(&format!("/characters/{}/fittings/", character_id), fitting)
            .await?;
        Ok(result.fitting_id)
    }

    /// Delete a fitting. Returns `()` on success (204).
    #[tracing::instrument(skip(self))]
    pub async fn delete_fitting(
        &self,
        character_id: i64,
        fitting_id: i64,
    ) -> Result<()> {
        self.delete_path(&format!(
            "/characters/{}/fittings/{}/",
            character_id, fitting_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Location endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's current location.
    #[tracing::instrument(skip(self))]
    pub async fn character_location(
        &self,
        character_id: i64,
    ) -> Result<EsiLocation> {
        self.get_json(&format!("/characters/{}/location/", character_id))
            .await
    }

    /// Fetch a character's current ship.
    #[tracing::instrument(skip(self))]
    pub async fn character_ship(
        &self,
        character_id: i64,
    ) -> Result<EsiShip> {
        self.get_json(&format!("/characters/{}/ship/", character_id))
            .await
    }

    /// Fetch a character's online status.
    #[tracing::instrument(skip(self))]
    pub async fn character_online(
        &self,
        character_id: i64,
    ) -> Result<EsiOnlineStatus> {
        self.get_json(&format!("/characters/{}/online/", character_id))
            .await
    }

    // -----------------------------------------------------------------------
    // Mail endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's mail headers (first batch).
    #[tracing::instrument(skip(self))]
    pub async fn character_mail(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiMailHeader>> {
        self.get_json(&format!("/characters/{}/mail/", character_id))
            .await
    }

    /// Fetch mail headers before a given mail ID (cursor pagination).
    #[tracing::instrument(skip(self))]
    pub async fn character_mail_before(
        &self,
        character_id: i64,
        last_mail_id: i64,
    ) -> Result<Vec<EsiMailHeader>> {
        self.get_json(&format!(
            "/characters/{}/mail/?last_mail_id={}",
            character_id, last_mail_id
        ))
        .await
    }

    /// Fetch a single mail body.
    #[tracing::instrument(skip(self))]
    pub async fn character_mail_body(
        &self,
        character_id: i64,
        mail_id: i64,
    ) -> Result<EsiMailBody> {
        self.get_json(&format!(
            "/characters/{}/mail/{}/",
            character_id, mail_id
        ))
        .await
    }

    /// Send a mail. Returns the new mail ID.
    #[tracing::instrument(skip(self, mail))]
    pub async fn send_mail(
        &self,
        character_id: i64,
        mail: &EsiNewMail,
    ) -> Result<i32> {
        self.post_json(&format!("/characters/{}/mail/", character_id), mail)
            .await
    }

    /// Fetch a character's mail labels.
    #[tracing::instrument(skip(self))]
    pub async fn character_mail_labels(
        &self,
        character_id: i64,
    ) -> Result<EsiMailLabels> {
        self.get_json(&format!(
            "/characters/{}/mail/labels/",
            character_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Notification endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's notifications.
    #[tracing::instrument(skip(self))]
    pub async fn character_notifications(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiNotification>> {
        self.get_json(&format!(
            "/characters/{}/notifications/",
            character_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Contact endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's contacts (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn character_contacts(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiContact>> {
        self.get_paginated_json(&format!(
            "/characters/{}/contacts/",
            character_id
        ))
        .await
    }

    /// Fetch a character's contact labels.
    #[tracing::instrument(skip(self))]
    pub async fn character_contact_labels(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiContactLabel>> {
        self.get_json(&format!(
            "/characters/{}/contacts/labels/",
            character_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Bookmark endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's bookmarks (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn character_bookmarks(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiBookmark>> {
        self.get_paginated_json(&format!(
            "/characters/{}/bookmarks/",
            character_id
        ))
        .await
    }

    /// Fetch a character's bookmark folders.
    #[tracing::instrument(skip(self))]
    pub async fn character_bookmark_folders(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiBookmarkFolder>> {
        self.get_json(&format!(
            "/characters/{}/bookmarks/folders/",
            character_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Calendar endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's upcoming calendar events.
    #[tracing::instrument(skip(self))]
    pub async fn character_calendar(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiCalendarEvent>> {
        self.get_json(&format!(
            "/characters/{}/calendar/",
            character_id
        ))
        .await
    }

    /// Fetch details of a specific calendar event.
    #[tracing::instrument(skip(self))]
    pub async fn character_calendar_event(
        &self,
        character_id: i64,
        event_id: i64,
    ) -> Result<EsiCalendarEventDetail> {
        self.get_json(&format!(
            "/characters/{}/calendar/{}/",
            character_id, event_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Clone endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's clones.
    #[tracing::instrument(skip(self))]
    pub async fn character_clones(
        &self,
        character_id: i64,
    ) -> Result<EsiClones> {
        self.get_json(&format!("/characters/{}/clones/", character_id))
            .await
    }

    /// Fetch a character's active implants.
    #[tracing::instrument(skip(self))]
    pub async fn character_implants(
        &self,
        character_id: i64,
    ) -> Result<Vec<i32>> {
        self.get_json(&format!("/characters/{}/implants/", character_id))
            .await
    }

    // -----------------------------------------------------------------------
    // Loyalty endpoints
    // -----------------------------------------------------------------------

    /// Fetch a character's LP balances.
    #[tracing::instrument(skip(self))]
    pub async fn character_loyalty_points(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiLoyaltyPoints>> {
        self.get_json(&format!(
            "/characters/{}/loyalty/points/",
            character_id
        ))
        .await
    }

    /// Fetch LP store offers for a corporation (public, no auth).
    #[tracing::instrument(skip(self))]
    pub async fn loyalty_store_offers(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiLoyaltyStoreOffer>> {
        self.get_json(&format!(
            "/loyalty/stores/{}/offers/",
            corporation_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // PI endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch a character's planetary colonies.
    #[tracing::instrument(skip(self))]
    pub async fn character_planets(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiPlanetSummary>> {
        self.get_json(&format!("/characters/{}/planets/", character_id))
            .await
    }

    /// Fetch detailed layout of a planetary colony.
    #[tracing::instrument(skip(self))]
    pub async fn character_planet_detail(
        &self,
        character_id: i64,
        planet_id: i32,
    ) -> Result<EsiPlanetDetail> {
        self.get_json(&format!(
            "/characters/{}/planets/{}/",
            character_id, planet_id
        ))
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

        let mut all_names = Vec::with_capacity(ids.len());

        for chunk in ids.chunks(RESOLVE_NAMES_CHUNK_SIZE) {
            let names: Vec<EsiResolvedName> =
                self.post_json("/universe/names/", &chunk).await?;
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

        let mut merged = EsiResolvedIds::default();

        for chunk in names.chunks(RESOLVE_IDS_CHUNK_SIZE) {
            let resolved: EsiResolvedIds =
                self.post_json("/universe/ids/", &chunk).await?;
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
