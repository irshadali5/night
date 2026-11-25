//! 🌙 Audio Volume Management
//! Provides volume querying and control via PipeWire (`wpctl`) and PulseAudio (`pactl`),
//! with optional SwayOSD on-screen display synchronization.

use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum VolumeAction {
    Raise(u8),
    Lower(u8),
    Set(u8),
    MuteToggle,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VolumeInfo {
    /// Volume level (0..100)
    pub percentage: u8,
    /// Whether audio output is currently muted
    pub is_muted: bool,
}

impl VolumeInfo {
    /// Returns dynamic Nerd Font icon matching volume level and mute status
    pub fn glyph(&self) -> &'static str {
        if self.is_muted {
            return "󰖁";
        }

        match self.percentage {
            70..=100 => "󰕾",
            30..=69 => "󰖀",
            1..=29 => "󰕿",
            _ => "󰖁",
        }
    }

    /// Formats volume status into a user-facing string (e.g. "󰕾 65%" or "󰖁 Muted")
    pub fn display_string(&self) -> String {
        if self.is_muted {
            "󰖁 Muted".to_string()
        } else {
            format!("{} {}%", self.glyph(), self.percentage)
        }
    }
}

/// Parses `wpctl get-volume @DEFAULT_AUDIO_SINK@` output into a [`VolumeInfo`].
pub fn parse_wpctl_volume(stdout: &str) -> Option<VolumeInfo> {
    // Expected format: "Volume: 0.65" or "Volume: 0.65 [MUTED]"
    let is_muted = stdout.contains("[MUTED]");
    let vol_part = stdout.split_whitespace().nth(1)?;
    let vol_float = vol_part.parse::<f32>().ok()?;
    let percentage = (vol_float * 100.0).round().min(150.0) as u8;
    Some(VolumeInfo {
        percentage,
        is_muted,
    })
}

/// Queries current default audio output volume using `wpctl` or `pactl`
pub fn get_volume() -> Option<VolumeInfo> {
    if which::which("wpctl").is_ok() {
        if let Ok(out) = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Some(info) = parse_wpctl_volume(&stdout) {
                    return Some(info);
                }
            }
        }
    }

    None
}

/// Executes a volume action and returns updated volume info
pub fn handle_volume_action(action: VolumeAction) -> Option<VolumeInfo> {
    let has_wpctl = which::which("wpctl").is_ok();
    let has_swayosd = which::which("swayosd-client").is_ok();

    match action {
        VolumeAction::Raise(delta) => {
            if has_wpctl {
                let arg = format!("{delta}%+");
                let _ = Command::new("wpctl")
                    .args(["set-volume", "-l", "1.5", "@DEFAULT_AUDIO_SINK@", &arg])
                    .status();
            }
            if has_swayosd {
                let _ = Command::new("swayosd-client")
                    .args(["--output-volume", "raise"])
                    .status();
            }
            info!(delta, "Audio volume raised");
        }
        VolumeAction::Lower(delta) => {
            if has_wpctl {
                let arg = format!("{delta}%-");
                let _ = Command::new("wpctl")
                    .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &arg])
                    .status();
            }
            if has_swayosd {
                let _ = Command::new("swayosd-client")
                    .args(["--output-volume", "lower"])
                    .status();
            }
            info!(delta, "Audio volume lowered");
        }
        VolumeAction::Set(pct) => {
            if has_wpctl {
                let val = (pct as f32) / 100.0;
                let arg = format!("{:.2}", val);
                let _ = Command::new("wpctl")
                    .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &arg])
                    .status();
            }
            info!(pct, "Audio volume set explicitly");
        }
        VolumeAction::MuteToggle => {
            if has_wpctl {
                let _ = Command::new("wpctl")
                    .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                    .status();
            }
            if has_swayosd {
                let _ = Command::new("swayosd-client")
                    .args(["--output-volume", "mute-toggle"])
                    .status();
            }
            info!("Audio mute toggled");
        }
        VolumeAction::Query => {}
    }

    get_volume()
}

/// An audio output sink device (e.g. Headphones, Speakers, HDMI, USB DAC)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AudioSink {
    pub id: u32,
    pub name: String,
    pub is_default: bool,
}

impl AudioSink {
    /// Friendly icon based on sink name
    pub fn icon(&self) -> &'static str {
        let lower = self.name.to_lowercase();
        if lower.contains("headphone") || lower.contains("headset") {
            "󰋋"
        } else if lower.contains("hdmi") || lower.contains("displayport") || lower.contains("monitor") {
            "󰍹"
        } else if lower.contains("usb") || lower.contains("dac") {
            "󰓃"
        } else {
            "󰕾"
        }
    }
}

