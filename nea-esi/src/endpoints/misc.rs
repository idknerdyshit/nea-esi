use crate::{
    EsiClient, EsiContract, EsiContractBid, EsiContractItem, EsiDogmaAttribute, EsiDogmaEffect,
    EsiDynamicItem, EsiFwCharacterLeaderboards, EsiFwCorporationLeaderboards,
    EsiFwFactionStats, EsiFwLeaderboards, EsiFwSystem, EsiFwWar, EsiIncursion,
    EsiIndustryFacility, EsiIndustrySystem, EsiInsurancePrice, EsiKillmailRef,
    EsiLoyaltyStoreOffer, EsiNewMailWindow, EsiServerStatus, EsiWar, Result,
};

impl EsiClient {
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
    // Insurance endpoint
    // -----------------------------------------------------------------------

    /// Fetch insurance prices for all ship types.
    #[tracing::instrument(skip(self))]
    pub async fn insurance_prices(&self) -> Result<Vec<EsiInsurancePrice>> {
        self.get_json("/insurance/prices/").await
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
    // Loyalty store endpoint
    // -----------------------------------------------------------------------

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
    // Industry public endpoints
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
    // Public contract endpoints
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
    // UI command endpoints
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
}
