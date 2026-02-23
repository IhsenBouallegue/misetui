use crate::config::MisetuiConfig;
use crate::model::{
    ConfigFile, DriftState, EnvVar, EnvVarEntry, InstalledTool, InstalledToolVersion, MiseProject,
    MiseSetting, MiseTask, OutdatedEntry, OutdatedTool, ProjectHealthStatus, ProjectToolHealth,
    PruneCandidate, RegistryEntry,
};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::process::Command;

async fn run_mise(args: &[&str]) -> Result<String, String> {
    let output = Command::new("mise")
        .args(args)
        .output()
        .await
        .map_err(|e| format!("Failed to run mise: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("mise {} failed: {stderr}", args.join(" ")))
    }
}

pub async fn fetch_tools() -> Result<Vec<InstalledTool>, String> {
    let json = run_mise(&["ls", "-J"]).await?;
    let map: BTreeMap<String, Vec<InstalledToolVersion>> =
        serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))?;
    Ok(InstalledTool::from_map(map))
}

pub async fn fetch_registry() -> Result<Vec<RegistryEntry>, String> {
    let json = run_mise(&["registry", "-J"]).await?;
    let entries: Vec<RegistryEntry> =
        serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))?;
    Ok(entries)
}

pub async fn fetch_config() -> Result<Vec<ConfigFile>, String> {
    let json = run_mise(&["config", "ls", "-J"]).await?;
    let configs: Vec<ConfigFile> =
        serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))?;
    Ok(configs)
}

pub async fn fetch_doctor() -> Result<Vec<String>, String> {
    let output = Command::new("mise")
        .args(["doctor"])
        .output()
        .await
        .map_err(|e| format!("Failed to run mise doctor: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let lines: Vec<String> = stdout.lines().map(|l| l.to_string()).collect();
    Ok(lines)
}

pub async fn fetch_versions(tool: &str) -> Result<Vec<String>, String> {
    let json = run_mise(&["ls-remote", tool]).await?;
    let versions: Vec<String> = json.lines().rev().map(|l| l.trim().to_string()).collect();
    // Return the most recent versions first, limit to 50
    Ok(versions.into_iter().take(50).collect())
}

pub async fn install_tool(tool: &str, version: &str) -> Result<String, String> {
    let tool_ver = format!("{tool}@{version}");
    run_mise(&["install", &tool_ver]).await?;
    Ok(format!("Installed {tool_ver}"))
}

pub async fn uninstall_tool(tool: &str, version: &str) -> Result<String, String> {
    let tool_ver = format!("{tool}@{version}");
    run_mise(&["uninstall", &tool_ver]).await?;
    Ok(format!("Uninstalled {tool_ver}"))
}

pub async fn update_tool(tool: &str) -> Result<String, String> {
    run_mise(&["upgrade", tool]).await?;
    Ok(format!("Updated {tool}"))
}

pub async fn fetch_outdated() -> Result<Vec<OutdatedTool>, String> {
    let json = run_mise(&["outdated", "-J"]).await?;
    let map: BTreeMap<String, OutdatedEntry> =
        serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))?;
    Ok(OutdatedTool::from_map(map))
}

pub async fn fetch_tasks() -> Result<Vec<MiseTask>, String> {
    let json = run_mise(&["tasks", "ls", "-J"]).await?;
    let tasks: Vec<MiseTask> =
        serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))?;
    Ok(tasks)
}

pub async fn fetch_env() -> Result<Vec<EnvVar>, String> {
    let json = run_mise(&["env", "--json-extended"]).await?;
    let map: BTreeMap<String, EnvVarEntry> =
        serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))?;
    Ok(EnvVar::from_map(map))
}

pub async fn fetch_settings() -> Result<Vec<MiseSetting>, String> {
    let json = run_mise(&["settings", "ls", "-J", "--all"]).await?;
    let value: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| format!("Parse error: {e}"))?;
    Ok(MiseSetting::from_json(value))
}

