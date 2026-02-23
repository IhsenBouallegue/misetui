use crate::model::{
    ConfigFile, DriftState, EnvVar, EnvVarEntry, InstalledTool, InstalledToolVersion, MiseSetting,
    MiseTask, OutdatedEntry, OutdatedTool, PruneCandidate, RegistryEntry,
};
use std::collections::BTreeMap;
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

/// Check the health of the current working directory's mise tool requirements.
/// Shells out to `mise status` and maps the output to a `DriftState`.
///
/// - `DriftState::NoConfig`  — no config file applies to CWD (empty output or "no config" message)
/// - `DriftState::Missing`   — at least one required tool is not installed
/// - `DriftState::Drifted`   — tools installed but version mismatch (non-zero exit)
/// - `DriftState::Healthy`   — all tools present and at the requested version
pub async fn check_cwd_drift() -> Result<DriftState, String> {
    // `mise status` exits 0 if all tools are installed and matching the config.
    // It exits non-zero if any are missing or drifted.
    // When no config applies to the CWD, stdout is empty and stderr may contain
    // a "no config" / "not found" message, or both are simply empty.
    let output = Command::new("mise")
        .args(["status"])
        .output()
        .await
        .map_err(|e| format!("Failed to run mise status: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // No config: mise prints nothing or a "no config" / "not found" message.
    if stdout.trim().is_empty() && stderr.trim().is_empty() {
        return Ok(DriftState::NoConfig);
    }

    if stderr.to_lowercase().contains("no config")
        || stderr.to_lowercase().contains("not found")
    {
        return Ok(DriftState::NoConfig);
    }

    if !output.status.success() {
        // Non-zero exit means a tool is missing or drifted — check for "missing" keyword.
        if stderr.to_lowercase().contains("missing") || stdout.to_lowercase().contains("missing") {
            return Ok(DriftState::Missing);
        }
        return Ok(DriftState::Drifted);
    }

    // Exit 0 — tools are present. Double-check stdout for any "missing" keyword as a safeguard.
    if stdout.to_lowercase().contains("missing") {
        return Ok(DriftState::Missing);
    }

    Ok(DriftState::Healthy)
}
