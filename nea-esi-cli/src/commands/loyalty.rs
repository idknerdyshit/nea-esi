#[derive(clap::Subcommand)]
pub enum LoyaltyCommand {
    /// Get character loyalty points
    Points,
    /// Get loyalty store offers for a corporation
    Offers {
        /// Corporation ID
        corporation_id: i64,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: LoyaltyCommand) -> anyhow::Result<()> {
    match cmd {
        LoyaltyCommand::Points => {
            let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_loyalty_points(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        LoyaltyCommand::Offers { corporation_id } => {
            let result = ctx.client.loyalty_store_offers(corporation_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
