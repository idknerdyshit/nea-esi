#[derive(clap::Subcommand)]
pub enum SkillsCommand {
    /// Get character skills
    List,
    /// Get character skill queue
    Queue,
    /// Get character attributes
    Attributes,
    /// Get character active implants
    Implants,
}

pub async fn execute(ctx: &super::ExecContext, cmd: SkillsCommand) -> anyhow::Result<()> {
    let cid = ctx.character_id.ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
    match cmd {
        SkillsCommand::List => {
            let result = ctx.client.character_skills(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        SkillsCommand::Queue => {
            let result = ctx.client.character_skillqueue(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        SkillsCommand::Attributes => {
            let result = ctx.client.character_attributes(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
        SkillsCommand::Implants => {
            let result = ctx.client.character_implants(cid).await?;
            crate::output::print_value(&result, ctx.format)
        }
    }
}
