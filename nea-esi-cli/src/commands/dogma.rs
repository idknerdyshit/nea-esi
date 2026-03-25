#[derive(clap::Subcommand)]
pub enum DogmaCommand {
    /// Get a dogma attribute
    Attribute {
        /// Attribute ID
        id: i32,
    },
    /// Get a dogma effect
    Effect {
        /// Effect ID
        id: i32,
    },
    /// Get a dynamic item (mutaplasmid result)
    DynamicItem {
        /// Type ID
        type_id: i32,
        /// Item ID
        item_id: i64,
    },
    /// List all dogma attribute IDs
    Attributes,
    /// List all dogma effect IDs
    Effects,
}

pub async fn execute(ctx: &super::ExecContext, cmd: DogmaCommand) -> anyhow::Result<()> {
    match cmd {
        DogmaCommand::Attribute { id } => {
            let result = ctx.client.get_dogma_attribute(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        DogmaCommand::Effect { id } => {
            let result = ctx.client.get_dogma_effect(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        DogmaCommand::DynamicItem { type_id, item_id } => {
            let result = ctx.client.get_dynamic_item(type_id, item_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        DogmaCommand::Attributes => {
            let result = ctx.client.list_dogma_attribute_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        DogmaCommand::Effects => {
            let result = ctx.client.list_dogma_effect_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
    }
}
