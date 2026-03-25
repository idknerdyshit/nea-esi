use serde::{Deserialize, Serialize};

/// Resolved name from POST /universe/names/.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiResolvedName {
    pub id: i64,
    pub name: String,
    pub category: String,
}

/// Result of POST /universe/ids/ — names resolved to IDs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EsiResolvedIds {
    #[serde(default)]
    pub characters: Vec<EsiIdEntry>,
    #[serde(default)]
    pub corporations: Vec<EsiIdEntry>,
    #[serde(default)]
    pub alliances: Vec<EsiIdEntry>,
    #[serde(default)]
    pub systems: Vec<EsiIdEntry>,
    #[serde(default)]
    pub constellations: Vec<EsiIdEntry>,
    #[serde(default)]
    pub regions: Vec<EsiIdEntry>,
    #[serde(default)]
    pub stations: Vec<EsiIdEntry>,
    #[serde(default)]
    pub inventory_types: Vec<EsiIdEntry>,
    #[serde(default)]
    pub factions: Vec<EsiIdEntry>,
    #[serde(default)]
    pub agents: Vec<EsiIdEntry>,
}

impl EsiResolvedIds {
    /// Merge another `EsiResolvedIds` into this one.
    pub fn merge(&mut self, other: EsiResolvedIds) {
        self.characters.extend(other.characters);
        self.corporations.extend(other.corporations);
        self.alliances.extend(other.alliances);
        self.systems.extend(other.systems);
        self.constellations.extend(other.constellations);
        self.regions.extend(other.regions);
        self.stations.extend(other.stations);
        self.inventory_types.extend(other.inventory_types);
        self.factions.extend(other.factions);
        self.agents.extend(other.agents);
    }
}

/// A single resolved ID + name entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiIdEntry {
    pub id: i64,
    pub name: String,
}

/// Result of GET /search/.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EsiSearchResult {
    #[serde(default)]
    pub character: Vec<i64>,
    #[serde(default)]
    pub corporation: Vec<i64>,
    #[serde(default)]
    pub alliance: Vec<i64>,
    #[serde(default)]
    pub solar_system: Vec<i32>,
    #[serde(default)]
    pub constellation: Vec<i32>,
    #[serde(default)]
    pub region: Vec<i32>,
    #[serde(default)]
    pub station: Vec<i32>,
    #[serde(default)]
    pub inventory_type: Vec<i32>,
    #[serde(default)]
    pub agent: Vec<i64>,
    #[serde(default)]
    pub faction: Vec<i32>,
}
