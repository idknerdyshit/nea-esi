#[derive(clap::Subcommand)]
pub enum PiCommand {
    /// List character planetary colonies
    Planets,
    /// Get planet colony details
    Planet {
        /// Planet ID
        planet_id: i32,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: PiCommand) -> anyhow::Result<()> {
    let cid = ctx
        .character_id
        .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
    match cmd {
        PiCommand::Planets => {
            let result = ctx.client.character_planets(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        PiCommand::Planet { planet_id } => {
            let result = ctx.client.character_planet_detail(cid, planet_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
    }
}
