#[derive(clap::Subcommand)]
pub enum MarketCommand {
    /// Get market history for a type in a region
    History {
        /// Region ID
        region_id: i32,
        /// Type ID
        type_id: i32,
    },
    /// Get market orders for a type in a region
    Orders {
        /// Region ID
        region_id: i32,
        /// Type ID
        type_id: i32,
        /// Order type: buy, sell, or all
        #[arg(long)]
        order_type: Option<String>,
    },
    /// Get average market prices for all types
    Prices,
    /// List type IDs on the market in a region
    Types {
        /// Region ID
        region_id: i32,
    },
    /// List all market group IDs
    Groups,
    /// Get details for a market group
    Group {
        /// Market group ID
        market_group_id: i32,
    },
    /// Get orders from a structure
    StructureOrders {
        /// Structure ID
        structure_id: i64,
        /// Order type: buy, sell, or all
        #[arg(long)]
        order_type: Option<String>,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: MarketCommand) -> anyhow::Result<()> {
    match cmd {
        MarketCommand::History { region_id, type_id } => {
            let result = ctx.client.market_history(region_id, type_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        MarketCommand::Orders {
            region_id,
            type_id,
            order_type,
        } => {
            let result = ctx
                .client
                .market_orders(region_id, type_id, order_type.as_deref())
                .await?;
            crate::output::print_list(&result, ctx.format)
        }
        MarketCommand::Prices => {
            let result = ctx.client.market_prices().await?;
            crate::output::print_list(&result, ctx.format)
        }
        MarketCommand::Types { region_id } => {
            let result = ctx.client.market_type_ids(region_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        MarketCommand::Groups => {
            let result = ctx.client.market_group_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        MarketCommand::Group { market_group_id } => {
            let result = ctx.client.get_market_group(market_group_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        MarketCommand::StructureOrders {
            structure_id,
            order_type: _,
        } => {
            let result = ctx.client.structure_orders(structure_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
