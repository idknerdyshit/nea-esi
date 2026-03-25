#[derive(clap::Subcommand)]
pub enum UniverseCommand {
    /// Get type info
    Type {
        /// Type ID
        type_id: i32,
    },
    /// List all type IDs
    Types,
    /// Get group info
    Group {
        /// Group ID
        group_id: i32,
    },
    /// List all group IDs
    Groups,
    /// Get category info
    Category {
        /// Category ID
        category_id: i32,
    },
    /// List all category IDs
    Categories,
    /// Get solar system info
    System {
        /// System ID
        system_id: i32,
    },
    /// List all system IDs
    Systems,
    /// Get constellation info
    Constellation {
        /// Constellation ID
        constellation_id: i32,
    },
    /// List all constellation IDs
    Constellations,
    /// Get region info
    Region {
        /// Region ID
        region_id: i32,
    },
    /// List all region IDs
    Regions,
    /// Get station info
    Station {
        /// Station ID
        station_id: i32,
    },
    /// Get stargate info
    Stargate {
        /// Stargate ID
        stargate_id: i32,
    },
    /// Get structure info (requires auth)
    Structure {
        /// Structure ID
        structure_id: i64,
    },
    /// List all ancestries
    Ancestries,
    /// List all bloodlines
    Bloodlines,
    /// List all races
    Races,
    /// List all factions
    Factions,
    /// Get asteroid belt info
    AsteroidBelt {
        /// Asteroid belt ID
        id: i32,
    },
    /// Get moon info
    Moon {
        /// Moon ID
        id: i32,
    },
    /// Get planet info
    Planet {
        /// Planet ID
        id: i32,
    },
    /// Get star info
    Star {
        /// Star ID
        id: i32,
    },
    /// Get graphic info
    Graphic {
        /// Graphic ID
        id: i32,
    },
    /// Get schematic info
    Schematic {
        /// Schematic ID
        id: i32,
    },
    /// List all graphic IDs
    Graphics,
    /// List public structure IDs
    PublicStructures,
    /// Get system jump statistics
    SystemJumps,
    /// Get system kill statistics
    SystemKills,
}

#[allow(clippy::too_many_lines)]
pub async fn execute(ctx: &super::ExecContext, cmd: UniverseCommand) -> anyhow::Result<()> {
    match cmd {
        UniverseCommand::Type { type_id } => {
            let result = ctx.client.get_type(type_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Types => {
            let result = ctx.client.list_type_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Group { group_id } => {
            let result = ctx.client.get_group(group_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Groups => {
            let result = ctx.client.list_universe_group_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Category { category_id } => {
            let result = ctx.client.get_category(category_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Categories => {
            let result = ctx.client.list_universe_category_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::System { system_id } => {
            let result = ctx.client.get_system(system_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Systems => {
            let result = ctx.client.list_universe_system_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Constellation { constellation_id } => {
            let result = ctx.client.get_constellation(constellation_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Constellations => {
            let result = ctx.client.list_universe_constellation_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Region { region_id } => {
            let result = ctx.client.get_region(region_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Regions => {
            let result = ctx.client.list_universe_region_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Station { station_id } => {
            let result = ctx.client.get_station(station_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Stargate { stargate_id } => {
            let result = ctx.client.get_stargate(stargate_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Structure { structure_id } => {
            let result = ctx.client.get_structure(structure_id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Ancestries => {
            let result = ctx.client.universe_ancestries().await?;
            crate::output::print_list(&result, ctx.format)
        }
        UniverseCommand::Bloodlines => {
            let result = ctx.client.universe_bloodlines().await?;
            crate::output::print_list(&result, ctx.format)
        }
        UniverseCommand::Races => {
            let result = ctx.client.universe_races().await?;
            crate::output::print_list(&result, ctx.format)
        }
        UniverseCommand::Factions => {
            let result = ctx.client.universe_factions().await?;
            crate::output::print_list(&result, ctx.format)
        }
        UniverseCommand::AsteroidBelt { id } => {
            let result = ctx.client.universe_asteroid_belt(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Moon { id } => {
            let result = ctx.client.universe_moon(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Planet { id } => {
            let result = ctx.client.universe_planet(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Star { id } => {
            let result = ctx.client.universe_star(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Graphic { id } => {
            let result = ctx.client.universe_graphic(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Schematic { id } => {
            let result = ctx.client.universe_schematic(id).await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::Graphics => {
            let result = ctx.client.list_universe_graphic_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::PublicStructures => {
            let result = ctx.client.list_public_structure_ids().await?;
            crate::output::print_value(&result, ctx.format)
        }
        UniverseCommand::SystemJumps => {
            let result = ctx.client.system_jumps().await?;
            crate::output::print_list(&result, ctx.format)
        }
        UniverseCommand::SystemKills => {
            let result = ctx.client.system_kills().await?;
            crate::output::print_list(&result, ctx.format)
        }
    }
}
