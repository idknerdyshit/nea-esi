pub mod alliance;
pub mod assets;
pub mod auth_cmd;
pub mod calendar;
pub mod character;
pub mod clones;
pub mod config_cmd;
pub mod contacts;
pub mod contracts;
pub mod corporation;
pub mod dogma;
pub mod fittings;
pub mod fleet;
pub mod fw;
pub mod industry;
pub mod killmails;
pub mod loyalty;
pub mod mail;
pub mod market;
pub mod mining;
pub mod navigation;
pub mod pi;
pub mod resolve;
pub mod search;
pub mod skills;
pub mod sovereignty;
pub mod status;
pub mod universe;
pub mod wallet;
pub mod wars;

pub struct ExecContext {
    pub client: nea_esi::EsiClient,
    pub format: crate::output::OutputFormat,
    pub character_id: Option<i64>,
    pub corporation_id: Option<i64>,
    pub paths: crate::config::CliPaths,
}
