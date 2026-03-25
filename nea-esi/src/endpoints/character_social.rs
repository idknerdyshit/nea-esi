use crate::{
    EsiCalendarEvent, EsiCalendarEventDetail, EsiCharacterOrder,
    EsiClient, EsiContact, EsiContactLabel, EsiContract, EsiContractBid, EsiContractItem,
    EsiError, EsiFitting, EsiMailBody, EsiMailHeader, EsiMailLabels, EsiMailUpdate, EsiMailingList,
    EsiNewFitting, EsiNewFittingResponse, EsiNewMail, EsiNewMailLabel, EsiNotification,
    EsiSearchResult, Result,
};

impl EsiClient {
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
    // Search endpoints
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
}
