#[derive(clap::Subcommand)]
pub enum FleetCommand {
    /// Get your current fleet info (requires auth)
    MyFleet,
    /// Get fleet info
    Info {
        /// Fleet ID
        fleet_id: i64,
    },
    /// List fleet members
    Members {
        /// Fleet ID
        fleet_id: i64,
    },
    /// List fleet wings
    Wings {
        /// Fleet ID
        fleet_id: i64,
    },
    /// Kick a fleet member
    Kick {
        /// Fleet ID
        fleet_id: i64,
        /// Member character ID
        member_id: i64,
    },
    /// Create a fleet wing
    CreateWing {
        /// Fleet ID
        fleet_id: i64,
    },
    /// Delete a fleet wing
    DeleteWing {
        /// Fleet ID
        fleet_id: i64,
        /// Wing ID
        wing_id: i64,
    },
    /// Rename a fleet wing
    RenameWing {
        /// Fleet ID
        fleet_id: i64,
        /// Wing ID
        wing_id: i64,
        /// New name
        name: String,
    },
    /// Create a fleet squad
    CreateSquad {
        /// Fleet ID
        fleet_id: i64,
        /// Wing ID
        wing_id: i64,
    },
    /// Delete a fleet squad
    DeleteSquad {
        /// Fleet ID
        fleet_id: i64,
        /// Squad ID
        squad_id: i64,
    },
    /// Rename a fleet squad
    RenameSquad {
        /// Fleet ID
        fleet_id: i64,
        /// Squad ID
        squad_id: i64,
        /// New name
        name: String,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: FleetCommand) -> anyhow::Result<()> {
    match cmd {
        FleetCommand::MyFleet => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_fleet(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        FleetCommand::Info { fleet_id } => {
            let result = ctx.client.get_fleet(fleet_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        FleetCommand::Members { fleet_id } => {
            let result = ctx.client.fleet_members(fleet_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        FleetCommand::Wings { fleet_id } => {
            let result = ctx.client.fleet_wings(fleet_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        FleetCommand::Kick {
            fleet_id,
            member_id,
        } => {
            ctx.client.kick_fleet_member(fleet_id, member_id).await?;
            println!("Member {member_id} kicked from fleet.");
            Ok(())
        }
        FleetCommand::CreateWing { fleet_id } => {
            let result = ctx.client.create_fleet_wing(fleet_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        FleetCommand::DeleteWing { fleet_id, wing_id } => {
            ctx.client.delete_fleet_wing(fleet_id, wing_id).await?;
            println!("Wing {wing_id} deleted.");
            Ok(())
        }
        FleetCommand::RenameWing {
            fleet_id,
            wing_id,
            name,
        } => {
            ctx.client
                .rename_fleet_wing(fleet_id, wing_id, &name)
                .await?;
            println!("Wing {wing_id} renamed.");
            Ok(())
        }
        FleetCommand::CreateSquad { fleet_id, wing_id } => {
            let result = ctx.client.create_fleet_squad(fleet_id, wing_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        FleetCommand::DeleteSquad { fleet_id, squad_id } => {
            ctx.client.delete_fleet_squad(fleet_id, squad_id).await?;
            println!("Squad {squad_id} deleted.");
            Ok(())
        }
        FleetCommand::RenameSquad {
            fleet_id,
            squad_id,
            name,
        } => {
            ctx.client
                .rename_fleet_squad(fleet_id, squad_id, &name)
                .await?;
            println!("Squad {squad_id} renamed.");
            Ok(())
        }
    }
}
