//! 🌙 Night Monitor & Display Profiles
//! Strongly-typed declarative display configurations parsed from RON (`~/.config/night/monitors.ron`).
//! Configures resolution, refresh rate, HiDPI scaling, position offsets, transforms, and VRR per monitor connector.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

/// Complete profile for a physical monitor connector
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitorProfile {
    /// Connector name (e.g. "eDP-1", "DP-1", "HDMI-A-1", or "*" for fallback)
    pub name: String,
    /// HiDPI scaling factor (e.g. 1.0, 1.25, 1.5, 2.0)
    #[serde(default)]
    pub scale: Option<f32>,
    /// Display resolution and refresh rate
    #[serde(default)]
    pub mode: Option<MonitorMode>,
    /// Position coordinates in global Wayland compositor space
    #[serde(default)]
    pub position: Option<MonitorPosition>,
    /// Screen orientation transform
    #[serde(default)]
    pub transform: Option<MonitorTransform>,
    /// Variable refresh rate / Adaptive sync
    #[serde(default)]
    pub vrr: Option<MonitorVrr>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitorMode {
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub refresh_rate_hz: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitorPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MonitorTransform {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MonitorVrr {
    Off,
    AlwaysOn,
    OnDemand,
}

impl MonitorProfile {
    /// Resolves path to `monitors.ron` via `dirs`
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("night/monitors.ron")
    }

    /// Loads all configured monitor profiles from RON
    pub fn load_all() -> Vec<Self> {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match ron::from_str::<Vec<Self>>(&content) {
                    Ok(profiles) => {
                        info!(path = %path.display(), count = profiles.len(), "Loaded monitor profiles from RON");
                        return profiles;
                    }
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "Failed to parse monitors.ron; using defaults");
                    }
                },
                Err(e) => {
                    warn!(path = %path.display(), error = %e, "Failed to read monitors.ron");
                }
            }
        }

        Self::defaults()
    }

    /// Saves all configured monitor profiles to RON
    pub fn save_all(profiles: &[Self]) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = ron::ser::to_string_pretty(profiles, ron::ser::PrettyConfig::default())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&path, content)?;
        info!(path = %path.display(), count = profiles.len(), "Saved monitor profiles to RON");
        Ok(())
    }

    /// Returns standard default profiles (wildcard fallback)
    pub fn defaults() -> Vec<Self> {
        vec![MonitorProfile {
            name: "*".to_string(),
            scale: Some(1.0),
            mode: None,
            position: None,
            transform: Some(MonitorTransform::Normal),
            vrr: Some(MonitorVrr::Off),
        }]
    }

    /// Applies this profile's configuration to a Pinnacle output handle
    pub fn apply_to(&self, op: &pinnacle_api::output::OutputHandle) {
        let name = op.name();
        info!(output = %name, profile = %self.name, "Applying monitor configuration profile");

        if let Some(scale) = self.scale {
            op.set_scale(scale);
        }

        if let Some(ref mode) = self.mode {
            op.set_mode(
                mode.width,
                mode.height,
                mode.refresh_rate_hz.map(|hz| hz * 1000),
            );
        }

        if let Some(ref pos) = self.position {
            op.set_loc(pos.x, pos.y);
        }

        if let Some(transform) = self.transform {
            use pinnacle_api::output::Transform;
            let p_transform = match transform {
                MonitorTransform::Normal => Transform::Normal,
                MonitorTransform::Rotate90 => Transform::_90,
                MonitorTransform::Rotate180 => Transform::_180,
                MonitorTransform::Rotate270 => Transform::_270,
                MonitorTransform::Flipped => Transform::Flipped,
            };
            op.set_transform(p_transform);
        }

        if let Some(vrr) = self.vrr {
            use pinnacle_api::output::Vrr;
            let p_vrr = match vrr {
                MonitorVrr::Off => Vrr::Off,
                MonitorVrr::AlwaysOn => Vrr::AlwaysOn,
                MonitorVrr::OnDemand => Vrr::OnDemand,
            };
            op.set_vrr(p_vrr);
        }
    }
}