pub async fn upgrade_tool(tool: &str) -> Result<String, String> {
    run_mise(&["upgrade", tool]).await?;
    Ok(format!("Upgraded {tool}"))
}

pub async fn upgrade_all() -> Result<String, String> {
    run_mise(&["upgrade"]).await?;
    Ok("Upgraded all tools".to_string())
}

pub async fn run_task(task: &str) -> Result<String, String> {
    run_mise(&["run", task]).await?;
    Ok(format!("Task '{task}' completed"))
}

pub async fn use_tool(tool: &str, version: &str) -> Result<String, String> {
    let tool_ver = format!("{tool}@{version}");
    run_mise(&["use", "--global", &tool_ver]).await?;
    Ok(format!("Now using {tool_ver}"))
}

pub async fn prune_dry_run() -> Result<Vec<PruneCandidate>, String> {
    let output = Command::new("mise")
        .args(["prune", "--dry-run"])
        .output()
        .await
        .map_err(|e| format!("Failed to run mise prune: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let candidates: Vec<PruneCandidate> = stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            // Parse lines like "node@18.0.0" or "node 18.0.0"
            if let Some((tool, version)) = line.split_once('@') {
                Some(PruneCandidate {
                    tool: tool.trim().to_string(),
                    version: version.trim().to_string(),
                })
            } else if let Some((tool, version)) = line.split_once(' ') {
                Some(PruneCandidate {
                    tool: tool.trim().to_string(),
                    version: version.trim().to_string(),
                })
            } else {
                Some(PruneCandidate {
                    tool: line.to_string(),
                    version: String::new(),
                })
            }
        })
        .collect();
    Ok(candidates)
}

pub async fn prune() -> Result<String, String> {
    run_mise(&["prune", "-y"]).await?;
    Ok("Pruned unused tool versions".to_string())
}

pub async fn trust_config(path: &str) -> Result<String, String> {
    run_mise(&["trust", path]).await?;
    Ok(format!("Trusted {path}"))
}

#[allow(dead_code)]
pub async fn untrust_config(path: &str) -> Result<String, String> {
    run_mise(&["trust", "--untrust", path]).await?;
    Ok(format!("Untrusted {path}"))
}

pub async fn fetch_tool_info(tool: &str) -> Result<String, String> {
    // Returns raw JSON string for display in popup
    run_mise(&["tool", tool, "-J"]).await
}

/// Scan configured directories for .mise.toml files and compute project health.
/// Cross-references against `installed_tools` (already loaded in-memory) to avoid
/// extra mise subprocess calls.
pub fn scan_projects(
    config: &MisetuiConfig,
    installed_tools: &[crate::model::InstalledTool],
) -> Vec<MiseProject> {
    // Build a fast lookup: tool name → all installed versions (regardless of active state).
    // A tool can be installed but not active when the shell is not inside that project's
    // directory — we still want to report it as installed for the Projects health check.
    let mut installed_map: std::collections::HashMap<&str, Vec<&str>> =
        std::collections::HashMap::new();
    for tool in installed_tools {
        installed_map
            .entry(tool.name.as_str())
            .or_default()
            .push(tool.version.as_str());
    }

    let mut projects = Vec::new();

    for scan_root in &config.scan_dirs {
        collect_projects(scan_root, 0, config.max_depth, &installed_map, &mut projects);
    }

    // Deduplicate by path (a dir might appear in multiple scan roots)
    projects.sort_by(|a, b| a.path.cmp(&b.path));
    projects.dedup_by(|a, b| a.path == b.path);
    projects.sort_by(|a, b| a.name.cmp(&b.name));
    projects
}

