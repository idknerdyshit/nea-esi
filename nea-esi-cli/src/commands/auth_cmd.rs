#[derive(clap::Subcommand)]
pub enum AuthCommand {
    /// Log in via EVE SSO
    Login {
        /// Comma-separated list of scopes to request
        #[arg(long, value_delimiter = ',')]
        scopes: Vec<String>,
        /// Request all available scopes (including write)
        #[arg(long)]
        all_scopes: bool,
        /// Use copy-paste mode instead of opening a browser
        #[arg(long)]
        headless: bool,
    },
    /// Remove stored tokens
    Logout,
    /// Show current auth status
    Status,
}

pub async fn execute(ctx: &super::ExecContext, cmd: AuthCommand) -> anyhow::Result<()> {
    match cmd {
        AuthCommand::Login {
            scopes,
            all_scopes,
            headless,
        } => {
            let config = crate::config::Config::load(ctx.config_path.as_ref())?;
            let opts = crate::auth::LoginOptions {
                scopes: if scopes.is_empty() {
                    None
                } else {
                    Some(scopes)
                },
                all_scopes,
                headless,
            };
            crate::auth::login(&ctx.client, &config, opts).await?;
            Ok(())
        }
        AuthCommand::Logout => {
            crate::token_store::delete_tokens()?;
            println!("Logged out. Tokens deleted.");
            Ok(())
        }
        AuthCommand::Status => {
            if let Some(tokens) = crate::token_store::load_tokens()? {
                if tokens.is_expired() {
                    println!("Status: EXPIRED (expired at {})", tokens.expires_at);
                    println!("Run `nea-esi-cli auth login` to re-authenticate.");
                } else if tokens.needs_refresh() {
                    println!("Status: NEEDS REFRESH (expires at {})", tokens.expires_at);
                    println!("Token will auto-refresh on next authenticated request.");
                } else {
                    println!("Status: VALID (expires at {})", tokens.expires_at);
                }
            } else {
                println!("Status: NOT LOGGED IN");
                println!("Run `nea-esi-cli auth login` to authenticate.");
            }
            Ok(())
        }
    }
}
