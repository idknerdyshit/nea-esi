use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::commands;

#[derive(Parser)]
#[command(name = "nea-esi-cli", about = "CLI for EVE Online's ESI API")]
pub struct Cli {
    /// Output format: json, table, csv (default: table for TTY, json otherwise)
    #[arg(long, global = true)]
    pub format: Option<String>,

    /// Config file path
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Override default character ID
    #[arg(long, global = true)]
    pub character_id: Option<i64>,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Authentication management
    Auth {
        #[command(subcommand)]
        command: commands::auth_cmd::AuthCommand,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: commands::config_cmd::ConfigCommand,
    },
    /// Launch interactive REPL
    Interactive,
    /// Server status
    Status,
    /// Market data endpoints
    Market {
        #[command(subcommand)]
        command: commands::market::MarketCommand,
    },
    /// Character endpoints
    Character {
        #[command(subcommand)]
        command: commands::character::CharacterCommand,
    },
    /// Corporation endpoints
    Corporation {
        #[command(subcommand)]
        command: commands::corporation::CorporationCommand,
    },
    /// Alliance endpoints
    Alliance {
        #[command(subcommand)]
        command: commands::alliance::AllianceCommand,
    },
    /// Universe data endpoints
    Universe {
        #[command(subcommand)]
        command: commands::universe::UniverseCommand,
    },
    /// Wallet endpoints
    Wallet {
        #[command(subcommand)]
        command: commands::wallet::WalletCommand,
    },
    /// Skill endpoints
    Skills {
        #[command(subcommand)]
        command: commands::skills::SkillsCommand,
    },
    /// Asset endpoints
    Assets {
        #[command(subcommand)]
        command: commands::assets::AssetsCommand,
    },
    /// Mail endpoints
    Mail {
        #[command(subcommand)]
        command: commands::mail::MailCommand,
    },
    /// Fleet management endpoints
    Fleet {
        #[command(subcommand)]
        command: commands::fleet::FleetCommand,
    },
    /// Industry endpoints
    Industry {
        #[command(subcommand)]
        command: commands::industry::IndustryCommand,
    },
    /// Contract endpoints
    Contracts {
        #[command(subcommand)]
        command: commands::contracts::ContractsCommand,
    },
    /// Killmail endpoints
    Killmails {
        #[command(subcommand)]
        command: commands::killmails::KillmailsCommand,
    },
    /// Search for entities
    Search {
        #[command(subcommand)]
        command: commands::search::SearchCommand,
    },
    /// Sovereignty endpoints
    Sovereignty {
        #[command(subcommand)]
        command: commands::sovereignty::SovereigntyCommand,
    },
    /// War endpoints
    Wars {
        #[command(subcommand)]
        command: commands::wars::WarsCommand,
    },
    /// Faction warfare endpoints
    Fw {
        #[command(subcommand)]
        command: commands::fw::FwCommand,
    },
    /// Dogma endpoints
    Dogma {
        #[command(subcommand)]
        command: commands::dogma::DogmaCommand,
    },
    /// Route and UI commands
    Navigation {
        #[command(subcommand)]
        command: commands::navigation::NavigationCommand,
    },
    /// Contact endpoints
    Contacts {
        #[command(subcommand)]
        command: commands::contacts::ContactsCommand,
    },
    /// Fitting endpoints
    Fittings {
        #[command(subcommand)]
        command: commands::fittings::FittingsCommand,
    },
    /// Calendar endpoints
    Calendar {
        #[command(subcommand)]
        command: commands::calendar::CalendarCommand,
    },
    /// Clone and implant endpoints
    Clones {
        #[command(subcommand)]
        command: commands::clones::ClonesCommand,
    },
    /// Loyalty point endpoints
    Loyalty {
        #[command(subcommand)]
        command: commands::loyalty::LoyaltyCommand,
    },
    /// Planetary interaction endpoints
    Pi {
        #[command(subcommand)]
        command: commands::pi::PiCommand,
    },
    /// Mining endpoints
    Mining {
        #[command(subcommand)]
        command: commands::mining::MiningCommand,
    },
    /// Resolve names/IDs
    Resolve {
        #[command(subcommand)]
        command: commands::resolve::ResolveCommand,
    },
}
