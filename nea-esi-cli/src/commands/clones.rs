#[derive(clap::Subcommand)]
pub enum ClonesCommand {
    /// Get character clone info
    List,
    /// Get character active implants
    Implants,
    /// Get character jump fatigue
    Fatigue,
}

pub async fn execute(ctx: &super::ExecContext, cmd: ClonesCommand) -> anyhow::Result<()> {
    let cid = ctx
        .character_id
        .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
    match cmd {
        ClonesCommand::List => {
            let result = ctx.client.character_clones(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        ClonesCommand::Implants => {
            let result = ctx.client.character_implants(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        ClonesCommand::Fatigue => {
            let result = ctx.client.character_fatigue(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
    }
}
