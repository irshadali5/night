//! 🌙 Display Brightness & Backlight Control Engine
//! Direct zero-dependency Linux sysfs reader for `/sys/class/backlight` with
//! `brightnessctl` and SwayOSD client integration.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BacklightAction {
    Raise(u8),
    Lower(u8),
    Set(u8),
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BacklightInfo {
    pub percentage: u8,
    pub current_raw: u64,
    pub max_raw: u64,
    pub device_name: String,
}

impl BacklightInfo {
    /// Returns dynamic Nerd Font icon matching brightness level
    pub fn glyph(&self) -> &'static str {
        match self.percentage {
            75..=100 => "󰃠",
            50..=74 => "󰃟",
            25..=49 => "󰃞",
            _ => "󰃝",
        }
    }

    /// User-facing display string (e.g. "󰃟 45%")
    pub fn display_string(&self) -> String {
        format!("{} {}%", self.glyph(), self.percentage)
    }
}

/// Discovers primary backlight sysfs device path
fn find_backlight_device() -> Option<(String, PathBuf)> {
    let base = Path::new("/sys/class/backlight");
    if !base.is_dir() {
        return None;
    }

    let entries = fs::read_dir(base).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.join("brightness").is_file() && path.join("max_brightness").is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            return Some((name, path));
        }
    }
    None
}

/// Reads current display backlight level from sysfs
pub fn get_backlight() -> Option<BacklightInfo> {
    let (device_name, path) = find_backlight_device()?;
    let cur_str = fs::read_to_string(path.join("brightness")).ok()?;
    let max_str = fs::read_to_string(path.join("max_brightness")).ok()?;

    let current_raw = cur_str.trim().parse::<u64>().ok()?;
    let max_raw = max_str.trim().parse::<u64>().ok()?;

    if max_raw == 0 {
        return None;
    }

    let percentage = ((current_raw as f64 / max_raw as f64) * 100.0).round().min(100.0) as u8;

    Some(BacklightInfo {
        percentage,
        current_raw,
        max_raw,
        device_name,
    })
}

/// Executes a display brightness adjustment and triggers SwayOSD feedback
pub fn handle_backlight_action(action: BacklightAction) -> Option<BacklightInfo> {
    let has_bctl = which::which("brightnessctl").is_ok();
    let has_swayosd = which::which("swayosd-client").is_ok();

    match action {
        BacklightAction::Raise(delta) => {
            if has_bctl {
                let arg = format!("{delta}%+");
                let _ = Command::new("brightnessctl").args(["set", &arg]).status();
            }
            if has_swayosd {
                let _ = Command::new("swayosd-client").args(["--brightness", "raise"]).status();
            }
            info!(delta, "Backlight brightness raised");
        }
        BacklightAction::Lower(delta) => {
            if has_bctl {
                let arg = format!("{delta}%-");
                let _ = Command::new("brightnessctl").args(["set", &arg]).status();
            }
            if has_swayosd {
                let _ = Command::new("swayosd-client").args(["--brightness", "lower"]).status();
            }
            info!(delta, "Backlight brightness lowered");
        }
        BacklightAction::Set(pct) => {
            if has_bctl {
                let arg = format!("{pct}%");
                let _ = Command::new("brightnessctl").args(["set", &arg]).status();
            }
            info!(pct, "Backlight brightness set explicitly");
        }
        BacklightAction::Query => {}
    }

    get_backlight()
}

