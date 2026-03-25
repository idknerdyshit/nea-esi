#[derive(clap::Subcommand)]
pub enum CorporationCommand {
    /// Get corporation public info
    Info {
        /// Corporation ID
        id: i64,
    },
    /// Get corporation alliance history
    AllianceHistory {
        /// Corporation ID
        id: i64,
    },
    /// Get corporation icons
    Icons,
    /// Get corporation member limit (requires auth)
    MemberLimit,
    /// List corporation member IDs (requires auth)
    Members,
    /// Get corporation member tracking (requires auth)
    MemberTracking,
    /// Get corporation member titles (requires auth)
    MemberTitles,
    /// Get corporation member roles (requires auth)
    MemberRoles,
    /// Get corporation roles change history (requires auth)
    RolesHistory,
    /// Get corporation structures (requires auth)
    Structures,
    /// Get corporation starbases (requires auth)
    Starbases,
    /// Get starbase detail (requires auth)
    StarbaseDetail {
        /// Starbase ID
        starbase_id: i64,
        /// System ID the starbase is in
        system_id: i32,
    },
    /// Get corporation divisions (requires auth)
    Divisions,
    /// Get corporation facilities (requires auth)
    Facilities,
    /// Get corporation faction warfare stats (requires auth)
    FwStats,
    /// Get corporation medals (requires auth)
    Medals,
    /// Get corporation issued medals (requires auth)
    MedalsIssued,
    /// Get corporation container logs (requires auth)
    ContainerLogs,
    /// Get corporation customs offices (requires auth)
    CustomsOffices {
        /// Filter by system ID
        #[arg(long)]
        system_id: Option<i32>,
    },
    /// Get corporation shareholders (requires auth)
    Shareholders,
    /// Get corporation titles (requires auth)
    Titles,
    /// Get corporation contacts (requires auth)
    Contacts,
    /// Get corporation contact labels (requires auth)
    ContactLabels,
    /// Get corporation standings (requires auth)
    Standings,
}

pub async fn execute(ctx: &super::ExecContext, cmd: CorporationCommand) -> anyhow::Result<()> {
    match cmd {
        CorporationCommand::Info { id } => {
            let result = ctx.client.get_corporation(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CorporationCommand::AllianceHistory { id } => {
            let result = ctx.client.corp_alliance_history(id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::Icons => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_icons(corp_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CorporationCommand::MemberLimit => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_member_limit(corp_id).await?;
            crate::output::print_scalar(result, "member_limit", ctx.format)
        }
        CorporationCommand::Members => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_members(corp_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CorporationCommand::MemberTracking => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_member_tracking(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::MemberTitles => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_member_titles(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::MemberRoles => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_member_roles(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::RolesHistory => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_roles_history(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::Structures => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_structures(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::Starbases => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_starbases(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::StarbaseDetail { starbase_id, system_id } => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_starbase_detail(corp_id, starbase_id, system_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CorporationCommand::Divisions => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_divisions(corp_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CorporationCommand::Facilities => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_facilities(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::FwStats => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_fw_stats(corp_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CorporationCommand::Medals => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_medals(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::MedalsIssued => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_medals_issued(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::ContainerLogs => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_container_logs(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::CustomsOffices { system_id: _ } => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_customs_offices(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::Shareholders => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_shareholders(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::Titles => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_titles(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::Contacts => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_contacts(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::ContactLabels => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_contact_labels(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CorporationCommand::Standings => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_standings(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
