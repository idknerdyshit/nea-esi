#[derive(clap::Subcommand)]
pub enum FittingsCommand {
    /// List character fittings
    List,
    /// Delete a fitting
    Delete {
        /// Fitting ID
        fitting_id: i64,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: FittingsCommand) -> anyhow::Result<()> {
    let cid = ctx
        .character_id
        .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
    match cmd {
        FittingsCommand::List => {
            let result = ctx.client.character_fittings(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        FittingsCommand::Delete { fitting_id } => {
            ctx.client.delete_fitting(cid, fitting_id).await?;
            println!("Fitting {fitting_id} deleted.");
            Ok(())
        }
    }
}
