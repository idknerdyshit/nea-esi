use crate::{EsiClient, EsiKillmail, EsiKillmailRef, Result};

impl EsiClient {
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
        self.get_json(&format!("/killmails/{killmail_id}/{killmail_hash}/"))
            .await
    }

    /// Fetch a single killmail by ID and hash, returning a typed struct.
    #[tracing::instrument(skip(self))]
    pub async fn get_killmail_typed(
        &self,
        killmail_id: i64,
        killmail_hash: &str,
    ) -> Result<EsiKillmail> {
        self.get_json(&format!("/killmails/{killmail_id}/{killmail_hash}/"))
            .await
    }

    /// Fetch recent killmails for a character (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn character_killmails(&self, character_id: i64) -> Result<Vec<EsiKillmailRef>> {
        self.get_paginated_json(&format!("/characters/{character_id}/killmails/recent/"))
            .await
    }

    /// Fetch recent killmails for a corporation (authenticated, paginated).
    #[tracing::instrument(skip(self))]
    pub async fn corporation_killmails(&self, corporation_id: i64) -> Result<Vec<EsiKillmailRef>> {
        self.get_paginated_json(&format!("/corporations/{corporation_id}/killmails/recent/"))
            .await
    }
}
