// ESI endpoint methods — split into domain modules.

mod alliance;
mod character;
mod character_social;
mod corporation;
mod fleet;
mod killmail;
mod market;
mod misc;
mod universe;

pub use market::compute_best_bid_ask;

use serde::de::DeserializeOwned;

use crate::{EsiClient, EsiError, Result};

pub(crate) const RESOLVE_NAMES_CHUNK_SIZE: usize = 1000;
pub(crate) const RESOLVE_IDS_CHUNK_SIZE: usize = 500;
pub(crate) const ASSET_NAMES_CHUNK_SIZE: usize = 1000;
pub(crate) const AFFILIATION_CHUNK_SIZE: usize = 1000;

impl EsiClient {
    // -----------------------------------------------------------------------
    // Private helpers (pub(crate) so submodules can use them)
    // -----------------------------------------------------------------------

    /// GET a path relative to `base_url` and deserialize the JSON response.
    pub(crate) async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.request(&url).await?;
        resp.json()
            .await
            .map_err(|e| EsiError::Deserialize(e.to_string()))
    }

    /// POST a path relative to `base_url` with a JSON body, deserialize the response.
    pub(crate) async fn post_json<T: DeserializeOwned>(
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
    pub(crate) async fn delete_path(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let _resp = self.request_delete(&url).await?;
        Ok(())
    }

    /// DELETE a full URL (including query params), discarding the response body.
    pub(crate) async fn delete_url(&self, url: &str) -> Result<()> {
        let _resp = self.request_delete(url).await?;
        Ok(())
    }

    /// PUT a path relative to `base_url` with a JSON body, discarding the response.
    pub(crate) async fn put_json(
        &self,
        path: &str,
        body: &(impl serde::Serialize + ?Sized),
    ) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let _resp = self.request_put(&url, body).await?;
        Ok(())
    }

    /// POST a path with a JSON body, discarding the response body.
    pub(crate) async fn post_json_void(
        &self,
        path: &str,
        body: &(impl serde::Serialize + ?Sized),
    ) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let _resp = self.request_post(&url, body).await?;
        Ok(())
    }

    /// POST a full URL with an empty body (for UI command endpoints).
    pub(crate) async fn post_empty_url(&self, url: &str) -> Result<()> {
        let _resp = self.request_post(url, &serde_json::json!({})).await?;
        Ok(())
    }

    /// Build a full URL from a base path with query parameters, using `Url::parse_with_params`.
    pub(crate) fn build_url(&self, path: &str, params: &[(&str, &str)]) -> Result<url::Url> {
        let base = format!("{}{}", self.base_url, path);
        url::Url::parse_with_params(&base, params)
            .map_err(|e| EsiError::Internal(format!("failed to build URL: {e}")))
    }

    /// Build a contact endpoint URL with standing and optional label/watched params.
    pub(crate) fn build_contact_url(
        &self,
        character_id: i64,
        standing: f64,
        label_ids: Option<&[i64]>,
        watched: Option<bool>,
    ) -> Result<url::Url> {
        let base = format!("{}/characters/{}/contacts/", self.base_url, character_id);
        let mut url = url::Url::parse(&base)
            .map_err(|e| EsiError::Internal(format!("failed to build URL: {e}")))?;
        let standing_str = standing.to_string();
        url.query_pairs_mut().append_pair("standing", &standing_str);
        if let Some(labels) = label_ids {
            for label in labels {
                url.query_pairs_mut()
                    .append_pair("label_ids", &label.to_string());
            }
        }
        if let Some(w) = watched {
            url.query_pairs_mut().append_pair("watched", &w.to_string());
        }
        Ok(url)
    }

    /// GET a paginated path relative to `base_url` and collect all pages.
    pub(crate) async fn get_paginated_json<T: DeserializeOwned + Send + 'static>(
        &self,
        path: &str,
    ) -> Result<Vec<T>> {
        let url = format!("{}{}", self.base_url, path);
        self.get_paginated::<T>(&url).await
    }
}
