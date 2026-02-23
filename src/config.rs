use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MisetuiConfig {
    #[serde(default = "default_scan_dirs")]
    pub scan_dirs: Vec<PathBuf>,
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

fn default_scan_dirs() -> Vec<PathBuf> {
    let mut scan_dirs = Vec::new();
    if let Some(home) = dirs::home_dir() {
        scan_dirs.push(home.join("projects"));
    }
    // Also include CWD
    if let Ok(cwd) = std::env::current_dir() {
        scan_dirs.push(cwd);
    }
    scan_dirs
}

fn default_max_depth() -> usize {
    3
}

impl Default for MisetuiConfig {
    fn default() -> Self {
        Self {
            scan_dirs: default_scan_dirs(),
            max_depth: default_max_depth(),
        }
    }
}

impl MisetuiConfig {
    /// Load from ~/.config/misetui/config.toml; returns defaults if file absent or parse fails.
    pub fn load() -> Self {
        let config_path = dirs::config_dir().map(|d| d.join("misetui").join("config.toml"));

        let Some(path) = config_path else {
            return Self::default();
        };

        let Ok(contents) = std::fs::read_to_string(&path) else {
            return Self::default();
        };

        toml::from_str(&contents).unwrap_or_default()
    }

    /// Save to ~/.config/misetui/config.toml. Creates directories if needed.
    pub fn save(&self) -> Result<(), String> {
        let config_path = dirs::config_dir()
            .map(|d| d.join("misetui").join("config.toml"))
            .ok_or_else(|| "Could not determine config directory".to_string())?;

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config dir: {e}"))?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {e}"))?;

        std::fs::write(&config_path, contents)
            .map_err(|e| format!("Failed to write config: {e}"))?;

        Ok(())
    }
}
