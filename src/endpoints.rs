// ESI endpoint methods.

use serde::de::DeserializeOwned;
use tracing::debug;

use crate::{
    EsiAgentResearch, EsiAllianceHistoryEntry, EsiAllianceIcons, EsiAllianceInfo, EsiAncestry,
    EsiAssetItem, EsiAssetLocation, EsiAssetName, EsiAsteroidBelt, EsiAttributes, EsiBloodline,
    EsiBlueprint, EsiBookmark, EsiBookmarkFolder, EsiCalendarEvent, EsiCalendarEventDetail,
    EsiCategoryInfo, EsiCharacterAffiliation, EsiCharacterFleet, EsiCharacterFwStats,
    EsiCharacterInfo, EsiCharacterMedal, EsiCharacterOrder, EsiCharacterPortrait,
    EsiCharacterRoles, EsiCharacterTitle, EsiClient, EsiClones, EsiCompletedOpportunity,
    EsiConstellationInfo, EsiContact, EsiContactLabel, EsiContactNotification, EsiContainerLog,
    EsiContract, EsiContractBid, EsiContractItem, EsiCorpDivisions, EsiCorpFacility,
    EsiCorpFwStats, EsiCorpIcons, EsiCorpMedal, EsiCorpMemberRole, EsiCorpMemberTitle,
    EsiCorpMemberTracking, EsiCorpStarbase, EsiCorpStarbaseDetail, EsiCorpStructure,
    EsiCorpTitle, EsiCorpWalletDivision, EsiCorporationHistoryEntry, EsiCorporationInfo,
    EsiCustomsOffice, EsiDogmaAttribute, EsiDogmaEffect, EsiDynamicItem, EsiError,
    EsiFaction, EsiFatigue, EsiFitting, EsiFleetInfo, EsiFleetInvitation, EsiFleetMember,
    EsiFleetMovement, EsiFleetNaming, EsiFleetSquadCreated, EsiFleetUpdate, EsiFleetWing,
    EsiFleetWingCreated, EsiFwCharacterLeaderboards, EsiFwCorporationLeaderboards,
    EsiFwFactionStats, EsiFwLeaderboards, EsiFwSystem, EsiFwWar, EsiGraphic, EsiGroupInfo,
    EsiIncursion, EsiIndustryFacility, EsiIndustryJob, EsiIndustrySystem, EsiInsurancePrice,
    EsiIssuedMedal, EsiKillmail, EsiKillmailRef, EsiLocation, EsiLoyaltyPoints,
    EsiLoyaltyStoreOffer, EsiMailBody, EsiMailHeader, EsiMailLabels, EsiMailUpdate,
    EsiMailingList, EsiMarketGroupInfo, EsiMarketHistoryEntry, EsiMarketOrder, EsiMarketPrice,
    EsiMiningEntry, EsiMiningExtraction, EsiMiningObserver, EsiMiningObserverEntry, EsiMoon,
    EsiNewFitting, EsiNewFittingResponse, EsiNewMail, EsiNewMailLabel, EsiNewMailWindow,
    EsiNotification, EsiOnlineStatus, EsiPlanet, EsiPlanetDetail, EsiPlanetSummary, EsiRace,
    EsiRegionInfo, EsiResolvedIds, EsiResolvedName, EsiRoleHistory, EsiSchematic, EsiSearchResult,
    EsiServerStatus, EsiShareholder, EsiShip, EsiSkillqueueEntry, EsiSkills, EsiSolarSystemInfo,
    EsiSovereigntyCampaign, EsiSovereigntyMap, EsiSovereigntyStructure, EsiStanding, EsiStar,
    EsiStargateInfo, EsiStationInfo, EsiStructureInfo, EsiSystemJumps, EsiSystemKills,
    EsiTypeInfo, EsiWalletJournalEntry, EsiWalletTransaction, EsiWar, Result,
};

