use crate::{
    EsiCharacterFleet, EsiClient, EsiFleetInfo, EsiFleetInvitation, EsiFleetMember,
    EsiFleetMovement, EsiFleetNaming, EsiFleetSquadCreated, EsiFleetUpdate, EsiFleetWing,
    EsiFleetWingCreated, Result,
};

impl EsiClient {
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
    // Fleet write endpoints
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
}
