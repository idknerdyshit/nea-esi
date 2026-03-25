#[derive(clap::Subcommand)]
pub enum IndustryCommand {
    /// Get character industry jobs
    Jobs,
    /// Get character blueprints
    Blueprints,
    /// Get corporation industry jobs
    CorpJobs,
    /// Get corporation blueprints
    CorpBlueprints,
    /// List public industry facilities
    Facilities,
    /// List industry cost indices per system
    Systems,
}

pub async fn execute(ctx: &super::ExecContext, cmd: IndustryCommand) -> anyhow::Result<()> {
    match cmd {
        IndustryCommand::Jobs => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_industry_jobs(cid, false).await?;
            crate::output::print_list(&result, ctx.format)
        }
        IndustryCommand::Blueprints => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_blueprints(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        IndustryCommand::CorpJobs => {
            let corp_id = ctx
                .corporation_id
                .ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_industry_jobs(corp_id, false).await?;
            crate::output::print_list(&result, ctx.format)
        }
        IndustryCommand::CorpBlueprints => {
            let corp_id = ctx
                .corporation_id
                .ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_blueprints(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        IndustryCommand::Facilities => {
            let result = ctx.client.industry_facilities().await?;
            crate::output::print_list(&result, ctx.format)
        }
        IndustryCommand::Systems => {
            let result = ctx.client.industry_systems().await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