const RESOLVE_NAMES_CHUNK_SIZE: usize = 1000;
const RESOLVE_IDS_CHUNK_SIZE: usize = 500;
const ASSET_NAMES_CHUNK_SIZE: usize = 1000;
const AFFILIATION_CHUNK_SIZE: usize = 1000;

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
        body: &(impl serde::Serialize + ?Sized),
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

    /// DELETE a full URL (including query params), discarding the response body.
    async fn delete_url(&self, url: &str) -> Result<()> {
        let _resp = self.request_delete(url).await?;
        Ok(())
    }

    /// PUT a path relative to `base_url` with a JSON body, discarding the response.
    async fn put_json(
        &self,
        path: &str,
        body: &(impl serde::Serialize + ?Sized),
    ) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let _resp = self.request_put(&url, body).await?;
        Ok(())
    }

    /// POST a path with a JSON body, discarding the response body.
    async fn post_json_void(
        &self,
        path: &str,
        body: &(impl serde::Serialize + ?Sized),
    ) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let _resp = self.request_post(&url, body).await?;
        Ok(())
    }

    /// POST a full URL with an empty body (for UI command endpoints).
    async fn post_empty_url(&self, url: &str) -> Result<()> {
        let _resp = self.request_post(url, &serde_json::json!({})).await?;
        Ok(())
    }

    /// Build a full URL from a base path with query parameters, using `Url::parse_with_params`.
    fn build_url(&self, path: &str, params: &[(&str, &str)]) -> Result<url::Url> {
        let base = format!("{}{}", self.base_url, path);
        url::Url::parse_with_params(&base, params)
            .map_err(|e| EsiError::Internal(format!("failed to build URL: {}", e)))
    }

    /// Build a contact endpoint URL with standing and optional label/watched params.
    fn build_contact_url(
        &self,
        character_id: i64,
        standing: f64,
        label_ids: Option<&[i64]>,
        watched: Option<bool>,
    ) -> Result<url::Url> {
        let base = format!(
            "{}/characters/{}/contacts/",
            self.base_url, character_id
        );
        let mut url = url::Url::parse(&base)
            .map_err(|e| EsiError::Internal(format!("failed to build URL: {}", e)))?;
        let standing_str = standing.to_string();
        url.query_pairs_mut().append_pair("standing", &standing_str);
        if let Some(labels) = label_ids {
            for label in labels {
                url.query_pairs_mut()
                    .append_pair("label_ids", &label.to_string());
            }
        }
        if let Some(w) = watched {
            url.query_pairs_mut()
                .append_pair("watched", &w.to_string());
        }
        Ok(url)
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
    ///
    /// Pass `from_id` to page backwards through older transactions.
    #[tracing::instrument(skip(self))]
    pub async fn wallet_transactions(
        &self,
        character_id: i64,
        from_id: Option<i64>,
    ) -> Result<Vec<EsiWalletTransaction>> {
        match from_id {
            Some(id) => {
                self.get_json(&format!(
                    "/characters/{}/wallet/transactions/?from_id={}",
                    character_id, id
                ))
                .await
            }
            None => {
                self.get_json(&format!(
                    "/characters/{}/wallet/transactions/",
                    character_id
                ))
                .await
            }
        }
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
    ///
    /// Pass `include_completed = true` to include completed jobs.
    #[tracing::instrument(skip(self))]
    pub async fn character_industry_jobs(
        &self,
        character_id: i64,
        include_completed: bool,
    ) -> Result<Vec<EsiIndustryJob>> {
        if include_completed {
            self.get_json(&format!(
                "/characters/{}/industry/jobs/?include_completed=true",
                character_id
            ))
            .await
        } else {
            self.get_json(&format!(
                "/characters/{}/industry/jobs/",
                character_id
            ))
            .await
        }
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
    ///
    /// Pass `labels` to filter by label IDs.
    #[tracing::instrument(skip(self))]
    pub async fn character_mail(
        &self,
        character_id: i64,
        labels: Option<&[i32]>,
    ) -> Result<Vec<EsiMailHeader>> {
        match labels {
            Some(ids) if !ids.is_empty() => {
                let label_str: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
                self.get_json(&format!(
                    "/characters/{}/mail/?labels={}",
                    character_id,
                    label_str.join(",")
                ))
                .await
            }
            _ => {
                self.get_json(&format!("/characters/{}/mail/", character_id))
                    .await
            }
        }
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
    ///
    /// Pass `from_event` to page backwards through older events.
    #[tracing::instrument(skip(self))]
    pub async fn character_calendar(
        &self,
        character_id: i64,
        from_event: Option<i32>,
    ) -> Result<Vec<EsiCalendarEvent>> {
        match from_event {
            Some(id) => {
                self.get_json(&format!(
                    "/characters/{}/calendar/?from_event={}",
                    character_id, id
                ))
                .await
            }
            None => {
                self.get_json(&format!(
                    "/characters/{}/calendar/",
                    character_id
                ))
                .await
            }
        }
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

    // -----------------------------------------------------------------------
    // Corp wallet endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation wallet division balances.
    #[tracing::instrument(skip(self))]
    pub async fn corp_wallet_balances(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCorpWalletDivision>> {
        self.get_json(&format!("/corporations/{}/wallets/", corporation_id))
            .await
    }

    /// Fetch corporation wallet journal for a division (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_wallet_journal(
        &self,
        corporation_id: i64,
        division: i32,
    ) -> Result<Vec<EsiWalletJournalEntry>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/wallets/{}/journal/",
            corporation_id, division
        ))
        .await
    }

    /// Fetch corporation wallet transactions for a division.
    ///
    /// Pass `from_id` to page backwards through older transactions.
    #[tracing::instrument(skip(self))]
    pub async fn corp_wallet_transactions(
        &self,
        corporation_id: i64,
        division: i32,
        from_id: Option<i64>,
    ) -> Result<Vec<EsiWalletTransaction>> {
        match from_id {
            Some(id) => {
                self.get_json(&format!(
                    "/corporations/{}/wallets/{}/transactions/?from_id={}",
                    corporation_id, division, id
                ))
                .await
            }
            None => {
                self.get_json(&format!(
                    "/corporations/{}/wallets/{}/transactions/",
                    corporation_id, division
                ))
                .await
            }
        }
    }

    // -----------------------------------------------------------------------
    // Corp asset endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation assets (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_assets(&self, corporation_id: i64) -> Result<Vec<EsiAssetItem>> {
        self.get_paginated_json(&format!("/corporations/{}/assets/", corporation_id))
            .await
    }

    /// Resolve corporation asset item IDs to names.
    #[tracing::instrument(skip(self, item_ids))]
    pub async fn corp_asset_names(
        &self,
        corporation_id: i64,
        item_ids: &[i64],
    ) -> Result<Vec<EsiAssetName>> {
        self.post_json(
            &format!("/corporations/{}/assets/names/", corporation_id),
            item_ids,
        )
        .await
    }

    /// Resolve corporation asset item IDs to locations.
    #[tracing::instrument(skip(self, item_ids))]
    pub async fn corp_asset_locations(
        &self,
        corporation_id: i64,
        item_ids: &[i64],
    ) -> Result<Vec<EsiAssetLocation>> {
        self.post_json(
            &format!("/corporations/{}/assets/locations/", corporation_id),
            item_ids,
        )
        .await
    }

    // -----------------------------------------------------------------------
    // Corp industry endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation industry jobs (paginated).
    ///
    /// Pass `include_completed = true` to include completed jobs.
    #[tracing::instrument(skip(self))]
    pub async fn corp_industry_jobs(
        &self,
        corporation_id: i64,
        include_completed: bool,
    ) -> Result<Vec<EsiIndustryJob>> {
        if include_completed {
            self.get_paginated_json(&format!(
                "/corporations/{}/industry/jobs/?include_completed=true",
                corporation_id
            ))
            .await
        } else {
            self.get_paginated_json(&format!(
                "/corporations/{}/industry/jobs/",
                corporation_id
            ))
            .await
        }
    }

    /// Fetch corporation blueprints (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_blueprints(&self, corporation_id: i64) -> Result<Vec<EsiBlueprint>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/blueprints/",
            corporation_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Corp contract endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation contracts (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_contracts(&self, corporation_id: i64) -> Result<Vec<EsiContract>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/contracts/",
            corporation_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Corp order endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation active market orders.
    #[tracing::instrument(skip(self))]
    pub async fn corp_orders(&self, corporation_id: i64) -> Result<Vec<EsiCharacterOrder>> {
        self.get_json(&format!("/corporations/{}/orders/", corporation_id))
            .await
    }

    /// Fetch corporation order history (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_order_history(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCharacterOrder>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/orders/history/",
            corporation_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Corp member endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation member IDs.
    #[tracing::instrument(skip(self))]
    pub async fn corp_members(&self, corporation_id: i64) -> Result<Vec<i64>> {
        self.get_json(&format!("/corporations/{}/members/", corporation_id))
            .await
    }

    /// Fetch corporation member titles.
    #[tracing::instrument(skip(self))]
    pub async fn corp_member_titles(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCorpMemberTitle>> {
        self.get_json(&format!(
            "/corporations/{}/members/titles/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation member roles.
    #[tracing::instrument(skip(self))]
    pub async fn corp_member_roles(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCorpMemberRole>> {
        self.get_json(&format!("/corporations/{}/roles/", corporation_id))
            .await
    }

    /// Fetch corporation member tracking info.
    #[tracing::instrument(skip(self))]
    pub async fn corp_member_tracking(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCorpMemberTracking>> {
        self.get_json(&format!(
            "/corporations/{}/membertracking/",
            corporation_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Corp structure endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation structures (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_structures(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCorpStructure>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/structures/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation starbases (POSes).
    #[tracing::instrument(skip(self))]
    pub async fn corp_starbases(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCorpStarbase>> {
        self.get_json(&format!(
            "/corporations/{}/starbases/",
            corporation_id
        ))
        .await
    }

    /// Fetch detailed configuration of a specific starbase.
    #[tracing::instrument(skip(self))]
    pub async fn corp_starbase_detail(
        &self,
        corporation_id: i64,
        starbase_id: i64,
        system_id: i32,
    ) -> Result<EsiCorpStarbaseDetail> {
        self.get_json(&format!(
            "/corporations/{}/starbases/{}/?system_id={}",
            corporation_id, starbase_id, system_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Dogma endpoints
    // -----------------------------------------------------------------------

    /// Fetch a dogma attribute by ID.
    #[tracing::instrument(skip(self))]
    pub async fn get_dogma_attribute(&self, attribute_id: i32) -> Result<EsiDogmaAttribute> {
        self.get_json(&format!("/dogma/attributes/{}/", attribute_id))
            .await
    }

    /// Fetch a dogma effect by ID.
    #[tracing::instrument(skip(self))]
    pub async fn get_dogma_effect(&self, effect_id: i32) -> Result<EsiDogmaEffect> {
        self.get_json(&format!("/dogma/effects/{}/", effect_id))
            .await
    }

    /// Fetch a mutated (dynamic) item's stats.
    #[tracing::instrument(skip(self))]
    pub async fn get_dynamic_item(&self, type_id: i32, item_id: i64) -> Result<EsiDynamicItem> {
        self.get_json(&format!("/dogma/dynamic/items/{}/{}/", type_id, item_id))
            .await
    }

    // -----------------------------------------------------------------------
    // Opportunities endpoints
    // -----------------------------------------------------------------------

    /// Fetch all opportunity group IDs.
    #[tracing::instrument(skip(self))]
    pub async fn opportunity_group_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/opportunities/groups/").await
    }

    /// Fetch all opportunity task IDs.
    #[tracing::instrument(skip(self))]
    pub async fn opportunity_task_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/opportunities/tasks/").await
    }

    /// Fetch completed opportunities for a character (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_opportunities(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiCompletedOpportunity>> {
        self.get_json(&format!("/characters/{}/opportunities/", character_id))
            .await
    }

    // -----------------------------------------------------------------------
    // Fleet endpoints
    // -----------------------------------------------------------------------

    /// Fetch a character's current fleet (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_fleet(&self, character_id: i64) -> Result<EsiCharacterFleet> {
        self.get_json(&format!("/characters/{}/fleet/", character_id))
            .await
    }

    /// Fetch fleet information.
    #[tracing::instrument(skip(self))]
    pub async fn get_fleet(&self, fleet_id: i64) -> Result<EsiFleetInfo> {
        self.get_json(&format!("/fleets/{}/", fleet_id)).await
    }

    /// Fetch fleet members.
    #[tracing::instrument(skip(self))]
    pub async fn fleet_members(&self, fleet_id: i64) -> Result<Vec<EsiFleetMember>> {
        self.get_json(&format!("/fleets/{}/members/", fleet_id))
            .await
    }

    /// Fetch fleet wings and squads.
    #[tracing::instrument(skip(self))]
    pub async fn fleet_wings(&self, fleet_id: i64) -> Result<Vec<EsiFleetWing>> {
        self.get_json(&format!("/fleets/{}/wings/", fleet_id))
            .await
    }

    // -----------------------------------------------------------------------
    // War endpoints
    // -----------------------------------------------------------------------

    /// List active war IDs (paginated).
    ///
    /// Pass `max_war_id` to page backwards from a specific war ID.
    #[tracing::instrument(skip(self))]
    pub async fn list_war_ids(&self, max_war_id: Option<i32>) -> Result<Vec<i32>> {
        match max_war_id {
            Some(id) => {
                self.get_paginated_json(&format!("/wars/?max_war_id={}", id))
                    .await
            }
            None => self.get_paginated_json("/wars/").await,
        }
    }

    /// Fetch war details.
    #[tracing::instrument(skip(self))]
    pub async fn get_war(&self, war_id: i32) -> Result<EsiWar> {
        self.get_json(&format!("/wars/{}/", war_id)).await
    }

    /// Fetch killmails for a war (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn war_killmails(&self, war_id: i32) -> Result<Vec<EsiKillmailRef>> {
        self.get_paginated_json(&format!("/wars/{}/killmails/", war_id))
            .await
    }

    // -----------------------------------------------------------------------
    // Faction Warfare endpoints
    // -----------------------------------------------------------------------

    /// Fetch faction warfare statistics.
    #[tracing::instrument(skip(self))]
    pub async fn fw_stats(&self) -> Result<Vec<EsiFwFactionStats>> {
        self.get_json("/fw/stats/").await
    }

    /// Fetch faction warfare contested systems.
    #[tracing::instrument(skip(self))]
    pub async fn fw_systems(&self) -> Result<Vec<EsiFwSystem>> {
        self.get_json("/fw/systems/").await
    }

    /// Fetch faction warfare leaderboards.
    #[tracing::instrument(skip(self))]
    pub async fn fw_leaderboards(&self) -> Result<EsiFwLeaderboards> {
        self.get_json("/fw/leaderboards/").await
    }

    /// Fetch faction warfare wars.
    #[tracing::instrument(skip(self))]
    pub async fn fw_wars(&self) -> Result<Vec<EsiFwWar>> {
        self.get_json("/fw/wars/").await
    }

    // -----------------------------------------------------------------------
    // Insurance endpoint
    // -----------------------------------------------------------------------

    /// Fetch insurance prices for all ship types.
    #[tracing::instrument(skip(self))]
    pub async fn insurance_prices(&self) -> Result<Vec<EsiInsurancePrice>> {
        self.get_json("/insurance/prices/").await
    }

    // -----------------------------------------------------------------------
    // Route endpoint
    // -----------------------------------------------------------------------

    /// Calculate a route between two solar systems.
    ///
    /// `flag` controls the pathfinding algorithm: `"shortest"` (default),
    /// `"secure"`, or `"insecure"`.
    /// `avoid` is a list of system IDs to avoid.
    /// `connections` is a list of `[from, to]` pairs for wormhole connections.
    #[tracing::instrument(skip(self, avoid, connections))]
    pub async fn get_route(
        &self,
        origin: i32,
        destination: i32,
        flag: Option<&str>,
        avoid: &[i32],
        connections: Option<&[[i32; 2]]>,
    ) -> Result<Vec<i32>> {
        let base = format!(
            "{}/route/{}/{}/",
            self.base_url, origin, destination
        );
        let mut url = url::Url::parse(&base)
            .map_err(|e| EsiError::Internal(format!("failed to build route URL: {}", e)))?;

        if let Some(f) = flag {
            url.query_pairs_mut().append_pair("flag", f);
        }
        for &system_id in avoid {
            url.query_pairs_mut()
                .append_pair("avoid", &system_id.to_string());
        }
        if let Some(conns) = connections {
            for conn in conns {
                url.query_pairs_mut().append_pair(
                    "connections",
                    &format!("{}|{}", conn[0], conn[1]),
                );
            }
        }

        self.request(url.as_str())
            .await?
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))
    }

    // -----------------------------------------------------------------------
    // History endpoints
    // -----------------------------------------------------------------------

    /// Fetch a corporation's alliance history.
    #[tracing::instrument(skip(self))]
    pub async fn corp_alliance_history(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiAllianceHistoryEntry>> {
        self.get_json(&format!(
            "/corporations/{}/alliancehistory/",
            corporation_id
        ))
        .await
    }

    /// Fetch a character's corporation history.
    #[tracing::instrument(skip(self))]
    pub async fn character_corporation_history(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiCorporationHistoryEntry>> {
        self.get_json(&format!(
            "/characters/{}/corporationhistory/",
            character_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Alliance endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// List all alliance IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_alliance_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/alliances/").await
    }

    /// Fetch corporation IDs in an alliance.
    #[tracing::instrument(skip(self))]
    pub async fn alliance_corporations(&self, alliance_id: i64) -> Result<Vec<i32>> {
        self.get_json(&format!("/alliances/{}/corporations/", alliance_id))
            .await
    }

    /// Fetch alliance icon URLs.
    #[tracing::instrument(skip(self))]
    pub async fn alliance_icons(&self, alliance_id: i64) -> Result<EsiAllianceIcons> {
        self.get_json(&format!("/alliances/{}/icons/", alliance_id))
            .await
    }

    /// Fetch alliance contacts (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn alliance_contacts(&self, alliance_id: i64) -> Result<Vec<EsiContact>> {
        self.get_paginated_json(&format!("/alliances/{}/contacts/", alliance_id))
            .await
    }

    /// Fetch alliance contact labels (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn alliance_contact_labels(
        &self,
        alliance_id: i64,
    ) -> Result<Vec<EsiContactLabel>> {
        self.get_json(&format!(
            "/alliances/{}/contacts/labels/",
            alliance_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Character info & history endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Bulk character affiliation lookup. Auto-chunks at 1000.
    #[tracing::instrument(skip(self, character_ids))]
    pub async fn character_affiliation(
        &self,
        character_ids: &[i64],
    ) -> Result<Vec<EsiCharacterAffiliation>> {
        if character_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut all = Vec::with_capacity(character_ids.len());
        for chunk in character_ids.chunks(AFFILIATION_CHUNK_SIZE) {
            let batch: Vec<EsiCharacterAffiliation> =
                self.post_json("/characters/affiliation/", &chunk).await?;
            all.extend(batch);
        }
        Ok(all)
    }

    /// Fetch character portrait URLs.
    #[tracing::instrument(skip(self))]
    pub async fn character_portrait(&self, character_id: i64) -> Result<EsiCharacterPortrait> {
        self.get_json(&format!("/characters/{}/portrait/", character_id))
            .await
    }

    /// Fetch character roles (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_roles(&self, character_id: i64) -> Result<EsiCharacterRoles> {
        self.get_json(&format!("/characters/{}/roles/", character_id))
            .await
    }

    /// Fetch character titles (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_titles(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiCharacterTitle>> {
        self.get_json(&format!("/characters/{}/titles/", character_id))
            .await
    }

    /// Fetch character standings (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_standings(&self, character_id: i64) -> Result<Vec<EsiStanding>> {
        self.get_json(&format!("/characters/{}/standings/", character_id))
            .await
    }

    /// Fetch character medals (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_medals(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiCharacterMedal>> {
        self.get_json(&format!("/characters/{}/medals/", character_id))
            .await
    }

    /// Fetch character agent research info (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_agents_research(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiAgentResearch>> {
        self.get_json(&format!(
            "/characters/{}/agents_research/",
            character_id
        ))
        .await
    }

    /// Fetch character jump fatigue (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_fatigue(&self, character_id: i64) -> Result<EsiFatigue> {
        self.get_json(&format!("/characters/{}/fatigue/", character_id))
            .await
    }

    /// Fetch character FW stats (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_fw_stats(
        &self,
        character_id: i64,
    ) -> Result<EsiCharacterFwStats> {
        self.get_json(&format!("/characters/{}/fw/stats/", character_id))
            .await
    }

    /// Calculate CSPA charge cost for contacting characters.
    #[tracing::instrument(skip(self, character_ids))]
    pub async fn character_cspa_charge(
        &self,
        character_id: i64,
        character_ids: &[i64],
    ) -> Result<f64> {
        self.post_json(
            &format!("/characters/{}/cspa/", character_id),
            character_ids,
        )
        .await
    }

    // -----------------------------------------------------------------------
    // Character asset detail endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Resolve character asset item IDs to locations.
    #[tracing::instrument(skip(self, item_ids))]
    pub async fn character_asset_locations(
        &self,
        character_id: i64,
        item_ids: &[i64],
    ) -> Result<Vec<EsiAssetLocation>> {
        self.post_json(
            &format!("/characters/{}/assets/locations/", character_id),
            item_ids,
        )
        .await
    }

    /// Resolve character asset item IDs to names. Auto-chunks at 1000.
    #[tracing::instrument(skip(self, item_ids))]
    pub async fn character_asset_names(
        &self,
        character_id: i64,
        item_ids: &[i64],
    ) -> Result<Vec<EsiAssetName>> {
        if item_ids.is_empty() {
            return Ok(Vec::new());
        }
        let path = format!("/characters/{}/assets/names/", character_id);
        let mut all = Vec::with_capacity(item_ids.len());
        for chunk in item_ids.chunks(ASSET_NAMES_CHUNK_SIZE) {
            let batch: Vec<EsiAssetName> = self.post_json(&path, &chunk).await?;
            all.extend(batch);
        }
        Ok(all)
    }

    // -----------------------------------------------------------------------
    // Character contact CRUD endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Add contacts to a character. Returns added contact IDs.
    #[tracing::instrument(skip(self, contact_ids))]
    pub async fn add_contacts(
        &self,
        character_id: i64,
        standing: f64,
        contact_ids: &[i64],
        label_ids: Option<&[i64]>,
        watched: Option<bool>,
    ) -> Result<Vec<i32>> {
        let url = self.build_contact_url(character_id, standing, label_ids, watched)?;
        let resp = self.request_post(url.as_str(), contact_ids).await?;
        resp.json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))
    }

    /// Edit contacts for a character.
    #[tracing::instrument(skip(self, contact_ids))]
    pub async fn edit_contacts(
        &self,
        character_id: i64,
        standing: f64,
        contact_ids: &[i64],
        label_ids: Option<&[i64]>,
        watched: Option<bool>,
    ) -> Result<()> {
        let url = self.build_contact_url(character_id, standing, label_ids, watched)?;
        let _resp = self.request_put(url.as_str(), contact_ids).await?;
        Ok(())
    }

    /// Delete contacts from a character.
    #[tracing::instrument(skip(self, contact_ids))]
    pub async fn delete_contacts(
        &self,
        character_id: i64,
        contact_ids: &[i64],
    ) -> Result<()> {
        let base = format!(
            "{}/characters/{}/contacts/",
            self.base_url, character_id
        );
        let mut url = url::Url::parse(&base)
            .map_err(|e| EsiError::Internal(format!("failed to build URL: {}", e)))?;
        for &id in contact_ids {
            url.query_pairs_mut()
                .append_pair("contact_ids", &id.to_string());
        }
        self.delete_url(url.as_str()).await
    }

    // -----------------------------------------------------------------------
    // Character calendar write endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Set a character's response to a calendar event.
    #[tracing::instrument(skip(self))]
    pub async fn set_event_response(
        &self,
        character_id: i64,
        event_id: i64,
        response: &str,
    ) -> Result<()> {
        self.put_json(
            &format!("/characters/{}/calendar/{}/", character_id, event_id),
            &crate::EsiEventResponse {
                response: response.to_string(),
            },
        )
        .await
    }

    /// Fetch attendees for a calendar event (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn event_attendees(
        &self,
        character_id: i64,
        event_id: i64,
    ) -> Result<Vec<crate::EsiEventAttendee>> {
        self.get_json(&format!(
            "/characters/{}/calendar/{}/attendees/",
            character_id, event_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Character mail management endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Create a new mail label. Returns the label ID.
    #[tracing::instrument(skip(self, label))]
    pub async fn create_mail_label(
        &self,
        character_id: i64,
        label: &EsiNewMailLabel,
    ) -> Result<i32> {
        self.post_json(
            &format!("/characters/{}/mail/labels/", character_id),
            label,
        )
        .await
    }

    /// Delete a mail label.
    #[tracing::instrument(skip(self))]
    pub async fn delete_mail_label(
        &self,
        character_id: i64,
        label_id: i32,
    ) -> Result<()> {
        self.delete_path(&format!(
            "/characters/{}/mail/labels/{}/",
            character_id, label_id
        ))
        .await
    }

    /// Fetch a character's mailing lists (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_mailing_lists(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiMailingList>> {
        self.get_json(&format!(
            "/characters/{}/mail/lists/",
            character_id
        ))
        .await
    }

    /// Delete a mail.
    #[tracing::instrument(skip(self))]
    pub async fn delete_mail(
        &self,
        character_id: i64,
        mail_id: i64,
    ) -> Result<()> {
        self.delete_path(&format!(
            "/characters/{}/mail/{}/",
            character_id, mail_id
        ))
        .await
    }

    /// Update mail metadata (read status, labels).
    #[tracing::instrument(skip(self, update))]
    pub async fn update_mail(
        &self,
        character_id: i64,
        mail_id: i64,
        update: &EsiMailUpdate,
    ) -> Result<()> {
        self.put_json(
            &format!("/characters/{}/mail/{}/", character_id, mail_id),
            update,
        )
        .await
    }

    // -----------------------------------------------------------------------
    // Character mining & notification endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Fetch a character's personal mining ledger (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn character_mining_ledger(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiMiningEntry>> {
        self.get_paginated_json(&format!("/characters/{}/mining/", character_id))
            .await
    }

    /// Fetch a character's contact notifications.
    #[tracing::instrument(skip(self))]
    pub async fn character_contact_notifications(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiContactNotification>> {
        self.get_json(&format!(
            "/characters/{}/notifications/contacts/",
            character_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Character search endpoint (Phase 5)
    // -----------------------------------------------------------------------

    /// Authenticated search for entities by name.
    #[tracing::instrument(skip(self))]
    pub async fn character_search(
        &self,
        character_id: i64,
        search: &str,
        categories: &str,
        strict: bool,
    ) -> Result<EsiSearchResult> {
        let strict_str = strict.to_string();
        let url = self.build_url(
            &format!("/characters/{}/search/", character_id),
            &[
                ("search", search),
                ("categories", categories),
                ("strict", &strict_str),
            ],
        )?;

        self.request(url.as_str())
            .await?
            .json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))
    }

    // -----------------------------------------------------------------------
    // Public contract endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Fetch public contracts in a region (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn public_contracts(&self, region_id: i32) -> Result<Vec<EsiContract>> {
        self.get_paginated_json(&format!("/contracts/public/{}/", region_id))
            .await
    }

    /// Fetch bids on a public contract (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn public_contract_bids(
        &self,
        contract_id: i64,
    ) -> Result<Vec<EsiContractBid>> {
        self.get_paginated_json(&format!(
            "/contracts/public/bids/{}/",
            contract_id
        ))
        .await
    }

    /// Fetch items in a public contract (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn public_contract_items(
        &self,
        contract_id: i64,
    ) -> Result<Vec<EsiContractItem>> {
        self.get_paginated_json(&format!(
            "/contracts/public/items/{}/",
            contract_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Corporation additional endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// List NPC corporation IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_npc_corp_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/corporations/npccorps/").await
    }

    /// Fetch corporation contacts (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_contacts(&self, corporation_id: i64) -> Result<Vec<EsiContact>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/contacts/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation contact labels (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_contact_labels(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiContactLabel>> {
        self.get_json(&format!(
            "/corporations/{}/contacts/labels/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation container audit logs (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_container_logs(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiContainerLog>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/containers/logs/",
            corporation_id
        ))
        .await
    }

    /// Fetch bids on a corporation contract.
    #[tracing::instrument(skip(self))]
    pub async fn corp_contract_bids(
        &self,
        corporation_id: i64,
        contract_id: i64,
    ) -> Result<Vec<EsiContractBid>> {
        self.get_json(&format!(
            "/corporations/{}/contracts/{}/bids/",
            corporation_id, contract_id
        ))
        .await
    }

    /// Fetch items in a corporation contract.
    #[tracing::instrument(skip(self))]
    pub async fn corp_contract_items(
        &self,
        corporation_id: i64,
        contract_id: i64,
    ) -> Result<Vec<EsiContractItem>> {
        self.get_json(&format!(
            "/corporations/{}/contracts/{}/items/",
            corporation_id, contract_id
        ))
        .await
    }

    /// Fetch corporation customs offices (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_customs_offices(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCustomsOffice>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/customs_offices/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation divisions.
    #[tracing::instrument(skip(self))]
    pub async fn corp_divisions(&self, corporation_id: i64) -> Result<EsiCorpDivisions> {
        self.get_json(&format!(
            "/corporations/{}/divisions/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation facilities.
    #[tracing::instrument(skip(self))]
    pub async fn corp_facilities(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCorpFacility>> {
        self.get_json(&format!(
            "/corporations/{}/facilities/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation FW stats.
    #[tracing::instrument(skip(self))]
    pub async fn corp_fw_stats(&self, corporation_id: i64) -> Result<EsiCorpFwStats> {
        self.get_json(&format!(
            "/corporations/{}/fw/stats/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation icon URLs.
    #[tracing::instrument(skip(self))]
    pub async fn corp_icons(&self, corporation_id: i64) -> Result<EsiCorpIcons> {
        self.get_json(&format!("/corporations/{}/icons/", corporation_id))
            .await
    }

    /// Fetch corporation medals (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_medals(&self, corporation_id: i64) -> Result<Vec<EsiCorpMedal>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/medals/",
            corporation_id
        ))
        .await
    }

    /// Fetch issued medals (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_medals_issued(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiIssuedMedal>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/medals/issued/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation member limit.
    #[tracing::instrument(skip(self))]
    pub async fn corp_member_limit(&self, corporation_id: i64) -> Result<i32> {
        self.get_json(&format!(
            "/corporations/{}/members/limit/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation role change history (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_roles_history(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiRoleHistory>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/roles/history/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation shareholders (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_shareholders(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiShareholder>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/shareholders/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation standings (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_standings(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiStanding>> {
        self.get_paginated_json(&format!(
            "/corporations/{}/standings/",
            corporation_id
        ))
        .await
    }

    /// Fetch corporation titles.
    #[tracing::instrument(skip(self))]
    pub async fn corp_titles(&self, corporation_id: i64) -> Result<Vec<EsiCorpTitle>> {
        self.get_json(&format!(
            "/corporations/{}/titles/",
            corporation_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Corporation mining endpoints (Phase 5, note: singular /corporation/ path)
    // -----------------------------------------------------------------------

    /// Fetch moon mining extractions (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_mining_extractions(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiMiningExtraction>> {
        self.get_paginated_json(&format!(
            "/corporation/{}/mining/extractions/",
            corporation_id
        ))
        .await
    }

    /// Fetch mining observers (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_mining_observers(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiMiningObserver>> {
        self.get_paginated_json(&format!(
            "/corporation/{}/mining/observers/",
            corporation_id
        ))
        .await
    }

    /// Fetch mining observer details (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_mining_observer_details(
        &self,
        corporation_id: i64,
        observer_id: i64,
    ) -> Result<Vec<EsiMiningObserverEntry>> {
        self.get_paginated_json(&format!(
            "/corporation/{}/mining/observers/{}/",
            corporation_id, observer_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Dogma list endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// List all dogma attribute IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_dogma_attribute_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/dogma/attributes/").await
    }

    /// List all dogma effect IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_dogma_effect_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/dogma/effects/").await
    }

    // -----------------------------------------------------------------------
    // Fleet write endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Update fleet settings.
    #[tracing::instrument(skip(self, settings))]
    pub async fn update_fleet(
        &self,
        fleet_id: i64,
        settings: &EsiFleetUpdate,
    ) -> Result<()> {
        self.put_json(&format!("/fleets/{}/", fleet_id), settings)
            .await
    }

    /// Invite a character to a fleet.
    #[tracing::instrument(skip(self, invitation))]
    pub async fn invite_to_fleet(
        &self,
        fleet_id: i64,
        invitation: &EsiFleetInvitation,
    ) -> Result<()> {
        self.post_json_void(
            &format!("/fleets/{}/members/", fleet_id),
            invitation,
        )
        .await
    }

    /// Kick a member from a fleet.
    #[tracing::instrument(skip(self))]
    pub async fn kick_fleet_member(&self, fleet_id: i64, member_id: i64) -> Result<()> {
        self.delete_path(&format!(
            "/fleets/{}/members/{}/",
            fleet_id, member_id
        ))
        .await
    }

    /// Move a fleet member to a different wing/squad/role.
    #[tracing::instrument(skip(self, movement))]
    pub async fn move_fleet_member(
        &self,
        fleet_id: i64,
        member_id: i64,
        movement: &EsiFleetMovement,
    ) -> Result<()> {
        self.put_json(
            &format!("/fleets/{}/members/{}/", fleet_id, member_id),
            movement,
        )
        .await
    }

    /// Create a fleet wing. Returns the wing ID.
    #[tracing::instrument(skip(self))]
    pub async fn create_fleet_wing(&self, fleet_id: i64) -> Result<EsiFleetWingCreated> {
        self.post_json(
            &format!("/fleets/{}/wings/", fleet_id),
            &serde_json::json!({}),
        )
        .await
    }

    /// Rename a fleet wing.
    #[tracing::instrument(skip(self))]
    pub async fn rename_fleet_wing(
        &self,
        fleet_id: i64,
        wing_id: i64,
        name: &str,
    ) -> Result<()> {
        self.put_json(
            &format!("/fleets/{}/wings/{}/", fleet_id, wing_id),
            &EsiFleetNaming {
                name: name.to_string(),
            },
        )
        .await
    }

    /// Delete a fleet wing.
    #[tracing::instrument(skip(self))]
    pub async fn delete_fleet_wing(&self, fleet_id: i64, wing_id: i64) -> Result<()> {
        self.delete_path(&format!(
            "/fleets/{}/wings/{}/",
            fleet_id, wing_id
        ))
        .await
    }

    /// Create a fleet squad. Returns the squad ID.
    #[tracing::instrument(skip(self))]
    pub async fn create_fleet_squad(
        &self,
        fleet_id: i64,
        wing_id: i64,
    ) -> Result<EsiFleetSquadCreated> {
        self.post_json(
            &format!("/fleets/{}/wings/{}/squads/", fleet_id, wing_id),
            &serde_json::json!({}),
        )
        .await
    }

    /// Rename a fleet squad.
    #[tracing::instrument(skip(self))]
    pub async fn rename_fleet_squad(
        &self,
        fleet_id: i64,
        squad_id: i64,
        name: &str,
    ) -> Result<()> {
        self.put_json(
            &format!("/fleets/{}/squads/{}/", fleet_id, squad_id),
            &EsiFleetNaming {
                name: name.to_string(),
            },
        )
        .await
    }

    /// Delete a fleet squad.
    #[tracing::instrument(skip(self))]
    pub async fn delete_fleet_squad(&self, fleet_id: i64, squad_id: i64) -> Result<()> {
        self.delete_path(&format!(
            "/fleets/{}/squads/{}/",
            fleet_id, squad_id
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // FW leaderboard endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Fetch character FW leaderboards.
    #[tracing::instrument(skip(self))]
    pub async fn fw_character_leaderboards(&self) -> Result<EsiFwCharacterLeaderboards> {
        self.get_json("/fw/leaderboards/characters/").await
    }

    /// Fetch corporation FW leaderboards.
    #[tracing::instrument(skip(self))]
    pub async fn fw_corporation_leaderboards(&self) -> Result<EsiFwCorporationLeaderboards> {
        self.get_json("/fw/leaderboards/corporations/").await
    }

    // -----------------------------------------------------------------------
    // Industry public endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Fetch public industry facilities.
    #[tracing::instrument(skip(self))]
    pub async fn industry_facilities(&self) -> Result<Vec<EsiIndustryFacility>> {
        self.get_json("/industry/facilities/").await
    }

    /// Fetch industry system cost indices.
    #[tracing::instrument(skip(self))]
    pub async fn industry_systems(&self) -> Result<Vec<EsiIndustrySystem>> {
        self.get_json("/industry/systems/").await
    }

    // -----------------------------------------------------------------------
    // Market structure orders (Phase 5)
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // UI command endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Set an autopilot waypoint.
    #[tracing::instrument(skip(self))]
    pub async fn ui_autopilot_waypoint(
        &self,
        destination_id: i64,
        add_to_beginning: bool,
        clear_other_waypoints: bool,
    ) -> Result<()> {
        let dest = destination_id.to_string();
        let atb = add_to_beginning.to_string();
        let cow = clear_other_waypoints.to_string();
        let url = self.build_url(
            "/ui/autopilot/waypoint/",
            &[
                ("destination_id", &dest),
                ("add_to_beginning", &atb),
                ("clear_other_waypoints", &cow),
            ],
        )?;
        self.post_empty_url(url.as_str()).await
    }

    /// Open the contract window in the client.
    #[tracing::instrument(skip(self))]
    pub async fn ui_open_contract_window(&self, contract_id: i64) -> Result<()> {
        let cid = contract_id.to_string();
        let url = self.build_url("/ui/openwindow/contract/", &[("contract_id", &cid)])?;
        self.post_empty_url(url.as_str()).await
    }

    /// Open the info window in the client.
    #[tracing::instrument(skip(self))]
    pub async fn ui_open_info_window(&self, target_id: i64) -> Result<()> {
        let tid = target_id.to_string();
        let url = self.build_url("/ui/openwindow/information/", &[("target_id", &tid)])?;
        self.post_empty_url(url.as_str()).await
    }

    /// Open the market details window in the client.
    #[tracing::instrument(skip(self))]
    pub async fn ui_open_market_details(&self, type_id: i32) -> Result<()> {
        let tid = type_id.to_string();
        let url = self.build_url("/ui/openwindow/marketdetails/", &[("type_id", &tid)])?;
        self.post_empty_url(url.as_str()).await
    }

    /// Open the new mail window in the client.
    #[tracing::instrument(skip(self, new_mail))]
    pub async fn ui_open_new_mail(&self, new_mail: &EsiNewMailWindow) -> Result<()> {
        self.post_json_void("/ui/openwindow/newmail/", new_mail)
            .await
    }

    // -----------------------------------------------------------------------
    // Universe additional endpoints (Phase 5)
    // -----------------------------------------------------------------------

    /// Fetch all ancestries.
    #[tracing::instrument(skip(self))]
    pub async fn universe_ancestries(&self) -> Result<Vec<EsiAncestry>> {
        self.get_json("/universe/ancestries/").await
    }

    /// Fetch asteroid belt info.
    #[tracing::instrument(skip(self))]
    pub async fn universe_asteroid_belt(&self, asteroid_belt_id: i32) -> Result<EsiAsteroidBelt> {
        self.get_json(&format!(
            "/universe/asteroid_belts/{}/",
            asteroid_belt_id
        ))
        .await
    }

    /// Fetch all bloodlines.
    #[tracing::instrument(skip(self))]
    pub async fn universe_bloodlines(&self) -> Result<Vec<EsiBloodline>> {
        self.get_json("/universe/bloodlines/").await
    }

    /// List all category IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_universe_category_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/universe/categories/").await
    }

    /// List all constellation IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_universe_constellation_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/universe/constellations/").await
    }

    /// Fetch all factions.
    #[tracing::instrument(skip(self))]
    pub async fn universe_factions(&self) -> Result<Vec<EsiFaction>> {
        self.get_json("/universe/factions/").await
    }

    /// List all graphic IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_universe_graphic_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/universe/graphics/").await
    }

    /// Fetch graphic info.
    #[tracing::instrument(skip(self))]
    pub async fn universe_graphic(&self, graphic_id: i32) -> Result<EsiGraphic> {
        self.get_json(&format!("/universe/graphics/{}/", graphic_id))
            .await
    }

    /// List all group IDs (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn list_universe_group_ids(&self) -> Result<Vec<i32>> {
        self.get_paginated_json("/universe/groups/").await
    }

    /// Fetch moon info.
    #[tracing::instrument(skip(self))]
    pub async fn universe_moon(&self, moon_id: i32) -> Result<EsiMoon> {
        self.get_json(&format!("/universe/moons/{}/", moon_id)).await
    }

    /// Fetch planet info (universe data, not PI).
    #[tracing::instrument(skip(self))]
    pub async fn universe_planet(&self, planet_id: i32) -> Result<EsiPlanet> {
        self.get_json(&format!("/universe/planets/{}/", planet_id))
            .await
    }

    /// Fetch all races.
    #[tracing::instrument(skip(self))]
    pub async fn universe_races(&self) -> Result<Vec<EsiRace>> {
        self.get_json("/universe/races/").await
    }

    /// List all region IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_universe_region_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/universe/regions/").await
    }

    /// Fetch PI schematic info.
    #[tracing::instrument(skip(self))]
    pub async fn universe_schematic(&self, schematic_id: i32) -> Result<EsiSchematic> {
        self.get_json(&format!(
            "/universe/schematics/{}/",
            schematic_id
        ))
        .await
    }

    /// Fetch star info.
    #[tracing::instrument(skip(self))]
    pub async fn universe_star(&self, star_id: i32) -> Result<EsiStar> {
        self.get_json(&format!("/universe/stars/{}/", star_id)).await
    }

    /// List all public structure IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_public_structure_ids(&self) -> Result<Vec<i64>> {
        self.get_json("/universe/structures/").await
    }

    /// Fetch system jump statistics.
    #[tracing::instrument(skip(self))]
    pub async fn system_jumps(&self) -> Result<Vec<EsiSystemJumps>> {
        self.get_json("/universe/system_jumps/").await
    }

    /// Fetch system kill statistics.
    #[tracing::instrument(skip(self))]
    pub async fn system_kills(&self) -> Result<Vec<EsiSystemKills>> {
        self.get_json("/universe/system_kills/").await
    }

    /// List all system IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_universe_system_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/universe/systems/").await
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
