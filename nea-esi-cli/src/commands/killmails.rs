#[derive(clap::Subcommand)]
pub enum KillmailsCommand {
    /// Get a killmail by ID and hash
    Get {
        /// Killmail ID
        killmail_id: i64,
        /// Killmail hash
        hash: String,
    },
    /// List character killmail refs (requires auth)
    Character,
    /// List corporation killmail refs (requires auth)
    Corporation,
    /// List war killmail refs
    War {
        /// War ID
        war_id: i32,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: KillmailsCommand) -> anyhow::Result<()> {
    match cmd {
        KillmailsCommand::Get { killmail_id, hash } => {
            let result = ctx.client.get_killmail(killmail_id, &hash).await?;
            crate::output::print_value(&result, ctx.format)
        }
        KillmailsCommand::Character => {
            let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_killmails(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        KillmailsCommand::Corporation => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corporation_killmails(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        KillmailsCommand::War { war_id } => {
            let result = ctx.client.war_killmails(war_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