fn collect_projects(
    dir: &std::path::Path,
    depth: usize,
    max_depth: usize,
    installed_map: &std::collections::HashMap<&str, Vec<&str>>,
    projects: &mut Vec<MiseProject>,
) {
    let config_path = dir.join(".mise.toml");
    if config_path.exists() {
        let project = parse_project(dir, &config_path, installed_map);
        projects.push(project);
        // Don't recurse into projects that already have a .mise.toml
        return;
    }

    if depth >= max_depth {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip hidden directories and common non-project dirs
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') || name_str == "node_modules" || name_str == "target" {
                continue;
            }
            collect_projects(&path, depth + 1, max_depth, installed_map, projects);
        }
    }
}

fn parse_project(
    dir: &std::path::Path,
    config_path: &std::path::Path,
    installed_map: &std::collections::HashMap<&str, Vec<&str>>,
) -> MiseProject {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.to_string_lossy().to_string());
    let path = dir.to_string_lossy().to_string();

    let Ok(contents) = std::fs::read_to_string(config_path) else {
        return MiseProject {
            name,
            path,
            tool_count: 0,
            health: ProjectHealthStatus::NoConfig,
            tools: Vec::new(),
        };
    };

    // Parse [tools] table from .mise.toml using basic TOML parsing
    let toml_val: toml::Value = match toml::from_str(&contents) {
        Ok(v) => v,
        Err(_) => {
            return MiseProject {
                name,
                path,
                tool_count: 0,
                health: ProjectHealthStatus::NoConfig,
                tools: Vec::new(),
            };
        }
    };

    let tool_entries: Vec<(String, String)> = toml_val
        .get("tools")
        .and_then(|t| t.as_table())
        .map(|table| {
            table
                .iter()
                .map(|(k, v)| {
                    let version = match v {
                        toml::Value::String(s) => s.clone(),
                        toml::Value::Array(arr) => arr
                            .first()
                            .and_then(|x| x.as_str())
                            .unwrap_or("?")
                            .to_string(),
                        other => other.to_string(),
                    };
                    (k.clone(), version)
                })
                .collect()
        })
        .unwrap_or_default();

    let mut tool_healths: Vec<ProjectToolHealth> = Vec::new();
    let mut worst = ProjectHealthStatus::Healthy;

    for (tool_name, required) in &tool_entries {
        let status = match installed_map.get(tool_name.as_str()) {
            None => ProjectHealthStatus::Missing,
            Some(versions) => {
                // Check if any installed version satisfies the requirement.
                // "latest" always satisfies. Exact match satisfies. Fuzzy prefix satisfies
                // (e.g. required "3.12" is satisfied by installed "3.12.12").
                let satisfied = required == "latest"
                    || versions.iter().any(|v| {
                        *v == required.as_str()
                            || v.starts_with(&format!("{}.", required))
                    });
                if satisfied {
                    ProjectHealthStatus::Healthy
                } else {
                    ProjectHealthStatus::Outdated
                }
            }
        };

        // Update worst-case aggregate
        match (&worst, &status) {
            (_, ProjectHealthStatus::Missing) => worst = ProjectHealthStatus::Missing,
            (ProjectHealthStatus::Healthy, ProjectHealthStatus::Outdated) => {
                worst = ProjectHealthStatus::Outdated
            }
            _ => {}
        }

        // Display the best-matching installed version (exact match first, else first available).
        let installed = installed_map
            .get(tool_name.as_str())
            .and_then(|versions| {
                versions
                    .iter()
                    .find(|v| {
                        **v == required.as_str()
                            || v.starts_with(&format!("{}.", required))
                    })
                    .or_else(|| versions.first())
                    .copied()
            })
            .unwrap_or("")
            .to_string();

        tool_healths.push(ProjectToolHealth {
            tool: tool_name.clone(),
            required: required.clone(),
            installed,
            status,
        });
    }

    MiseProject {
        name,
        path,
        tool_count: tool_entries.len(),
        health: worst,
        tools: tool_healths,
    }
}

