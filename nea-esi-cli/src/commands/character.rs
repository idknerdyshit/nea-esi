#[derive(clap::Subcommand)]
pub enum CharacterCommand {
    /// Get character public info
    Info {
        /// Character ID
        id: i64,
    },
    /// Get character portrait URLs
    Portrait {
        /// Character ID
        id: i64,
    },
    /// Get affiliations for character IDs
    Affiliation {
        /// Character IDs
        ids: Vec<i64>,
    },
    /// Get character roles (requires auth)
    Roles,
    /// Get character titles (requires auth)
    Titles,
    /// Get corporation history for a character
    CorporationHistory {
        /// Character ID
        id: i64,
    },
    /// Get character medals (requires auth)
    Medals,
    /// Get character agent research (requires auth)
    AgentsResearch,
    /// Get character jump fatigue (requires auth)
    Fatigue,
    /// Get character faction warfare stats (requires auth)
    FwStats,
    /// Get character fleet info (requires auth)
    Fleet,
    /// Get character standings (requires auth)
    Standings,
    /// Get character location (requires auth)
    Location,
    /// Get character current ship (requires auth)
    Ship,
    /// Get character online status (requires auth)
    Online,
    /// Get character completed opportunities (requires auth)
    Opportunities,
    /// Get character notifications (requires auth)
    Notifications,
    /// Get character contact notifications (requires auth)
    ContactNotifications,
    /// Get character killmail refs (requires auth)
    Killmails,
    /// Authenticated character search
    Search {
        /// Search query
        query: String,
        /// Comma-separated categories
        categories: String,
        /// Strict matching
        #[arg(long)]
        strict: bool,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: CharacterCommand) -> anyhow::Result<()> {
    match cmd {
        CharacterCommand::Info { id } => {
            let result = ctx.client.get_character(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CharacterCommand::Portrait { id } => {
            let result = ctx.client.character_portrait(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CharacterCommand::Affiliation { ids } => {
            let result = ctx.client.character_affiliation(&ids).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::Roles => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_roles(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CharacterCommand::Titles => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_titles(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::CorporationHistory { id } => {
            let result = ctx.client.character_corporation_history(id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::Medals => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_medals(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::AgentsResearch => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_agents_research(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::Fatigue => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_fatigue(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CharacterCommand::FwStats => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_fw_stats(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CharacterCommand::Fleet => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_fleet(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CharacterCommand::Standings => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_standings(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::Location => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_location(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CharacterCommand::Ship => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_ship(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CharacterCommand::Online => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_online(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        CharacterCommand::Opportunities => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_opportunities(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::Notifications => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_notifications(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::ContactNotifications => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_contact_notifications(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::Killmails => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_killmails(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        CharacterCommand::Search {
            query,
            categories,
            strict,
        } => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx
                .client
                .character_search(cid, &query, &categories, strict)
                .await?;
            crate::output::print_value(&result, ctx.format)
        }
    }
}
