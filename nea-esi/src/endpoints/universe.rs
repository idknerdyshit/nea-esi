use tracing::debug;

use crate::{
    EsiAncestry, EsiAsteroidBelt, EsiBloodline, EsiCategoryInfo, EsiClient,
    EsiConstellationInfo, EsiError, EsiFaction, EsiGraphic, EsiGroupInfo, EsiMoon, EsiPlanet,
    EsiRace, EsiRegionInfo, EsiResolvedIds, EsiResolvedName, EsiSchematic, EsiSolarSystemInfo,
    EsiSovereigntyCampaign, EsiSovereigntyMap, EsiSovereigntyStructure, EsiStar, EsiStargateInfo,
    EsiStationInfo, EsiStructureInfo, EsiSystemJumps, EsiSystemKills, EsiTypeInfo, Result,
};

use super::{RESOLVE_IDS_CHUNK_SIZE, RESOLVE_NAMES_CHUNK_SIZE};

impl EsiClient {
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
    // Structure endpoint (authenticated)
    // -----------------------------------------------------------------------

    /// Fetch info about a player-owned structure.
    #[tracing::instrument(skip(self))]
    pub async fn get_structure(&self, structure_id: i64) -> Result<EsiStructureInfo> {
        self.get_json(&format!("/universe/structures/{}/", structure_id))
            .await
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
    // Universe additional endpoints
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
