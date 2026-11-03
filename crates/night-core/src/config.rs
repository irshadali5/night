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
    #[serde(default)]
    pub gamma: GammaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub autostart_daemons: bool,
    pub primary_monitor: Option<String>,
    pub default_tag: String,
    pub border_width: u32,
    pub inner_gaps: f32,
    pub outer_gaps: f32,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autostart_daemons: true,
            primary_monitor: None,
            default_tag: "1".to_string(),
            border_width: 2,
            inner_gaps: 6.0,
            outer_gaps: 10.0,
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
    pub term_file_manager: String,
    pub browser: String,
    pub alt_browser: String,
    pub lock_cmd: String,
}

impl Default for AppsConfig {
    fn default() -> Self {
        Self {
            terminal: "alacritty".to_string(),
            alt_terminal: "rio".to_string(),
            launcher: "anyrun".to_string(),
            minimal_launcher: "yofi".to_string(),
            file_manager: "favnyr".to_string(),
            term_file_manager: "yazi".to_string(),
            browser: "zen-browser".to_string(),
            alt_browser: "brave".to_string(),
            lock_cmd: "veila".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GammaConfig {
    pub enabled: bool,
    pub day_temp: u32,
    pub night_temp: u32,
}

impl Default for GammaConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            day_temp: 6500,
            night_temp: 3800,
        }
    }
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            apps: AppsConfig::default(),
            gamma: GammaConfig::default(),
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

    /// Saves user configuration to standard XDG path
    pub fn save(&self) -> Result<(), std::io::Error> {
        let config_path = Self::path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&config_path, content)?;
        info!(path = %config_path.display(), "Saved TOML configuration");
        Ok(())
    }
}
