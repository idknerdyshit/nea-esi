#[derive(clap::Subcommand)]
pub enum AllianceCommand {
    /// Get alliance public info
    Info {
        /// Alliance ID
        id: i64,
    },
    /// Get alliance icons
    Icons {
        /// Alliance ID
        id: i64,
    },
    /// List corporation IDs in an alliance
    Corporations {
        /// Alliance ID
        id: i64,
    },
    /// Get alliance contacts (requires auth)
    Contacts {
        /// Alliance ID
        id: i64,
    },
    /// Get alliance contact labels (requires auth)
    ContactLabels {
        /// Alliance ID
        id: i64,
    },
    /// List all alliance IDs
    List,
}

pub async fn execute(ctx: &super::ExecContext, cmd: AllianceCommand) -> anyhow::Result<()> {
    match cmd {
        AllianceCommand::Info { id } => {
            let result = ctx.client.get_alliance(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        AllianceCommand::Icons { id } => {
            let result = ctx.client.alliance_icons(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        AllianceCommand::Corporations { id } => {
            let result = ctx.client.alliance_corporations(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        AllianceCommand::Contacts { id } => {
            let result = ctx.client.alliance_contacts(id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        AllianceCommand::ContactLabels { id } => {
            let result = ctx.client.alliance_contact_labels(id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        AllianceCommand::List => {
            let result = ctx.client.list_alliance_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
    }
}
