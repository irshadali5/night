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

/// Parses sysfs battery `capacity` and `status` file contents into a [`BatteryInfo`].
pub fn parse_battery_values(capacity_str: &str, status_str: Option<&str>) -> Option<BatteryInfo> {
    let percentage = capacity_str.trim().parse::<u8>().ok()?;
    let status = status_str
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown".to_string());
    let is_charging =
        status.eq_ignore_ascii_case("charging") || status.eq_ignore_ascii_case("full");

    Some(BatteryInfo {
        percentage: percentage.min(100),
        is_charging,
        status,
    })
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
            if let Ok(cap_str) = fs::read_to_string(path.join("capacity")) {
                let status_str = fs::read_to_string(path.join("status")).ok();
                if let Some(info) = parse_battery_values(&cap_str, status_str.as_deref()) {
                    return Some(info);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_battery_values() {
        let charging = parse_battery_values("85\n", Some("Charging\n")).unwrap();
        assert_eq!(charging.percentage, 85);
        assert!(charging.is_charging);
        assert_eq!(charging.glyph(), "󰂄");

        let discharging = parse_battery_values("120", Some("Discharging")).unwrap();
        assert_eq!(discharging.percentage, 100);
        assert!(!discharging.is_charging);
        assert_eq!(discharging.display_string(), "󰁹 100%");

        assert!(parse_battery_values("invalid", None).is_none());
    }
}

