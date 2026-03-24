use crate::config::Config;

#[derive(clap::Subcommand)]
pub enum ConfigCommand {
    /// Initialize configuration with defaults
    Init,
    /// Show current configuration
    Show,
    /// Set a configuration value (dot-separated key, e.g. defaults.character_id)
    Set {
        /// Key path (e.g. defaults.character_id, app.client_id)
        key: String,
        /// Value to set
        value: String,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: ConfigCommand) -> anyhow::Result<()> {
    let _ = ctx;
    match cmd {
        ConfigCommand::Init => {
            let config = Config::default();
            config.save(None)?;
            let path = Config::config_path().unwrap_or_default();
            println!("Config initialized at {}", path.display());
            Ok(())
        }
        ConfigCommand::Show => {
            let config = Config::load(None)?;
            let toml_str = toml::to_string_pretty(&config)?;
            println!("{toml_str}");
            Ok(())
        }
        ConfigCommand::Set { key, value } => {
            let mut config = Config::load(None)?;
            match key.as_str() {
                "app.client_id" => config.app.client_id = Some(value),
                "app.client_secret" => config.app.client_secret = Some(value),
                "app.user_agent" => config.app.user_agent = Some(value),
                "defaults.character_id" => {
                    config.defaults.character_id = Some(value.parse()?);
                }
                "defaults.corporation_id" => {
                    config.defaults.corporation_id = Some(value.parse()?);
                }
                "defaults.format" => config.defaults.format = Some(value),
                "defaults.region_id" => {
                    config.defaults.region_id = Some(value.parse()?);
                }
                "auth.headless" => {
                    config.auth.headless = value.parse()?;
                }
                other => {
                    return Err(anyhow::anyhow!("Unknown config key: {other}"));
                }
            }
            config.save(None)?;
            println!("Set {key} successfully.");
            Ok(())
        }
    }
}
