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

/// Queries current default audio output volume using `wpctl` or `pactl`
pub fn get_volume() -> Option<VolumeInfo> {
    if which::which("wpctl").is_ok() {
        if let Ok(out) = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
        {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Expected format: "Volume: 0.65" or "Volume: 0.65 [MUTED]"
                let is_muted = stdout.contains("[MUTED]");
                if let Some(vol_part) = stdout.split_whitespace().nth(1) {
                    if let Ok(vol_float) = vol_part.parse::<f32>() {
                        let pct = (vol_float * 100.0).round().min(150.0) as u8;
                        return Some(VolumeInfo {
                            percentage: pct,
                            is_muted,
                        });
                    }
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

