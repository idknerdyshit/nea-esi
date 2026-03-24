#[derive(clap::Subcommand)]
pub enum FwCommand {
    /// Get faction warfare statistics
    Stats,
    /// Get faction warfare systems
    Systems,
    /// Get faction warfare leaderboards
    Leaderboards,
    /// Get faction warfare wars
    Wars,
    /// Get character faction warfare leaderboards
    CharacterLeaderboards,
    /// Get corporation faction warfare leaderboards
    CorporationLeaderboards,
}

pub async fn execute(ctx: &super::ExecContext, cmd: FwCommand) -> anyhow::Result<()> {
    match cmd {
        FwCommand::Stats => {
            let result = ctx.client.fw_stats().await?;
            crate::output::print_list(&result, ctx.format)
        }
        FwCommand::Systems => {
            let result = ctx.client.fw_systems().await?;
            crate::output::print_list(&result, ctx.format)
        }
        FwCommand::Leaderboards => {
            let result = ctx.client.fw_leaderboards().await?;
            crate::output::print_value(&result, ctx.format)
        }
        FwCommand::Wars => {
            let result = ctx.client.fw_wars().await?;
            crate::output::print_list(&result, ctx.format)
        }
        FwCommand::CharacterLeaderboards => {
            let result = ctx.client.fw_character_leaderboards().await?;
            crate::output::print_value(&result, ctx.format)
        }
        FwCommand::CorporationLeaderboards => {
            let result = ctx.client.fw_corporation_leaderboards().await?;
            crate::output::print_value(&result, ctx.format)
        }
    }
}