/// Discovers all available audio output sinks via `wpctl` or `pactl`
pub fn list_sinks() -> Vec<AudioSink> {
    if which::which("wpctl").is_ok() {
        if let Ok(out) = Command::new("wpctl").arg("status").output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut in_sinks = false;
                let mut sinks = Vec::new();

                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains("Sinks:") {
                        in_sinks = true;
                        continue;
                    }
                    if in_sinks && (trimmed.contains("Sources:") || trimmed.contains("Filters:") || trimmed.contains("Streams:")) {
                        break;
                    }

                    if in_sinks && (trimmed.contains('.') || trimmed.contains('*')) {
                        let is_default = trimmed.contains('*');
                        // Example line: "│  *   49. Built-in Audio Analog Stereo        [vol: 0.65]"
                        // Clean line by removing tree characters
                        let clean = trimmed.replace(['│', '*'], "");
                        let parts: Vec<&str> = clean.split_whitespace().collect();
                        if let Some(first) = parts.first() {
                            let id_str = first.trim_end_matches('.');
                            if let Ok(id) = id_str.parse::<u32>() {
                                let name_end = clean.find('[').unwrap_or(clean.len());
                                let name_start = clean.find(id_str).map(|p| p + id_str.len()).unwrap_or(0);
                                let name = clean[name_start..name_end].trim().trim_start_matches('.').trim().to_string();

                                if !name.is_empty() {
                                    sinks.push(AudioSink {
                                        id,
                                        name,
                                        is_default,
                                    });
                                }
                            }
                        }
                    }
                }

                if !sinks.is_empty() {
                    return sinks;
                }
            }
        }
    }

    // Fallback via pactl if wpctl is unavailable
    if which::which("pactl").is_ok() {
        if let Ok(out) = Command::new("pactl").args(["list", "sinks", "short"]).output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut sinks = Vec::new();
                for (idx, line) in text.lines().enumerate() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let id = parts[0].parse::<u32>().unwrap_or(idx as u32);
                        let name = parts[1].replace(['_', '.'], " ");
                        sinks.push(AudioSink {
                            id,
                            name,
                            is_default: idx == 0,
                        });
                    }
                }
                if !sinks.is_empty() {
                    return sinks;
                }
            }
        }
    }

    Vec::new()
}

/// Sets the active default audio output sink by ID
pub fn set_default_sink(id: u32) -> bool {
    if which::which("wpctl").is_ok() {
        let status = Command::new("wpctl")
            .args(["set-default", &id.to_string()])
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            info!(id, "Default audio sink switched via wpctl");
            return true;
        }
    }

    if which::which("pactl").is_ok() {
        let status = Command::new("pactl")
            .args(["set-default-sink", &id.to_string()])
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            info!(id, "Default audio sink switched via pactl");
            return true;
        }
    }

    false
}

/// Cycles to the next available audio output sink
pub fn cycle_sink() -> Option<AudioSink> {
    let sinks = list_sinks();
    if sinks.is_empty() {
        return None;
    }

    let current_idx = sinks.iter().position(|s| s.is_default).unwrap_or(0);
    let next_idx = (current_idx + 1) % sinks.len();
    let target = &sinks[next_idx];

    set_default_sink(target.id);
    Some(target.clone())
}

/// Parses `wpctl status` output into a list of [`AudioSink`]s.
pub fn parse_wpctl_sinks(text: &str) -> Vec<AudioSink> {
    let mut in_sinks = false;
    let mut sinks = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Sinks:") {
            in_sinks = true;
            continue;
        }
        if in_sinks
            && (trimmed.contains("Sources:")
                || trimmed.contains("Filters:")
                || trimmed.contains("Streams:"))
        {
            break;
        }

        if in_sinks && (trimmed.contains('.') || trimmed.contains('*')) {
            let is_default = trimmed.contains('*');
            let clean = trimmed.replace(['│', '*'], "");
            let parts: Vec<&str> = clean.split_whitespace().collect();
            if let Some(first) = parts.first() {
                let id_str = first.trim_end_matches('.');
                if let Ok(id) = id_str.parse::<u32>() {
                    let name_end = clean.find('[').unwrap_or(clean.len());
                    let name_start = clean.find(id_str).map(|p| p + id_str.len()).unwrap_or(0);
                    let name = clean[name_start..name_end]
                        .trim()
                        .trim_start_matches('.')
                        .trim()
                        .to_string();

                    if !name.is_empty() {
                        sinks.push(AudioSink {
                            id,
                            name,
                            is_default,
                        });
                    }
                }
            }
        }
    }

    sinks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_wpctl_volume() {
        let normal = parse_wpctl_volume("Volume: 0.65").unwrap();
        assert_eq!(normal.percentage, 65);
        assert!(!normal.is_muted);
        assert_eq!(normal.display_string(), "󰖀 65%");

        let muted = parse_wpctl_volume("Volume: 0.40 [MUTED]").unwrap();
        assert_eq!(muted.percentage, 40);
        assert!(muted.is_muted);
        assert_eq!(muted.display_string(), "󰖁 Muted");

        assert!(parse_wpctl_volume("invalid").is_none());
    }

    #[test]
    fn test_parse_wpctl_sinks() {
        let sample = "\
Audio
 ├─ Sinks:
 │  *   49. Built-in Audio Analog Stereo        [vol: 0.65]
 │      52. USB-C Headphones                    [vol: 0.80]
 ├─ Sources:
 │      55. Built-in Mic                        [vol: 1.00]
";
        let sinks = parse_wpctl_sinks(sample);
        assert_eq!(sinks.len(), 2);
        assert_eq!(sinks[0].id, 49);
        assert!(sinks[0].is_default);
        assert_eq!(sinks[0].name, "Built-in Audio Analog Stereo");
        assert_eq!(sinks[1].id, 52);
        assert!(!sinks[1].is_default);
        assert_eq!(sinks[1].icon(), "󰋋");
    }
}

