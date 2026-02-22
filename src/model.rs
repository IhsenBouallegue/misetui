use serde::Deserialize;
use std::collections::BTreeMap;

/// Represents the source of a tool configuration.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ToolSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub path: String,
}

/// A single installed tool version from `mise ls -J`.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct InstalledToolVersion {
    pub version: String,
    pub requested_version: Option<String>,
    pub install_path: Option<String>,
    pub source: Option<ToolSource>,
    pub symlinked_to: Option<String>,
    #[serde(default)]
    pub installed: bool,
    #[serde(default)]
    pub active: bool,
}

/// Flattened tool for display in the tools table.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct InstalledTool {
    pub name: String,
    pub version: String,
    pub active: bool,
    pub installed: bool,
    pub source: String,
    pub requested_version: String,
}

impl InstalledTool {
    pub fn from_map(map: BTreeMap<String, Vec<InstalledToolVersion>>) -> Vec<Self> {
        let mut tools = Vec::new();
        for (name, versions) in map {
            for v in versions {
                let source = v
                    .source
                    .as_ref()
                    .map(|s| {
                        s.path
                            .rsplit('/')
                            .next()
                            .unwrap_or(&s.path)
                            .to_string()
                    })
                    .unwrap_or_default();
                tools.push(InstalledTool {
                    name: name.clone(),
                    version: v.version,
                    active: v.active,
                    installed: v.installed,
                    source,
                    requested_version: v.requested_version.unwrap_or_default(),
                });
            }
        }
        tools
    }
}

/// A registry entry from `mise registry -J`.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct RegistryEntry {
    pub short: String,
    #[serde(default)]
    pub backends: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// A config file from `mise config ls -J`.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigFile {
    pub path: String,
    #[serde(default)]
    pub tools: Vec<String>,
}
