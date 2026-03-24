use crate::{
    EsiAllianceIcons, EsiAllianceInfo, EsiClient, EsiContact, EsiContactLabel, Result,
};

impl EsiClient {
    // -----------------------------------------------------------------------
    // Alliance endpoints
    // -----------------------------------------------------------------------

    /// Fetch alliance info from ESI.
    #[tracing::instrument(skip(self))]
    pub async fn get_alliance(&self, alliance_id: i64) -> Result<EsiAllianceInfo> {
        self.get_json(&format!("/alliances/{}/", alliance_id)).await
    }

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
}
