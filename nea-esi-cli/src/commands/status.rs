pub async fn execute(ctx: &super::ExecContext) -> anyhow::Result<()> {
    let status = ctx.client.server_status().await?;
    crate::output::print_value(&status, ctx.format)
}
