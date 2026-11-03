//! 🌙 Night Wayland Clipboard History Manager
//! Ephemeral clipboard ring buffer serialized with `postcard` in `$XDG_RUNTIME_DIR/night-clipboard.bin`.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

const MAX_CLIPBOARD_ENTRIES: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClipboardItem {
    pub content: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClipboardHistory {
    pub items: Vec<ClipboardItem>,
}

static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());

impl ClipboardHistory {
    /// Resolves standard ephemeral path in `$XDG_RUNTIME_DIR`
    pub fn cache_path() -> PathBuf {
        dirs::runtime_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("night-clipboard.bin")
    }

    /// Loads history from postcard binary cache
    pub fn load() -> Self {
        let _guard = CLIPBOARD_LOCK.lock().unwrap();
        let path = Self::cache_path();
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(history) = postcard::from_bytes::<Self>(&bytes) {
                    return history;
                }
            }
        }
        Self::default()
    }

    /// Persists history to postcard binary cache
    pub fn save(&self) {
        let path = Self::cache_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(bytes) = postcard::to_stdvec(self) {
            let _ = fs::write(&path, bytes);
        }
    }

    /// Adds new text to clipboard ring buffer (avoids duplicate adjacent entries)
    pub fn record_text(text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }

        let mut history = Self::load();
        let _guard = CLIPBOARD_LOCK.lock().unwrap();

        // Avoid adding duplicate if it's already the most recent item
        if let Some(first) = history.items.first() {
            if first.content == trimmed {
                return;
            }
        }

        // Remove previous occurrence if exists
        history.items.retain(|item| item.content != trimmed);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        history.items.insert(
            0,
            ClipboardItem {
                content: trimmed.to_string(),
                timestamp: now,
            },
        );

        if history.items.len() > MAX_CLIPBOARD_ENTRIES {
            history.items.truncate(MAX_CLIPBOARD_ENTRIES);
        }

        history.save();
        info!(length = trimmed.len(), "Recorded new clipboard history entry");
    }

    /// Clears all clipboard history
    pub fn clear_all() {
        let _guard = CLIPBOARD_LOCK.lock().unwrap();
        let empty = Self::default();
        empty.save();
        info!("Cleared clipboard history");
    }

    /// Copies a specific history index back into the active Wayland clipboard using `wl-copy`
    pub fn copy_entry_at(index: usize) -> Option<String> {
        let history = Self::load();
        if let Some(item) = history.items.get(index) {
            let content = item.content.clone();
            copy_to_system_clipboard(&content);
            Some(content)
        } else {
            None
        }
    }
}

/// Spawns `wl-copy` to place text into active Wayland clipboard selection
pub fn copy_to_system_clipboard(text: &str) {
    use std::io::Write;
    use std::process::{Command, Stdio};

    if which::which("wl-copy").is_ok() {
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
            info!(length = text.len(), "Pushed entry to Wayland system clipboard via wl-copy");
        }
    } else {
        warn!("wl-copy binary not found on PATH; cannot write to Wayland clipboard");
    }
}

