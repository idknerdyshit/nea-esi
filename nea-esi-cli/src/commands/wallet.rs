#[derive(clap::Subcommand)]
pub enum WalletCommand {
    /// Get character wallet balance
    Balance,
    /// Get character wallet journal
    Journal,
    /// Get character wallet transactions
    Transactions {
        /// Fetch transactions before this ID
        #[arg(long)]
        from_id: Option<i64>,
    },
    /// Get corporation wallet balances
    CorpBalances,
    /// Get corporation wallet journal for a division
    CorpJournal {
        /// Wallet division (1-7)
        division: i32,
        /// Fetch entries before this ID
        #[arg(long)]
        before_id: Option<i64>,
    },
    /// Get corporation wallet transactions for a division
    CorpTransactions {
        /// Wallet division (1-7)
        division: i32,
        /// Fetch transactions before this ID
        #[arg(long)]
        from_id: Option<i64>,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: WalletCommand) -> anyhow::Result<()> {
    match cmd {
        WalletCommand::Balance => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.wallet_balance(cid).await?;
            crate::output::print_scalar(result, "balance", ctx.format);
            Ok(())
        }
        WalletCommand::Journal => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.wallet_journal(cid).await?;
            crate::output::print_list(&result, ctx.format)
        }
        WalletCommand::Transactions { from_id } => {
            let cid = ctx
                .character_id
                .ok_or_else(|| anyhow::anyhow!("No character ID specified"))?;
            let result = ctx.client.wallet_transactions(cid, from_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        WalletCommand::CorpBalances => {
            let corp_id = ctx
                .corporation_id
                .ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_wallet_balances(corp_id).await?;
            crate::output::print_list(&result, ctx.format)
        }
        WalletCommand::CorpJournal {
            division,
            before_id: _,
        } => {
            let corp_id = ctx
                .corporation_id
                .ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx.client.corp_wallet_journal(corp_id, division).await?;
            crate::output::print_list(&result, ctx.format)
        }
        WalletCommand::CorpTransactions { division, from_id } => {
            let corp_id = ctx
                .corporation_id
                .ok_or_else(|| anyhow::anyhow!("No corporation ID specified"))?;
            let result = ctx
                .client
                .corp_wallet_transactions(corp_id, division, from_id)
                .await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
