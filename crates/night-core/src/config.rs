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
    #[serde(default)]
    pub theme: crate::theme::ThemeVariant,
    #[serde(default = "default_true")]
    pub enable_decorations: bool,
    #[serde(default)]
    pub enable_titlebars: bool,
    #[serde(default = "default_titlebar_height")]
    pub titlebar_height: u32,
    #[serde(default)]
    pub auto_theme_schedule: bool,
    #[serde(default)]
    pub power_saver_mode: bool,
}

fn default_true() -> bool {
    true
}

fn default_titlebar_height() -> u32 {
    18
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
            theme: crate::theme::ThemeVariant::Night,
            enable_decorations: true,
            enable_titlebars: false,
            titlebar_height: 18,
            auto_theme_schedule: false,
            power_saver_mode: false,
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
    #[serde(default = "default_media_player")]
    pub media_player: String,
    pub lock_cmd: String,
}

fn default_media_player() -> String {
    "vivi".to_string()
}

impl Default for AppsConfig {
    fn default() -> Self {
        Self {
            terminal: "alacritty".to_string(),
            alt_terminal: "rio".to_string(),
            launcher: "walker".to_string(),
            minimal_launcher: "walker".to_string(),
            file_manager: "favnyr".to_string(),
            term_file_manager: "yazi".to_string(),
            browser: "zen-browser".to_string(),
            alt_browser: "brave".to_string(),
            media_player: "vivi".to_string(),
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
    pub fn save(&self) -> Result<(), crate::error::ConfigError> {
        let config_path = Self::path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).map_err(|source| crate::error::ConfigError::WriteFailed {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::error::ConfigError::WriteFailed {
                path: config_path.clone(),
                source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
            }
        })?;
        fs::write(&config_path, content).map_err(|source| crate::error::ConfigError::WriteFailed {
            path: config_path.clone(),
            source,
        })?;
        info!(path = %config_path.display(), "Saved TOML configuration");
        Ok(())
    }
}
