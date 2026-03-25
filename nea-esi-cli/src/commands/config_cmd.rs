use crate::config::Config;

#[derive(clap::Subcommand)]
pub enum ConfigCommand {
    /// Initialize configuration with defaults
    Init,
    /// Show current configuration
    Show,
    /// Set a configuration value (dot-separated key, e.g. `defaults.character_id`)
    Set {
        /// Key path (e.g. `defaults.character_id`, `app.client_id`)
        key: String,
        /// Value to set
        value: String,
    },
}

pub fn execute(ctx: &super::ExecContext, cmd: ConfigCommand) -> anyhow::Result<()> {
    match cmd {
        ConfigCommand::Init => {
            let config = Config::default();
            config.save(ctx.config_path.as_ref())?;
            let path = ctx
                .config_path
                .clone()
                .or_else(Config::config_path)
                .unwrap_or_default();
            println!("Config initialized at {}", path.display());
            Ok(())
        }
        ConfigCommand::Show => {
            let mut config = Config::load(ctx.config_path.as_ref())?;
            // Mask client_secret to avoid exposing it in terminal output.
            if let Some(ref secret) = config.app.client_secret {
                let masked = if secret.len() > 4 {
                    format!("****{}", &secret[secret.len() - 4..])
                } else {
                    "****".to_string()
                };
                config.app.client_secret = Some(masked);
            }
            let toml_str = toml::to_string_pretty(&config)?;
            println!("{toml_str}");
            Ok(())
        }
        ConfigCommand::Set { key, value } => {
            let mut config = Config::load(ctx.config_path.as_ref())?;
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
            config.save(ctx.config_path.as_ref())?;
            println!("Set {key} successfully.");
            Ok(())
        }
    }
}
