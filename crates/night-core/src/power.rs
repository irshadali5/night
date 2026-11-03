//! 🌙 Pure-Rust Battery & Power Supply Monitor
//! Zero-dependency sysfs reader for Linux `/sys/class/power_supply` (BAT0, BAT1, ADP1).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BatteryInfo {
    /// Battery capacity percentage (0..100)
    pub percentage: u8,
    /// Whether AC power is actively charging the battery
    pub is_charging: bool,
    /// Raw sysfs status string ("Charging", "Discharging", "Full", "Not charging")
    pub status: String,
}

impl BatteryInfo {
    /// Returns dynamic Nerd Font icon matching battery percentage and charging state
    pub fn glyph(&self) -> &'static str {
        if self.is_charging {
            return "󰂄";
        }

        match self.percentage {
            90..=100 => "󰁹",
            75..=89 => "󰂂",
            50..=74 => "󰂀",
            25..=49 => "󰁾",
            10..=24 => "󰁼",
            _ => "󰁺",
        }
    }

    /// Formats battery status into a clean user-facing string (e.g. "󰂀 74%")
    pub fn display_string(&self) -> String {
        format!("{} {}%", self.glyph(), self.percentage)
    }
}

/// Reads current battery information directly from Linux `/sys/class/power_supply`
pub fn read_battery() -> Option<BatteryInfo> {
    let base = Path::new("/sys/class/power_supply");
    if !base.is_dir() {
        return None;
    }

    let entries = fs::read_dir(base).ok()?;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("BAT") {
            let path = entry.path();
            let capacity_file = path.join("capacity");
            let status_file = path.join("status");

            if let Ok(cap_str) = fs::read_to_string(&capacity_file) {
                if let Ok(percentage) = cap_str.trim().parse::<u8>() {
                    let status = fs::read_to_string(&status_file)
                        .map(|s| s.trim().to_string())
                        .unwrap_or_else(|_| "Unknown".to_string());

                    let is_charging = status.eq_ignore_ascii_case("charging")
                        || status.eq_ignore_ascii_case("full");

                    return Some(BatteryInfo {
                        percentage: percentage.min(100),
                        is_charging,
                        status,
                    });
                }
            }
        }
    }

    None
}

