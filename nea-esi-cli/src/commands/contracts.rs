#[derive(clap::Subcommand)]
pub enum ContractsCommand {
    /// List character contracts
    List,
    /// Get character contract items
    Items {
        /// Contract ID
        contract_id: i64,
    },
    /// Get character contract bids
    Bids {
        /// Contract ID
        contract_id: i64,
    },
    /// List corporation contracts
    CorpList,
    /// Get corporation contract items
    CorpItems {
        /// Contract ID
        contract_id: i64,
    },
    /// Get corporation contract bids
    CorpBids {
        /// Contract ID
        contract_id: i64,
    },
    /// List public contracts in a region
    Public {
        /// Region ID
        region_id: i32,
    },
    /// Get public contract items
    PublicItems {
        /// Contract ID
        contract_id: i64,
        /// Region ID
        region_id: i32,
    },
    /// Get public contract bids
    PublicBids {
        /// Contract ID
        contract_id: i64,
        /// Region ID
        region_id: i32,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: ContractsCommand) -> anyhow::Result<()> {
    match cmd {
        ContractsCommand::List => {
            let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_contracts(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContractsCommand::Items { contract_id } => {
            let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_contract_items(cid, contract_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContractsCommand::Bids { contract_id } => {
            let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.character_contract_bids(cid, contract_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContractsCommand::CorpList => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_contracts(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContractsCommand::CorpItems { contract_id } => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_contract_items(corp_id, contract_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContractsCommand::CorpBids { contract_id } => {
            let corp_id = ctx.corporation_id.ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_contract_bids(corp_id, contract_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContractsCommand::Public { region_id } => {
            let result = ctx.client.public_contracts(region_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContractsCommand::PublicItems { contract_id, region_id: _ } => {
            let result = ctx.client.public_contract_items(contract_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        ContractsCommand::PublicBids { contract_id, region_id: _ } => {
            let result = ctx.client.public_contract_bids(contract_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
