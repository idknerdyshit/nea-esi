#[derive(clap::Subcommand)]
pub enum WarsCommand {
    /// List war IDs
    List {
        /// Fetch wars before this ID
        #[arg(long)]
        max_war_id: Option<i32>,
    },
    /// Get war details
    Get {
        /// War ID
        war_id: i32,
    },
    /// Get war killmail refs
    Killmails {
        /// War ID
        war_id: i32,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: WarsCommand) -> anyhow::Result<()> {
    match cmd {
        WarsCommand::List { max_war_id } => {
            let result = ctx.client.list_war_ids(max_war_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        WarsCommand::Get { war_id } => {
            let result = ctx.client.get_war(war_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        WarsCommand::Killmails { war_id } => {
            let result = ctx.client.war_killmails(war_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
