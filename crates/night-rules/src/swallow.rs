//! 🌙 Bihar Night Terminal Window Swallowing Engine
//! Dynamically hides parent terminal windows when a GUI application is spawned from them,
//! and seamlessly unswallows/restores the parent terminal when the child application exits.

use pinnacle_api::signal::WindowSignal;
use pinnacle_api::window::{self, WindowHandle};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tracing::info;

/// Mapping from child window ID to (parent window ID, tag name)
static SWALLOW_REGISTRY: OnceLock<Mutex<HashMap<u32, (u32, String)>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<u32, (u32, String)>> {
    SWALLOW_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Checks if an app_id corresponds to a known terminal emulator
pub fn is_terminal_app(app_id: &str) -> bool {
    let lower = app_id.to_lowercase();
    lower.contains("alacritty")
        || lower.contains("rio")
        || lower.contains("foot")
        || lower.contains("kitty")
        || lower.contains("wezterm")
        || lower.contains("xterm")
        || lower.contains("terminal")
}

/// Checks if an app_id corresponds to an ephemeral overlay or scratchpad that should NOT swallow
pub fn is_scratchpad_or_overlay(app_id: &str) -> bool {
    let lower = app_id.to_lowercase();
    lower.contains("scratchpad")
        || lower.contains("satty")
        || lower.contains("swayosd")
        || lower.contains("settings")
}

/// Swallows a parent terminal under a child GUI window
pub fn swallow(child: &WindowHandle, parent: &WindowHandle) -> bool {
    if child.id() == parent.id() {
        return false;
    }

    if let Some(active_tag) = parent.tags().next() {
        let tag_name = active_tag.name();
        info!(
            child_id = child.id(),
            child_app = %child.app_id(),
            parent_id = parent.id(),
            parent_app = %parent.app_id(),
            tag = %tag_name,
            "Swallowing parent terminal window"
        );

        // Conceal parent from the tag
        parent.set_tag(&active_tag, false);

        // Record swallow mapping
        registry()
            .lock()
            .unwrap()
            .insert(child.id(), (parent.id(), tag_name.clone()));

        // Ensure child is active on the tag and focused
        child.set_tag(&active_tag, true);
        child.raise();
        child.set_focused(true);
        true
    } else {
        false
    }
}

/// Restores a swallowed parent window when its child window exits
pub fn handle_window_destroyed(destroyed_id: u32) {
    let mapping = {
        let mut reg = registry().lock().unwrap();
        reg.remove(&destroyed_id)
    };

    if let Some((parent_id, tag_name)) = mapping {
        info!(
            destroyed_child = destroyed_id,
            restoring_parent = parent_id,
            tag = %tag_name,
            "Child window closed, restoring swallowed terminal"
        );

        let parent = WindowHandle::from_id(parent_id);
        if let Some(tag) = pinnacle_api::tag::get(&tag_name) {
            parent.set_tag(&tag, true);
            parent.raise();
            parent.set_focused(true);
        }
    }
}

/// Restores all currently swallowed terminals
pub fn unswallow_all() {
    let entries: Vec<(u32, (u32, String))> = {
        let mut reg = registry().lock().unwrap();
        reg.drain().collect()
    };

    for (_child_id, (parent_id, tag_name)) in entries {
        let parent = WindowHandle::from_id(parent_id);
        if let Some(tag) = pinnacle_api::tag::get(&tag_name) {
            parent.set_tag(&tag, true);
            parent.raise();
        }
    }
    info!("All swallowed windows unswallowed");
}

/// Sets up lifecycle hooks for automatic window swallowing and restoration
pub fn setup_swallow_listeners() {
    info!("Initializing Bihar Night window swallowing lifecycle signals...");

    // Connect window destroyed signal to restore swallowed terminals
    let _destroy_handle = Box::leak(Box::new(window::connect_signal(
        WindowSignal::Destroyed(Box::new(|win, _title, _app_id| {
            handle_window_destroyed(win.id());
        })),
    )));

    // Connect window created signal to automatically evaluate swallowing
    let _created_handle = Box::leak(Box::new(window::connect_signal(
        WindowSignal::Created(Box::new(|new_win| {
            let child_app = new_win.app_id();
            if is_terminal_app(&child_app) || is_scratchpad_or_overlay(&child_app) {
                return;
            }

            // If the currently focused window on the tag is a terminal, swallow it!
            if let Some(focused) = window::get_focused() {
                if is_terminal_app(&focused.app_id()) && !is_scratchpad_or_overlay(&focused.app_id()) {
                    swallow(new_win, &focused);
                }
            }
        })),
    )));
}
