use crate::{
    EsiAgentResearch, EsiAssetItem, EsiAssetLocation, EsiAssetName, EsiAttributes, EsiBlueprint,
    EsiCharacterAffiliation, EsiCharacterFwStats, EsiCharacterInfo, EsiCharacterMedal,
    EsiCharacterPortrait, EsiCharacterRoles, EsiCharacterTitle, EsiClient, EsiClones,
    EsiCompletedOpportunity, EsiContactNotification, EsiCorporationHistoryEntry, EsiFatigue,
    EsiIndustryJob, EsiLocation, EsiLoyaltyPoints, EsiMiningEntry, EsiOnlineStatus,
    EsiPlanetDetail, EsiPlanetSummary, EsiShip, EsiSkillqueueEntry, EsiSkills, EsiStanding,
    EsiWalletJournalEntry, EsiWalletTransaction, Result,
};

use super::{AFFILIATION_CHUNK_SIZE, ASSET_NAMES_CHUNK_SIZE};

impl EsiClient {
    // -----------------------------------------------------------------------
    // Character info endpoints
    // -----------------------------------------------------------------------

    /// Fetch character info from ESI.
    #[tracing::instrument(skip(self))]
    pub async fn get_character(&self, character_id: i64) -> Result<EsiCharacterInfo> {
        self.get_json(&format!("/characters/{}/", character_id)).await
    }

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
    // Character assets endpoints
    // -----------------------------------------------------------------------

    /// Fetch all assets for a character, handling pagination.
    #[tracing::instrument(skip(self))]
    pub async fn character_assets(&self, character_id: i64) -> Result<Vec<EsiAssetItem>> {
        self.get_paginated_json(&format!("/characters/{}/assets/", character_id))
            .await
    }

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
    // Mining & notification endpoints
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

    /// Fetch completed opportunities for a character (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn character_opportunities(
        &self,
        character_id: i64,
    ) -> Result<Vec<EsiCompletedOpportunity>> {
        self.get_json(&format!("/characters/{}/opportunities/", character_id))
            .await
    }
}
