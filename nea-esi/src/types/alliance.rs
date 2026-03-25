use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiAllianceInfo {
    pub name: String,
    #[serde(default)]
    pub ticker: Option<String>,
}

/// Alliance icon URLs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsiAllianceIcons {
    #[serde(default)]
    pub px64: Option<String>,
    #[serde(default)]
    pub px128: Option<String>,
}
