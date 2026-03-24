#[derive(clap::Subcommand)]
pub enum ResolveCommand {
    /// Resolve IDs to names
    Names {
        /// Entity IDs
        #[arg(required = true)]
        ids: Vec<i64>,
    },
    /// Resolve names to IDs
    Ids {
        /// Entity names
        #[arg(required = true)]
        names: Vec<String>,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: ResolveCommand) -> anyhow::Result<()> {
    match cmd {
        ResolveCommand::Names { ids } => {
            let result = ctx.client.resolve_names(&ids).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ResolveCommand::Ids { names } => {
            let result = ctx.client.resolve_ids(&names).await?;
            crate::output::print_value(&result, ctx.format)
        }
    }
}
