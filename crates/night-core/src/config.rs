//! 🌙 Night TOML Configuration
//! Human-readable user settings and theme options parsed from TOML using standard XDG paths (`dirs`).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub apps: AppsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub autostart_daemons: bool,
    pub primary_monitor: Option<String>,
    pub default_tag: String,
    pub border_width: u32,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autostart_daemons: true,
            primary_monitor: None,
            default_tag: "1".to_string(),
            border_width: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppsConfig {
    pub terminal: String,
    pub alt_terminal: String,
    pub launcher: String,
    pub minimal_launcher: String,
    pub file_manager: String,
    pub lock_cmd: String,
}

impl Default for AppsConfig {
    fn default() -> Self {
        Self {
            terminal: "alacritty".to_string(),
            alt_terminal: "rio".to_string(),
            launcher: "anyrun".to_string(),
            minimal_launcher: "yofi".to_string(),
            file_manager: "yazi".to_string(),
            lock_cmd: "veila".to_string(),
        }
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            apps: AppsConfig::default(),
        }
    }
}

impl UserConfig {
    /// Resolves standard XDG config directory via `dirs`
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("night/night.toml")
    }

    /// Loads user configuration from standard XDG path
    pub fn load() -> Self {
        let config_path = Self::path();
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(cfg) => {
                        info!(path = %config_path.display(), "Loaded TOML configuration");
                        return cfg;
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to parse night.toml, falling back to defaults");
                    }
                },
                Err(e) => {
                    warn!(error = %e, "Failed to read night.toml, using defaults");
                }
            }
        }
        Self::default()
    }
}
