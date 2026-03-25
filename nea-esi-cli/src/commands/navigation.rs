#[derive(clap::Subcommand)]
pub enum NavigationCommand {
    /// Calculate a route between two systems
    Route {
        /// Origin system ID
        origin: i32,
        /// Destination system ID
        destination: i32,
        /// Route preference: shortest, secure, insecure
        #[arg(long)]
        flag: Option<String>,
        /// System IDs to avoid
        #[arg(long, value_delimiter = ',')]
        avoid: Vec<i32>,
    },
    /// Set autopilot waypoint (requires auth)
    Waypoint {
        /// Destination ID
        destination_id: i64,
        /// Add to beginning of route
        #[arg(long)]
        add_to_beginning: bool,
        /// Clear other waypoints
        #[arg(long)]
        clear_other: bool,
    },
    /// Open contract window in-game (requires auth)
    OpenContract {
        /// Contract ID
        contract_id: i64,
    },
    /// Open info window in-game (requires auth)
    OpenInfo {
        /// Target ID
        target_id: i64,
    },
    /// Open market details in-game (requires auth)
    OpenMarket {
        /// Type ID
        type_id: i32,
    },
}

pub async fn execute(ctx: &super::ExecContext, cmd: NavigationCommand) -> anyhow::Result<()> {
    match cmd {
        NavigationCommand::Route {
            origin,
            destination,
            flag,
            avoid,
        } => {
            let result = ctx
                .client
                .get_route(origin, destination, flag.as_deref(), &avoid, None)
                .await?;
            crate::output::print_value(&result, ctx.format)
        }
        NavigationCommand::Waypoint {
            destination_id,
            add_to_beginning,
            clear_other,
        } => {
            ctx.client
                .ui_autopilot_waypoint(destination_id, add_to_beginning, clear_other)
                .await?;
            println!("Waypoint set.");
            Ok(())
        }
        NavigationCommand::OpenContract { contract_id } => {
            ctx.client.ui_open_contract_window(contract_id).await?;
            println!("Contract window opened.");
            Ok(())
        }
        NavigationCommand::OpenInfo { target_id } => {
            ctx.client.ui_open_info_window(target_id).await?;
            println!("Info window opened.");
            Ok(())
        }
        NavigationCommand::OpenMarket { type_id } => {
            ctx.client.ui_open_market_details(type_id).await?;
            println!("Market details opened.");
            Ok(())
        }
    }
}
