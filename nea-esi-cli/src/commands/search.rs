#[derive(clap::Subcommand)]
pub enum SearchCommand {
    /// Public search
    Public {
        /// Search query
        query: String,
        /// Comma-separated categories
        categories: String,
        /// Strict matching
        #[arg(long)]
        strict: bool,
    },
    /// Authenticated character search
    Character {
        /// Search query
        query: String,
        /// Comma-separated categories
        categories: String,
        /// Strict matching
        #[arg(long)]
        strict: bool,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: SearchCommand) -> anyhow::Result<()> {
    match cmd {
        SearchCommand::Public {
            query,
            categories,
            strict,
        } => {
            let result = ctx.client.search(&query, &categories, strict).await?;
            crate::output::print_value(&result, ctx.format)
        }
        SearchCommand::Character {
            query,
            categories,
            strict,
        } => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx
                .client
                .character_search(cid, &query, &categories, strict)
                .await?;
            crate::output::print_value(&result, ctx.format)
        }
    }
}