/// Check the health of the current working directory's mise tool requirements.
///
/// Uses `mise config ls --json` + `mise ls --current --json` (proper tool-status APIs).
///
/// - `DriftState::NoConfig`  — no local .mise.toml applies; only global config (or nothing)
/// - `DriftState::Missing`   — one or more local-config tools are not installed
/// - `DriftState::Untrusted` — a local .mise.toml exists but hasn't been trusted
/// - `DriftState::Healthy`   — all local-config tools are installed
pub async fn check_cwd_drift() -> Result<DriftState, String> {
    let cwd = std::env::current_dir()
        .map_err(|e| format!("Cannot determine CWD: {e}"))?;

    // Step 1: Detect whether a local (non-global) config applies to this directory.
    // `mise config ls --json` returns an array of config entries ordered by precedence.
    let config_out = Command::new("mise")
        .args(["config", "ls", "--json"])
        .current_dir(&cwd)
        .output()
        .await
        .map_err(|e| format!("Failed to run mise config ls: {e}"))?;

    // Untrusted config — exit 1, empty stdout, error in stderr.
    if !config_out.status.success() {
        let stderr = String::from_utf8_lossy(&config_out.stderr);
        if stderr.contains("not trusted") {
            return Ok(DriftState::Untrusted);
        }
        return Ok(DriftState::NoConfig);
    }

    let configs: Vec<serde_json::Value> =
        serde_json::from_slice(&config_out.stdout).unwrap_or_default();

    // Global config lives at $XDG_CONFIG_HOME/mise/config.toml.
    let global_config: Option<PathBuf> = dirs::config_dir().map(|d| d.join("mise/config.toml"));

    let local_config_paths: Vec<PathBuf> = configs
        .iter()
        .filter_map(|e| e["path"].as_str().map(PathBuf::from))
        .filter(|p| {
            global_config
                .as_ref()
                .map(|g| p != g)
                .unwrap_or(true)
        })
        .collect();

    if local_config_paths.is_empty() {
        return Ok(DriftState::NoConfig);
    }

    // Step 2: Check tool installation state for local-config tools only.
    // `mise ls --current --json` returns { "tool": [ { installed, source, ... } ] }
    let ls_out = Command::new("mise")
        .args(["ls", "--current", "--json"])
        .current_dir(&cwd)
        .output()
        .await
        .map_err(|e| format!("Failed to run mise ls: {e}"))?;

    if !ls_out.status.success() {
        let stderr = String::from_utf8_lossy(&ls_out.stderr);
        if stderr.contains("not trusted") {
            return Ok(DriftState::Untrusted);
        }
        return Err(String::from_utf8_lossy(&ls_out.stderr).into_owned());
    }

    let tools: serde_json::Map<String, serde_json::Value> =
        serde_json::from_slice(&ls_out.stdout).unwrap_or_default();

    for (_tool, versions) in &tools {
        if let Some(arr) = versions.as_array() {
            for entry in arr {
                // Only flag tools sourced from a local config file.
                let source_path = entry["source"]["path"]
                    .as_str()
                    .map(PathBuf::from);
                let is_local = source_path
                    .as_ref()
                    .map(|p| local_config_paths.contains(p))
                    .unwrap_or(false);

                if is_local && entry["installed"].as_bool() == Some(false) {
                    return Ok(DriftState::Missing);
                }
            }
        }
    }

    Ok(DriftState::Healthy)
}

/// Run `mise install` in the specified project directory.
pub async fn install_project_tools(path: &str) -> Result<String, String> {
    let output = Command::new("mise")
        .args(["install"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to run mise install: {e}"))?;

    if output.status.success() {
        Ok(format!("Installed tools in {path}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("mise install failed: {stderr}"))
    }
}

/// Run `mise upgrade` in the specified project directory to update outdated tool pins.
pub async fn update_project_pins(path: &str) -> Result<String, String> {
    let output = Command::new("mise")
        .args(["upgrade"])
        .current_dir(path)
        .output()
        .await
        .map_err(|e| format!("Failed to run mise upgrade: {e}"))?;

    if output.status.success() {
        Ok(format!("Updated tool pins in {path}"))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("mise upgrade failed: {stderr}"))
    }
}
