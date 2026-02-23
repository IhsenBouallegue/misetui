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
/// Checking = async check in flight; Healthy = all local-config tools installed;
/// Missing = one or more not installed (covers exact-version pins too);
/// NoConfig = no local .mise.toml applies to CWD (global config only);
/// Untrusted = .mise.toml exists but has not been trusted with `mise trust`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriftState {
    Checking,
    Healthy,
    Missing,
    NoConfig,
    Untrusted,
}

/// Health status for a project or individual tool requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectHealthStatus {
    /// All required tools are installed at the correct version.
    Healthy,
    /// At least one required tool is installed but at an older version than required.
    Outdated,
    /// At least one required tool is not installed at all.
    Missing,
    /// No .mise.toml found for this project path.
    NoConfig,
}

impl ProjectHealthStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ProjectHealthStatus::Healthy  => "● healthy",
            ProjectHealthStatus::Outdated => "◐ outdated",
            ProjectHealthStatus::Missing  => "○ missing",
            ProjectHealthStatus::NoConfig => "  no config",
        }
    }
}

/// Per-tool health row inside a project drill-down.
#[derive(Debug, Clone)]
pub struct ProjectToolHealth {
    /// Tool name (e.g. "node", "python").
    pub tool: String,
    /// Version string as specified in .mise.toml (e.g. "20", "20.1.0", "latest").
    pub required: String,
    /// Installed version string, or empty if not installed.
    pub installed: String,
    pub status: ProjectHealthStatus,
}

/// A project discovered by scanning configured directories.
#[derive(Debug, Clone)]
pub struct MiseProject {
    /// Directory name (last path component).
    pub name: String,
    /// Absolute path to the project directory (parent of .mise.toml).
    pub path: String,
    /// Number of tools declared in .mise.toml.
    pub tool_count: usize,
    /// Aggregate health status (worst-case of all tool health statuses).
    pub health: ProjectHealthStatus,
    /// Per-tool health breakdown (populated during scan).
    pub tools: Vec<ProjectToolHealth>,
}

/// A tool detected from filesystem indicators or migrated from legacy pin files.
#[derive(Debug, Clone)]
pub struct DetectedTool {
    /// Short tool name as used by mise (e.g. "node", "python", "rust", "go", "ruby", "php").
    pub name: String,
    /// Version string if migrated from a legacy pin file (e.g. "20.11.0"), or empty if auto-detected without a pin.
    pub version: String,
    /// Source description for display (e.g. "package.json", ".nvmrc", "Cargo.toml").
    pub source: String,
    /// Whether this tool is toggled ON (will be written to .mise.toml). Defaults to true.
    pub enabled: bool,
    /// True if this tool version is already installed locally (from `mise ls -J` cross-reference).
    pub installed: bool,
}

/// Multi-step wizard state for BOOT-01 through BOOT-07.
#[derive(Debug, Clone)]
pub struct WizardState {
    /// Target directory for the .mise.toml to be written.
    pub target_dir: String,
    /// Current step in the wizard flow.
    pub step: WizardStep,
    /// Tools detected/migrated — each with enabled flag for toggling in Review step.
    pub tools: Vec<DetectedTool>,
    /// Which tool row is currently highlighted in Review step.
    pub selected: usize,
    /// Generated .mise.toml content (populated when entering Preview step).
    pub preview_content: String,
    /// If true, write AGENTS.md and CLAUDE.md alongside .mise.toml (BOOT-07).
    pub write_agent_files: bool,
    /// Scroll offset for Preview step paragraph.
    pub preview_scroll: usize,
}

/// Steps in the Bootstrap Wizard flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    /// Tab is open but wizard not started yet.
    Idle,
    /// Scanning filesystem — spinner shown.
    Detecting,
    /// User reviews and toggles tool list.
    Review,
    /// User previews the generated .mise.toml content.
    Preview,
    /// Writing file and running mise install — spinner shown.
    Writing,
}

/// Which sub-tab is active inside the inline editor popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorTab {
    Tools,
    Env,
    Tasks,
}

/// Change status for a row in the editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorRowStatus {
    /// Unchanged from the file on disk.
    Unchanged,
    /// Modified by the user.
    Modified,
    /// Newly added by the user.
    Added,
    /// Marked for deletion.
    Deleted,
}

/// A single editable row in the editor's Tools sub-tab.
#[derive(Debug, Clone)]
pub struct EditorToolRow {
    pub name: String,
    pub version: String,
    pub status: EditorRowStatus,
    /// Original name (for rename tracking in toml_edit Document).
    pub original_name: Option<String>,
}

/// A single editable row in the editor's Env sub-tab.
#[derive(Debug, Clone)]
pub struct EditorEnvRow {
    pub key: String,
    pub value: String,
    pub status: EditorRowStatus,
    pub original_key: Option<String>,
}

/// A single editable row in the editor's Tasks sub-tab.
#[derive(Debug, Clone)]
pub struct EditorTaskRow {
    pub name: String,
    pub command: String,
    pub status: EditorRowStatus,
    pub original_name: Option<String>,
}

/// Full state for the inline editor popup.
#[derive(Debug, Clone)]
pub struct EditorState {
    /// Absolute path to the .mise.toml being edited.
    pub file_path: String,
    /// Active sub-tab (Tools / Env / Tasks).
    pub tab: EditorTab,
    /// Tool rows parsed from [tools] table.
    pub tools: Vec<EditorToolRow>,
    /// Env rows parsed from [env] table.
    pub env_vars: Vec<EditorEnvRow>,
    /// Task rows parsed from [tasks] table.
    pub tasks: Vec<EditorTaskRow>,
    /// Currently selected row index (within the active sub-tab).
    pub selected: usize,
    /// True when the user is in inline edit mode (typing into a cell).
    pub editing: bool,
    /// Which column is being edited (0 = name/key, 1 = version/value/command).
    pub edit_column: usize,
    /// Text buffer for the cell being edited.
    pub edit_buffer: String,
    /// The raw toml_edit Document for round-trip writes.
    /// Stored as String (serialized Document) to keep model.rs free of toml_edit dependency.
    /// Re-parsed in write_editor_changes().
    pub raw_document: String,
    /// True if any row has been modified/added/deleted since open.
    pub dirty: bool,
}
