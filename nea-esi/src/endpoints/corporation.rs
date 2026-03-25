use crate::{
    EsiAllianceHistoryEntry, EsiAssetItem, EsiAssetLocation, EsiAssetName, EsiBlueprint,
    EsiCharacterOrder, EsiClient, EsiContact, EsiContactLabel, EsiContainerLog, EsiContract,
    EsiContractBid, EsiContractItem, EsiCorpDivisions, EsiCorpFacility, EsiCorpFwStats,
    EsiCorpIcons, EsiCorpMedal, EsiCorpMemberRole, EsiCorpMemberTitle, EsiCorpMemberTracking,
    EsiCorpStarbase, EsiCorpStarbaseDetail, EsiCorpStructure, EsiCorpTitle, EsiCorpWalletDivision,
    EsiCorporationInfo, EsiCustomsOffice, EsiIndustryJob, EsiIssuedMedal, EsiMiningExtraction,
    EsiMiningObserver, EsiMiningObserverEntry, EsiRoleHistory, EsiShareholder, EsiStanding,
    EsiWalletJournalEntry, EsiWalletTransaction, Result,
};

impl EsiClient {
    // -----------------------------------------------------------------------
    // Corporation info
    // -----------------------------------------------------------------------

    /// Fetch corporation info from ESI.
    #[tracing::instrument(skip(self))]
    pub async fn get_corporation(&self, corporation_id: i64) -> Result<EsiCorporationInfo> {
        self.get_json(&format!("/corporations/{corporation_id}/"))
            .await
    }

