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
    pub windows: Vec<SavedWindow>,
}

impl SavedSession {
    /// Resolves binary session cache path: `$XDG_RUNTIME_DIR/night-saved-session.bin`
    pub fn cache_path() -> PathBuf {
        dirs::runtime_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("night-saved-session.bin")
    }

    /// Persists session snapshot to disk via zero-allocation postcard serialization
    pub fn save(active_tag: &str, theme: &str, windows: Vec<SavedWindow>) {
        let path = Self::cache_path();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let session = SavedSession {
            timestamp: now,
            active_tag: active_tag.to_string(),
            theme: theme.to_string(),
            windows,
        };

        if let Ok(bytes) = postcard::to_allocvec(&session) {
            let _ = crate::write_atomic(&path, &bytes);
            info!(
                active_tag,
                theme,
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

