#[derive(clap::Subcommand)]
pub enum MiningCommand {
    /// Get character mining ledger
    Ledger,
    /// List corporation mining observers
    Observers,
    /// Get details for a corporation mining observer
    ObserverDetails {
        /// Observer ID
        observer_id: i64,
    },
    /// Get corporation mining extractions
    Extractions,
}

pub async fn execute(ctx: &super::ExecContext, cmd: MiningCommand) -> anyhow::Result<()> {
    match cmd {
        MiningCommand::Ledger => {
            let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_mining_ledger(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        MiningCommand::Observers => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_mining_observers(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        MiningCommand::ObserverDetails { observer_id } => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_mining_observer_details(corp_id, observer_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        MiningCommand::Extractions => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_mining_extractions(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
