#[derive(clap::Subcommand)]
pub enum AssetsCommand {
    /// List character assets
    List,
    /// Get names for character asset item IDs
    Names {
        /// Item IDs
        item_ids: Vec<i64>,
    },
    /// Get locations for character asset item IDs
    Locations {
        /// Item IDs
        item_ids: Vec<i64>,
    },
    /// List corporation assets
    CorpList,
    /// Get names for corporation asset item IDs
    CorpNames {
        /// Item IDs
        item_ids: Vec<i64>,
    },
    /// Get locations for corporation asset item IDs
    CorpLocations {
        /// Item IDs
        item_ids: Vec<i64>,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: AssetsCommand) -> anyhow::Result<()> {
    match cmd {
        AssetsCommand::List => {
            let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_assets(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        AssetsCommand::Names { item_ids } => {
            let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_asset_names(cid, &item_ids).await?;
            crate::output::print_list(&result, ctx.format)
        }
        AssetsCommand::Locations { item_ids } => {
            let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_asset_locations(cid, &item_ids).await?;
            crate::output::print_list(&result, ctx.format)
        }
        AssetsCommand::CorpList => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_assets(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        AssetsCommand::CorpNames { item_ids } => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_asset_names(corp_id, &item_ids).await?;
            crate::output::print_list(&result, ctx.format)
        }
        AssetsCommand::CorpLocations { item_ids } => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_asset_locations(corp_id, &item_ids).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
