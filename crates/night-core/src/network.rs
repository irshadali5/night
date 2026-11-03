//! 🌙 Network Connectivity & Wi-Fi Monitor
//! Zero-dependency Linux sysfs reader for `/sys/class/net` combined with fast command discovery
//! for active SSID, local IPv4, and signal strength.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkKind {
    Wifi,
    Ethernet,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkInfo {
    pub interface: String,
    pub kind: NetworkKind,
    pub ssid: Option<String>,
    pub signal_pct: Option<u8>,
    pub ip_addr: Option<String>,
    pub is_online: bool,
}

impl NetworkInfo {
    /// Returns dynamic Nerd Font icon matching network type and Wi-Fi signal quality
    pub fn glyph(&self) -> &'static str {
        if !self.is_online || self.kind == NetworkKind::Disconnected {
            return "󰤮";
        }

        match self.kind {
            NetworkKind::Ethernet => "󰈀",
            NetworkKind::Wifi => {
                let sig = self.signal_pct.unwrap_or(100);
                match sig {
                    80..=100 => "󰤨",
                    60..=79 => "󰤥",
                    40..=59 => "󰤢",
                    20..=39 => "󰤟",
                    _ => "󰤯",
                }
            }
            NetworkKind::Disconnected => "󰤮",
        }
    }

    /// Formats network status into a clean user-facing string (e.g. "󰤨 Home-WiFi (192.168.1.10)")
    pub fn display_string(&self) -> String {
        if !self.is_online || self.kind == NetworkKind::Disconnected {
            return "󰤮 Offline".to_string();
        }

        let ip_part = self
            .ip_addr
            .as_deref()
            .map(|ip| format!(" ({ip})"))
            .unwrap_or_default();

        match self.kind {
            NetworkKind::Wifi => {
                let name = self.ssid.as_deref().unwrap_or(&self.interface);
                format!("{} {}{}", self.glyph(), name, ip_part)
            }
            NetworkKind::Ethernet => {
                format!("{} Wired{}", self.glyph(), ip_part)
            }
            NetworkKind::Disconnected => "󰤮 Offline".to_string(),
        }
    }
}

/// Reads active network interface state, Wi-Fi SSID, and IP address
pub fn read_network() -> Option<NetworkInfo> {
    let base = Path::new("/sys/class/net");
    if !base.is_dir() {
        return None;
    }

    let entries = fs::read_dir(base).ok()?;
    let mut active_iface: Option<(String, NetworkKind)> = None;

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "lo" || name.starts_with("docker") || name.starts_with("veth") || name.starts_with("br-") {
            continue;
        }

        let operstate_path = entry.path().join("operstate");
        if let Ok(state) = fs::read_to_string(&operstate_path) {
            let s = state.trim();
            if s.eq_ignore_ascii_case("up") {
                let kind = if name.starts_with("wl") || entry.path().join("wireless").exists() {
                    NetworkKind::Wifi
                } else if name.starts_with("en") || name.starts_with("eth") {
                    NetworkKind::Ethernet
                } else {
                    NetworkKind::Wifi
                };
                active_iface = Some((name, kind));
                break;
            }
        }
    }

    let (iface, kind) = match active_iface {
        Some(pair) => pair,
        None => {
            return Some(NetworkInfo {
                interface: "none".into(),
                kind: NetworkKind::Disconnected,
                ssid: None,
                signal_pct: None,
                ip_addr: None,
                is_online: false,
            });
        }
    };

    // 2. Query Wi-Fi SSID and Signal Quality if available
    let mut ssid = None;
    let mut signal_pct = None;

    if kind == NetworkKind::Wifi {
        // Try nmcli
        if which::which("nmcli").is_ok() {
            if let Ok(out) = Command::new("nmcli")
                .args(["-t", "-f", "active,ssid,signal", "dev", "wifi"])
                .output()
            {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        if line.starts_with("yes:") {
                            let parts: Vec<&str> = line.split(':').collect();
                            if parts.len() >= 3 {
                                ssid = Some(parts[1].to_string());
                                signal_pct = parts[2].parse::<u8>().ok();
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Fallback to iwgetid for SSID
        if ssid.is_none() && which::which("iwgetid").is_ok() {
            if let Ok(out) = Command::new("iwgetid").arg("-r").output() {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !s.is_empty() {
                        ssid = Some(s);
                    }
                }
            }
        }
    }

    // 3. Query local IPv4 address
    let mut ip_addr = None;
    if which::which("ip").is_ok() {
        if let Ok(out) = Command::new("ip")
            .args(["-4", "addr", "show", &iface])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for word in text.split_whitespace() {
                    if word.contains('/') && word.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        if let Some(ip) = word.split('/').next() {
                            ip_addr = Some(ip.to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    Some(NetworkInfo {
        interface: iface,
        kind,
        ssid,
        signal_pct,
        ip_addr,
        is_online: true,
    })
}

