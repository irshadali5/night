//! 🌙 Pure-Rust Bluetooth Peripheral & Controller Monitor
//! Zero-dependency Linux sysfs reader for `/sys/class/bluetooth` combined with fast command discovery
//! for connected audio headsets, input mice, keyboards, and controllers.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Connected Bluetooth Peripheral
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BluetoothDevice {
    pub mac: String,
    pub name: String,
    pub battery_pct: Option<u8>,
    pub icon: String,
}

/// Global Bluetooth subsystem state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BluetoothInfo {
    pub is_powered: bool,
    pub is_connected: bool,
    pub devices: Vec<BluetoothDevice>,
}

impl BluetoothInfo {
    /// Dynamic Nerd Font icon for Bluetooth state
    pub fn glyph(&self) -> &str {
        if !self.is_powered {
            "󰂲" // Bluetooth disabled / off
        } else if self.is_connected {
            if let Some(first) = self.devices.first() {
                &first.icon
            } else {
                "󰂱" // Bluetooth connected
            }
        } else {
            "󰂯" // Bluetooth idle
        }
    }

    /// User-facing display string (e.g. "󰋋 Sony WH-1000XM4 (80%)")
    pub fn display_string(&self) -> String {
        if !self.is_powered {
            return "󰂲 Off".to_string();
        }

        if !self.is_connected || self.devices.is_empty() {
            return "󰂯 Disconnected".to_string();
        }

        let first = &self.devices[0];
        let bat_str = first
            .battery_pct
            .map(|b| format!(" ({b}%)"))
            .unwrap_or_default();

        let extra = if self.devices.len() > 1 {
            format!(" +{}", self.devices.len() - 1)
        } else {
            String::new()
        };

        format!("{} {}{}{}", first.icon, first.name, bat_str, extra)
    }
}

/// Detects appropriate Nerd Font icon based on device name or type
fn detect_device_icon(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("headset")
        || lower.contains("headphone")
        || lower.contains("buds")
        || lower.contains("airpod")
        || lower.contains("wh-")
        || lower.contains("wf-")
    {
        "󰋋" // Headset
    } else if lower.contains("mouse") || lower.contains("trackpad") {
        "󰍽" // Mouse
    } else if lower.contains("keyboard") || lower.contains("keychron") {
        "󰌌" // Keyboard
    } else if lower.contains("controller") || lower.contains("gamepad") || lower.contains("xbox") || lower.contains("dualsense") {
        "󰊴" // Controller
    } else {
        "󰂯" // Generic Bluetooth
    }
}

/// Reads current Bluetooth state from `/sys/class/bluetooth` and system discovery
pub fn read_bluetooth() -> Option<BluetoothInfo> {
    let base = Path::new("/sys/class/bluetooth");
    if !base.is_dir() {
        return None;
    }

    let entries = fs::read_dir(base).ok()?;
    let mut has_adapter = false;
    let mut is_powered = false;
    let mut link_count = 0usize;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("hci") && !name.contains(':') {
            has_adapter = true;
            // Check rfkill state
            let rfkill_path = entry.path().join("rfkill0/state");
            if let Ok(state) = fs::read_to_string(&rfkill_path) {
                if state.trim() == "1" {
                    is_powered = true;
                }
            } else {
                // Default to powered if adapter sysfs exists without rfkill soft block
                is_powered = true;
            }
        } else if name.contains(':') {
            // HCI link connection e.g. hci0:3585
            link_count += 1;
        }
    }

    if !has_adapter {
        return None;
    }

    let mut devices = Vec::new();

    // Query connected devices names via bluetoothctl if installed
    if is_powered && which::which("bluetoothctl").is_ok() {
        if let Ok(out) = Command::new("bluetoothctl")
            .args(["devices", "Connected"])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    // Line format: "Device AA:BB:CC:DD:EE:FF Device Name"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 && parts[0] == "Device" {
                        let mac = parts[1].to_string();
                        let name = parts[2..].join(" ");
                        let icon = detect_device_icon(&name).to_string();

                        // Try reading battery percentage for this device
                        let bat = read_device_battery(&mac);

                        devices.push(BluetoothDevice {
                            mac,
                            name,
                            battery_pct: bat,
                            icon,
                        });
                    }
                }
            }
        }
    }

    // Fallback if link connections were detected in sysfs but bluetoothctl returned no parsed names
    if devices.is_empty() && link_count > 0 && is_powered {
        devices.push(BluetoothDevice {
            mac: "unknown".into(),
            name: format!("{link_count} Connected Device(s)"),
            battery_pct: None,
            icon: "󰂱".into(),
        });
    }

    let is_connected = !devices.is_empty();

    Some(BluetoothInfo {
        is_powered,
        is_connected,
        devices,
    })
}

/// Attempts to read battery level for a specific Bluetooth device MAC
fn read_device_battery(mac: &str) -> Option<u8> {
    let formatted_mac = mac.replace(':', "_");
    let power_base = Path::new("/sys/class/power_supply");
    if let Ok(entries) = fs::read_dir(power_base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(&formatted_mac) {
                let cap_path = entry.path().join("capacity");
                if let Ok(content) = fs::read_to_string(cap_path) {
                    if let Ok(pct) = content.trim().parse::<u8>() {
                        return Some(pct.min(100));
                    }
                }
            }
        }
    }
    None
}