    /// Fetch a corporation's alliance history.
    #[tracing::instrument(skip(self))]
    pub async fn corp_alliance_history(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiAllianceHistoryEntry>> {
        self.get_json(&format!("/corporations/{corporation_id}/alliancehistory/"))
            .await
    }

    /// List NPC corporation IDs.
    #[tracing::instrument(skip(self))]
    pub async fn list_npc_corp_ids(&self) -> Result<Vec<i32>> {
        self.get_json("/corporations/npccorps/").await
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
        self.get_json(&format!("/corporations/{corporation_id}/wallets/"))
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
            "/corporations/{corporation_id}/wallets/{division}/journal/"
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
                    "/corporations/{corporation_id}/wallets/{division}/transactions/?from_id={id}"
                ))
                .await
            }
            None => {
                self.get_json(&format!(
                    "/corporations/{corporation_id}/wallets/{division}/transactions/"
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
        self.get_paginated_json(&format!("/corporations/{corporation_id}/assets/"))
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
            &format!("/corporations/{corporation_id}/assets/names/"),
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
            &format!("/corporations/{corporation_id}/assets/locations/"),
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
                "/corporations/{corporation_id}/industry/jobs/?include_completed=true"
            ))
            .await
        } else {
            self.get_paginated_json(&format!("/corporations/{corporation_id}/industry/jobs/"))
                .await
        }
    }

    /// Fetch corporation blueprints (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_blueprints(&self, corporation_id: i64) -> Result<Vec<EsiBlueprint>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/blueprints/"))
            .await
    }

    // -----------------------------------------------------------------------
    // Corp contract endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation contracts (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_contracts(&self, corporation_id: i64) -> Result<Vec<EsiContract>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/contracts/"))
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
            "/corporations/{corporation_id}/contracts/{contract_id}/bids/"
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
            "/corporations/{corporation_id}/contracts/{contract_id}/items/"
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Corp order endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation active market orders.
    #[tracing::instrument(skip(self))]
    pub async fn corp_orders(&self, corporation_id: i64) -> Result<Vec<EsiCharacterOrder>> {
        self.get_json(&format!("/corporations/{corporation_id}/orders/"))
            .await
    }

    /// Fetch corporation order history (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_order_history(&self, corporation_id: i64) -> Result<Vec<EsiCharacterOrder>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/orders/history/"))
            .await
    }

    // -----------------------------------------------------------------------
    // Corp member endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation member IDs.
    #[tracing::instrument(skip(self))]
    pub async fn corp_members(&self, corporation_id: i64) -> Result<Vec<i64>> {
        self.get_json(&format!("/corporations/{corporation_id}/members/"))
            .await
    }

    /// Fetch corporation member titles.
    #[tracing::instrument(skip(self))]
    pub async fn corp_member_titles(&self, corporation_id: i64) -> Result<Vec<EsiCorpMemberTitle>> {
        self.get_json(&format!("/corporations/{corporation_id}/members/titles/"))
            .await
    }

    /// Fetch corporation member roles.
    #[tracing::instrument(skip(self))]
    pub async fn corp_member_roles(&self, corporation_id: i64) -> Result<Vec<EsiCorpMemberRole>> {
        self.get_json(&format!("/corporations/{corporation_id}/roles/"))
            .await
    }

    /// Fetch corporation member tracking info.
    #[tracing::instrument(skip(self))]
    pub async fn corp_member_tracking(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiCorpMemberTracking>> {
        self.get_json(&format!("/corporations/{corporation_id}/membertracking/"))
            .await
    }

    /// Fetch corporation member limit.
    #[tracing::instrument(skip(self))]
    pub async fn corp_member_limit(&self, corporation_id: i64) -> Result<i32> {
        self.get_json(&format!("/corporations/{corporation_id}/members/limit/"))
            .await
    }

    // -----------------------------------------------------------------------
    // Corp structure endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation structures (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_structures(&self, corporation_id: i64) -> Result<Vec<EsiCorpStructure>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/structures/"))
            .await
    }

    /// Fetch corporation starbases (POSes).
    #[tracing::instrument(skip(self))]
    pub async fn corp_starbases(&self, corporation_id: i64) -> Result<Vec<EsiCorpStarbase>> {
        self.get_json(&format!("/corporations/{corporation_id}/starbases/"))
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
            "/corporations/{corporation_id}/starbases/{starbase_id}/?system_id={system_id}"
        ))
        .await
    }

    // -----------------------------------------------------------------------
    // Corp contacts endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation contacts (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_contacts(&self, corporation_id: i64) -> Result<Vec<EsiContact>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/contacts/"))
            .await
    }

    /// Fetch corporation contact labels (authenticated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_contact_labels(&self, corporation_id: i64) -> Result<Vec<EsiContactLabel>> {
        self.get_json(&format!("/corporations/{corporation_id}/contacts/labels/"))
            .await
    }

    // -----------------------------------------------------------------------
    // Corp additional endpoints (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch corporation container audit logs (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_container_logs(&self, corporation_id: i64) -> Result<Vec<EsiContainerLog>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/containers/logs/"))
            .await
    }

    /// Fetch corporation customs offices (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_customs_offices(&self, corporation_id: i64) -> Result<Vec<EsiCustomsOffice>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/customs_offices/"))
            .await
    }

    /// Fetch corporation divisions.
    #[tracing::instrument(skip(self))]
    pub async fn corp_divisions(&self, corporation_id: i64) -> Result<EsiCorpDivisions> {
        self.get_json(&format!("/corporations/{corporation_id}/divisions/"))
            .await
    }

    /// Fetch corporation facilities.
    #[tracing::instrument(skip(self))]
    pub async fn corp_facilities(&self, corporation_id: i64) -> Result<Vec<EsiCorpFacility>> {
        self.get_json(&format!("/corporations/{corporation_id}/facilities/"))
            .await
    }

    /// Fetch corporation FW stats.
    #[tracing::instrument(skip(self))]
    pub async fn corp_fw_stats(&self, corporation_id: i64) -> Result<EsiCorpFwStats> {
        self.get_json(&format!("/corporations/{corporation_id}/fw/stats/"))
            .await
    }

    /// Fetch corporation icon URLs.
    #[tracing::instrument(skip(self))]
    pub async fn corp_icons(&self, corporation_id: i64) -> Result<EsiCorpIcons> {
        self.get_json(&format!("/corporations/{corporation_id}/icons/"))
            .await
    }

    /// Fetch corporation medals (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_medals(&self, corporation_id: i64) -> Result<Vec<EsiCorpMedal>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/medals/"))
            .await
    }

    /// Fetch issued medals (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_medals_issued(&self, corporation_id: i64) -> Result<Vec<EsiIssuedMedal>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/medals/issued/"))
            .await
    }

    /// Fetch corporation role change history (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_roles_history(&self, corporation_id: i64) -> Result<Vec<EsiRoleHistory>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/roles/history/"))
            .await
    }

    /// Fetch corporation shareholders (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_shareholders(&self, corporation_id: i64) -> Result<Vec<EsiShareholder>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/shareholders/"))
            .await
    }

    /// Fetch corporation standings (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_standings(&self, corporation_id: i64) -> Result<Vec<EsiStanding>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/standings/"))
            .await
    }

    /// Fetch corporation titles.
    #[tracing::instrument(skip(self))]
    pub async fn corp_titles(&self, corporation_id: i64) -> Result<Vec<EsiCorpTitle>> {
        self.get_json(&format!("/corporations/{corporation_id}/titles/"))
            .await
    }

    // -----------------------------------------------------------------------
    // Corporation mining endpoints (note: singular /corporation/ path)
    // -----------------------------------------------------------------------

    /// Fetch moon mining extractions (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_mining_extractions(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiMiningExtraction>> {
        self.get_paginated_json(&format!(
            "/corporation/{corporation_id}/mining/extractions/"
        ))
        .await
    }

    /// Fetch mining observers (paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corp_mining_observers(
        &self,
        corporation_id: i64,
    ) -> Result<Vec<EsiMiningObserver>> {
        self.get_paginated_json(&format!("/corporation/{corporation_id}/mining/observers/"))
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
            "/corporation/{corporation_id}/mining/observers/{observer_id}/"
        ))
        .await
    }
}
