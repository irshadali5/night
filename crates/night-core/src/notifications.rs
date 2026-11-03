//! 🌙 Desktop Notification Center & Do Not Disturb (DND) Daemon
//! Manages notification history recording, DND state toggling, and fast postcard caching.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// Single notification event record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationRecord {
    pub id: u64,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub timestamp: u64,
}

/// Notification history ring-buffer and DND status container
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotificationHistory {
    pub items: Vec<NotificationRecord>,
    pub dnd_enabled: bool,
}

impl Default for NotificationHistory {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            dnd_enabled: false,
        }
    }
}

impl NotificationHistory {
    const MAX_ITEMS: usize = 30;

    /// Resolves runtime cache path: `$XDG_RUNTIME_DIR/night-notifications.bin`
    pub fn cache_path() -> PathBuf {
        dirs::runtime_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("night-notifications.bin")
    }

    /// Loads history from postcard binary cache
    pub fn load() -> Self {
        let path = Self::cache_path();
        if path.is_file() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(history) = postcard::from_bytes::<Self>(&bytes) {
                    return history;
                }
            }
        }
        Self::default()
    }

    /// Persists current history to postcard binary cache
    pub fn save(&self) {
        let path = Self::cache_path();
        if let Ok(bytes) = postcard::to_allocvec(self) {
            let _ = fs::write(path, bytes);
        }
    }

    /// Records a new notification into history
    pub fn record(app_name: &str, summary: &str, body: &str) {
        let mut history = Self::load();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let id = history.items.first().map(|i| i.id + 1).unwrap_or(1);
        history.items.insert(
            0,
            NotificationRecord {
                id,
                app_name: app_name.to_string(),
                summary: summary.to_string(),
                body: body.to_string(),
                timestamp: now,
            },
        );

        if history.items.len() > Self::MAX_ITEMS {
            history.items.truncate(Self::MAX_ITEMS);
        }

        history.save();
        info!(id, app_name, summary, "Recorded desktop notification to history");
    }

    /// Clears all recorded notification history
    pub fn clear_all() {
        let mut history = Self::load();
        history.items.clear();
        history.save();
        info!("Notification history wiped");
    }

    /// Toggles Do Not Disturb (DND) state
    pub fn toggle_dnd() -> bool {
        let mut history = Self::load();
        history.dnd_enabled = !history.dnd_enabled;
        let new_state = history.dnd_enabled;
        history.save();

        if which::which("wired").is_ok() {
            // Signal wired daemon if running
            let arg = if new_state { "--dnd" } else { "--no-dnd" };
            let _ = std::process::Command::new("wired").arg(arg).status();
        }

        info!(dnd = new_state, "Do Not Disturb toggled");
        new_state
    }

    /// Queries active DND state
    pub fn is_dnd() -> bool {
        Self::load().dnd_enabled
    }

    /// Returns dynamic Nerd Font glyph for DND state
    pub fn glyph_dnd(dnd: bool) -> &'static str {
        if dnd {
            "󰂛" // Muted / DND
        } else {
            "󰂚" // Bell / Normal
        }
    }
}

