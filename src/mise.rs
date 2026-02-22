use crate::model::{ConfigFile, InstalledTool, InstalledToolVersion, RegistryEntry};
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
