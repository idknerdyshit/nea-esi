mod auth;
mod cli;
mod commands;
mod config;
mod error;
mod output;
mod repl;
mod token_store;

use clap::Parser;
use nea_esi::EsiClient;
use secrecy::SecretString;

use cli::{Cli, Command};
use commands::ExecContext;
use output::OutputFormat;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::DEBUG.into()),
            )
            .with_target(false)
            .init();
    }

    let config = config::Config::load(cli.config.as_ref())?;
    let format = OutputFormat::from_str_or_auto(cli.format.as_deref());

    // Build the ESI client
    let client = build_client(&config)?;

    // Load tokens if available
    if let Some(tokens) = token_store::load_tokens()? {
        client.set_tokens(tokens).await;
    }

    let character_id = cli.character_id.or(config.defaults.character_id);
    let corporation_id = config.defaults.corporation_id;

    let ctx = ExecContext {
        client,
        format,
        character_id,
        corporation_id,
        config_path: cli.config.clone(),
    };

    if matches!(cli.command, Command::Interactive) {
        repl::run(ctx, config).await?;
    } else {
        dispatch(&ctx, cli.command).await?;

        // Save tokens back if they were refreshed
        if let Some(tokens) = ctx.client.get_tokens().await {
            token_store::save_tokens(&tokens)?;
        }
    }

    Ok(())
}

pub async fn dispatch(ctx: &ExecContext, command: Command) -> anyhow::Result<()> {
    match command {
        Command::Auth { command } => commands::auth_cmd::execute(ctx, command).await?,
        Command::Config { command } => commands::config_cmd::execute(ctx, command).await?,
        Command::Interactive => unreachable!(),
        Command::Status => commands::status::execute(ctx).await?,
        Command::Market { command } => commands::market::execute(ctx, command).await?,
        Command::Character { command } => commands::character::execute(ctx, command).await?,
        Command::Corporation { command } => commands::corporation::execute(ctx, command).await?,
        Command::Alliance { command } => commands::alliance::execute(ctx, command).await?,
        Command::Universe { command } => commands::universe::execute(ctx, command).await?,
        Command::Wallet { command } => commands::wallet::execute(ctx, command).await?,
        Command::Skills { command } => commands::skills::execute(ctx, command).await?,
        Command::Assets { command } => commands::assets::execute(ctx, command).await?,
        Command::Mail { command } => commands::mail::execute(ctx, command).await?,
        Command::Fleet { command } => commands::fleet::execute(ctx, command).await?,
        Command::Industry { command } => commands::industry::execute(ctx, command).await?,
        Command::Contracts { command } => commands::contracts::execute(ctx, command).await?,
        Command::Killmails { command } => commands::killmails::execute(ctx, command).await?,
        Command::Search { command } => commands::search::execute(ctx, command).await?,
        Command::Sovereignty { command } => commands::sovereignty::execute(ctx, command).await?,
        Command::Wars { command } => commands::wars::execute(ctx, command).await?,
        Command::Fw { command } => commands::fw::execute(ctx, command).await?,
        Command::Dogma { command } => commands::dogma::execute(ctx, command).await?,
        Command::Navigation { command } => commands::navigation::execute(ctx, command).await?,
        Command::Contacts { command } => commands::contacts::execute(ctx, command).await?,
        Command::Fittings { command } => commands::fittings::execute(ctx, command).await?,
        Command::Calendar { command } => commands::calendar::execute(ctx, command).await?,
        Command::Clones { command } => commands::clones::execute(ctx, command).await?,
        Command::Loyalty { command } => commands::loyalty::execute(ctx, command).await?,
        Command::Pi { command } => commands::pi::execute(ctx, command).await?,
        Command::Mining { command } => commands::mining::execute(ctx, command).await?,
        Command::Resolve { command } => commands::resolve::execute(ctx, command).await?,
    }
    Ok(())
}

fn build_client(config: &config::Config) -> anyhow::Result<EsiClient> {
    let ua = config
        .app
        .user_agent
        .as_deref()
        .unwrap_or("nea-esi-cli");

    let client = match (&config.app.client_id, &config.app.client_secret) {
        (Some(id), Some(secret)) => {
            EsiClient::with_web_app(ua, id, SecretString::from(secret.clone()))?
        }
        (Some(id), None) => EsiClient::with_native_app(ua, id)?,
        _ => EsiClient::with_user_agent(ua)?,
    };

    Ok(client.with_cache())
}
