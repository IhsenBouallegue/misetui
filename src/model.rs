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

/// An outdated tool from `mise outdated -J`.
#[derive(Debug, Clone)]
pub struct OutdatedTool {
    pub name: String,
    pub current: String,
    pub requested: String,
    pub latest: String,
}

/// Deserializable entry for `mise outdated -J` (per-tool object).
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct OutdatedEntry {
    #[serde(default)]
    pub current: Option<String>,
    #[serde(default)]
    pub requested: Option<String>,
    #[serde(default)]
    pub latest: Option<String>,
}

impl OutdatedTool {
    pub fn from_map(map: BTreeMap<String, OutdatedEntry>) -> Vec<Self> {
        map.into_iter()
            .map(|(name, entry)| OutdatedTool {
                name,
                current: entry.current.unwrap_or_default(),
                requested: entry.requested.unwrap_or_default(),
                latest: entry.latest.unwrap_or_default(),
            })
            .collect()
    }
}

/// A mise task from `mise tasks ls -J`.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct MiseTask {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// An environment variable from `mise env --json-extended`.
#[derive(Debug, Clone)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    pub source: String,
    pub tool: String,
}

/// Deserializable entry for `mise env --json-extended` (per-variable object).
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct EnvVarEntry {
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
}

impl EnvVar {
    pub fn from_map(map: BTreeMap<String, EnvVarEntry>) -> Vec<Self> {
        map.into_iter()
            .map(|(name, entry)| EnvVar {
                name,
                value: entry.value.unwrap_or_default(),
                source: entry.source.unwrap_or_default(),
                tool: entry.tool.unwrap_or_default(),
            })
            .collect()
    }
}

/// A mise setting from `mise settings ls -J --all`.
#[derive(Debug, Clone)]
pub struct MiseSetting {
    pub key: String,
    pub value: String,
    pub value_type: String,
}

impl MiseSetting {
    pub fn from_json(value: serde_json::Value) -> Vec<Self> {
        let Some(obj) = value.as_object() else {
            return Vec::new();
        };
        obj.iter()
            .map(|(key, val)| {
                let (value_str, type_str) = match val {
                    serde_json::Value::String(s) => (s.clone(), "string".to_string()),
                    serde_json::Value::Bool(b) => (b.to_string(), "bool".to_string()),
                    serde_json::Value::Number(n) => (n.to_string(), "number".to_string()),
                    serde_json::Value::Null => ("null".to_string(), "null".to_string()),
                    serde_json::Value::Array(arr) => {
                        let items: Vec<String> =
                            arr.iter().map(|v| v.to_string()).collect();
                        (items.join(", "), "array".to_string())
                    }
                    serde_json::Value::Object(_) => (val.to_string(), "object".to_string()),
                };
                MiseSetting {
                    key: key.clone(),
                    value: value_str,
                    value_type: type_str,
                }
            })
            .collect()
    }
}

/// A prune candidate from `mise prune --dry-run`.
#[derive(Debug, Clone)]
pub struct PruneCandidate {
    pub tool: String,
    pub version: String,
}

/// Health state of the current working directory's tool requirements.
/// Checking = async check in flight; Healthy = all tools present and correct version;
/// Drifted = at least one tool version mismatch; Missing = at least one tool not installed;
/// NoConfig = no .mise.toml or global config applies to CWD.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftState {
    Checking,
    Healthy,
    Drifted,
    Missing,
    NoConfig,
}
