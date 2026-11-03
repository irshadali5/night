//! 🌙 Night Session State & Caching
//! Ultra-fast binary session serialization using `postcard` and standard XDG runtime paths (`dirs`).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

/// Compact session state saved across compositor restarts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionState {
    pub active_tag: String,
    pub last_reload_timestamp: u64,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            active_tag: "1".to_string(),
            last_reload_timestamp: 0,
        }
    }
}

impl SessionState {
    /// Resolves standard XDG runtime directory via `dirs`
    pub fn runtime_path() -> PathBuf {
        dirs::runtime_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("night-session.bin")
    }

    /// Saves session state to a binary file using `postcard`
    pub fn save(&self) {
        let path = Self::runtime_path();
        match postcard::to_allocvec(self) {
            Ok(bytes) => {
                if let Err(e) = fs::write(&path, &bytes) {
                    error!(path = %path.display(), error = ?e, "Failed to write binary session state");
                } else {
                    info!(bytes = bytes.len(), path = %path.display(), "Saved binary session state via postcard");
                }
            }
            Err(e) => {
                error!(error = ?e, "Failed to serialize session state with postcard");
            }
        }
    }

    /// Loads session state from binary cache using `postcard`
    pub fn load() -> Option<Self> {
        let path = Self::runtime_path();
        if path.exists() {
            match fs::read(&path) {
                Ok(bytes) => match postcard::from_bytes::<Self>(&bytes) {
                    Ok(state) => {
                        info!(path = %path.display(), tag = %state.active_tag, "Restored binary session state via postcard");
                        return Some(state);
                    }
                    Err(e) => {
                        error!(error = ?e, "Failed to deserialize binary session state");
                    }
                },
                Err(e) => {
                    error!(error = ?e, "Failed to read binary state file");
                }
            }
        }
        None
    }
}
