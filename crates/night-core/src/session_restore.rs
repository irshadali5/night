//! 🌙 Session State Persistence & Window Restoration Engine
//! Serializes open workspace windows, active tag, and layout assignments using `postcard`
//! for instant zero-loss session recovery across restarts or desktop logouts.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// Description of an active application window at session save time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedWindow {
    pub app_id: String,
    pub title: String,
    pub tag: String,
    pub is_floating: bool,
}

/// Comprehensive serialized desktop state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedSession {
    pub timestamp: u64,
    pub active_tag: String,
    pub theme: String,
    #[serde(default)]
    pub layout: String,
    pub windows: Vec<SavedWindow>,
}

/// Resolves a Wayland `app_id` to an executable command when they differ
/// (e.g. reverse-DNS IDs, capitalized IDs, or `.desktop` `StartupWMClass` entries).
pub fn resolve_launch_command(app_id: &str) -> Option<String> {
    let trimmed = app_id.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 1. Direct binary match (exact or lowercase)
    if crate::resolve_executable(trimmed).is_some() {
        return Some(trimmed.to_string());
    }
    let lower = trimmed.to_lowercase();
    if crate::resolve_executable(&lower).is_some() {
        return Some(lower);
    }

    // 2. Common Wayland reverse-DNS / branded app_id mappings
    let mapped = match lower.as_str() {
        "org.mozilla.firefox" | "firefox-esr" => Some("firefox"),
        "zen" | "app.zen_browser.zen" | "zen-alpha" => Some("zen-browser"),
        "brave-browser" => Some("brave"),
        "com.mitchellh.ghostty" => Some("ghostty"),
        "org.wezfurlong.wezterm" => Some("wezterm"),
        "org.gnome.nautilus" => Some("nautilus"),
        "code-oss" | "vscodium" => Some("code"),
        _ => trimmed.rsplit('.').next().filter(|seg| *seg != trimmed),
    };

    if let Some(candidate) = mapped {
        let cand_lower = candidate.to_lowercase();
        if crate::resolve_executable(&cand_lower).is_some() {
            return Some(cand_lower);
        }
    }

    None
}

impl SavedSession {
    /// Resolves persistent XDG data path: `$XDG_DATA_HOME/night/saved-session.bin`
    /// so saved sessions survive logouts and reboots.
    pub fn cache_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| h.join(".local/share"))
                    .unwrap_or_else(std::env::temp_dir)
            })
            .join("night/saved-session.bin")
    }

    /// Persists session snapshot to disk via atomic postcard serialization
    pub fn save(active_tag: &str, theme: &str, layout: &str, windows: Vec<SavedWindow>) {
        let path = Self::cache_path();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let session = SavedSession {
            timestamp: now,
            active_tag: active_tag.to_string(),
            theme: theme.to_string(),
            layout: layout.to_string(),
            windows,
        };

        if let Ok(bytes) = postcard::to_allocvec(&session) {
            let _ = crate::write_atomic(&path, &bytes);
            info!(
                active_tag,
                theme,
                layout,
                window_count = session.windows.len(),
                "Saved active desktop session state"
            );
        }
    }

    /// Loads persisted session state if present
    pub fn load() -> Option<Self> {
        let path = Self::cache_path();
        if path.is_file() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(session) = postcard::from_bytes::<Self>(&bytes) {
                    return Some(session);
                }
            }
        }
        None
    }

    /// Clears saved session file
    pub fn clear() {
        let path = Self::cache_path();
        if path.exists() {
            let _ = fs::remove_file(path);
            info!("Saved session state removed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_launch_command_known_binary() {
        assert_eq!(resolve_launch_command("sh"), Some("sh".to_string()));
        assert_eq!(resolve_launch_command(""), None);
        assert_eq!(resolve_launch_command("nonexistent_app_xyz_999"), None);
    }

    #[test]
    fn test_saved_session_roundtrip() {
        let session = SavedSession {
            timestamp: 1_700_000_000,
            active_tag: "2".to_string(),
            theme: "Night".to_string(),
            layout: "MasterStack".to_string(),
            windows: vec![SavedWindow {
                app_id: "Alacritty".to_string(),
                title: "zsh".to_string(),
                tag: "2".to_string(),
                is_floating: false,
            }],
        };
        let bytes = postcard::to_allocvec(&session).expect("serialize");
        let decoded: SavedSession = postcard::from_bytes(&bytes).expect("deserialize");
        assert_eq!(decoded, session);
    }
}

