#[derive(clap::Subcommand)]
pub enum SovereigntyCommand {
    /// Get sovereignty map
    Map,
    /// Get sovereignty campaigns
    Campaigns,
    /// Get sovereignty structures
    Structures,
}

pub async fn execute(ctx: &super::ExecContext, cmd: SovereigntyCommand) -> anyhow::Result<()> {
    match cmd {
        SovereigntyCommand::Map => {
            let result = ctx.client.sovereignty_map().await?;
            crate::output::print_list(&result, ctx.format)
        }
        SovereigntyCommand::Campaigns => {
            let result = ctx.client.sovereignty_campaigns().await?;
            crate::output::print_list(&result, ctx.format)
        }
        SovereigntyCommand::Structures => {
            let result = ctx.client.sovereignty_structures().await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
