//! 🌙 Night Tag & Workspace Management
//! Configures AwesomeWM-style per-output dynamic tags, monitor profile application,
//! multi-monitor focus navigation, window movement across displays, and DPMS power management.

use night_core::monitors::MonitorProfile;
use pinnacle_api::signal::TagSignal;
use pinnacle_api::{output, tag, window};
use tracing::info;

pub const DEFAULT_TAG_NAMES: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];
pub const DEFAULT_TAG_GLYPHS: [&str; 9] = ["󰣆", "󰈹", "󰉋", "󰎆", "󰙯", "󰌨", "󰈙", "󰭹", "󰝰"];
pub const DEFAULT_TAG_LABELS: [&str; 9] = [
    "Code", "Web", "Files", "Media", "Chat", "Dev", "Docs", "Comm", "Misc",
];

/// Returns the nerd font glyph corresponding to a tag name
pub fn tag_glyph(name: &str) -> &'static str {
    match name {
        "1" => DEFAULT_TAG_GLYPHS[0],
        "2" => DEFAULT_TAG_GLYPHS[1],
        "3" => DEFAULT_TAG_GLYPHS[2],
        "4" => DEFAULT_TAG_GLYPHS[3],
        "5" => DEFAULT_TAG_GLYPHS[4],
        "6" => DEFAULT_TAG_GLYPHS[5],
        "7" => DEFAULT_TAG_GLYPHS[6],
        "8" => DEFAULT_TAG_GLYPHS[7],
        "9" => DEFAULT_TAG_GLYPHS[8],
        _ => "󰝰",
    }
}

/// Returns the semantic label corresponding to a tag name
pub fn tag_label(name: &str) -> &'static str {
    match name {
        "1" => DEFAULT_TAG_LABELS[0],
        "2" => DEFAULT_TAG_LABELS[1],
        "3" => DEFAULT_TAG_LABELS[2],
        "4" => DEFAULT_TAG_LABELS[3],
        "5" => DEFAULT_TAG_LABELS[4],
        "6" => DEFAULT_TAG_LABELS[5],
        "7" => DEFAULT_TAG_LABELS[6],
        "8" => DEFAULT_TAG_LABELS[7],
        "9" => DEFAULT_TAG_LABELS[8],
        _ => "Workspace",
    }
}

/// Initializes dynamic desktop tags across all connected outputs, applying RON monitor profiles.
pub fn setup_tags() {
    info!("Configuring dynamic tags [1..9] with multi-monitor hotplug detection and RON profiles...");
    let profiles = MonitorProfile::load_all();

    // `for_each_output` runs on all existing displays and attaches a listener for newly hotplugged monitors
    output::for_each_output(move |op| {
        let name = op.name();

        // 1. Match and apply configured monitor profile
        if let Some(profile) = profiles.iter().find(|p| p.name == name) {
            profile.apply_to(op);
        } else if let Some(fallback) = profiles.iter().find(|p| p.name == "*") {
            fallback.apply_to(op);
        }

        // 2. Provision independent dynamic tags [1..9] on this physical display
        info!(output = %name, "Initializing per-output workspace tags [1..9]");
        tag::add(op, DEFAULT_TAG_NAMES).for_each(drop);
    });

    // Real-time signal hook: updates postcard binary session state whenever active tag changes
    tag::connect_signal(TagSignal::Active(Box::new(|handle, active| {
        if active {
            let name = handle.name();
            info!(tag = %name, "Active workspace tag updated in real-time");
            let state = night_core::SessionState {
                active_tag: name,
                last_reload_timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            };
            state.save();
        }
    })));

    info!("Multi-monitor tag system with per-output isolation initialized.");
}

/// Switches to the specified tag on the currently focused display output
pub fn switch_to_tag(name: &str) {
    if let Some(focused_op) = output::get_focused() {
        if let Some(t) = tag::get_on_output(name, &focused_op) {
            t.switch_to();
            return;
        }
    }
    if let Some(t) = tag::get(name) {
        t.switch_to();
    }
}

/// Moves the currently focused window to the specified tag on the focused display
pub fn move_focused_to_tag(name: &str) {
    if let Some(focused_win) = window::get_focused() {
        if let Some(focused_op) = output::get_focused() {
            if let Some(t) = tag::get_on_output(name, &focused_op) {
                focused_win.move_to_tag(&t);
                return;
            }
        }
        if let Some(t) = tag::get(name) {
            focused_win.move_to_tag(&t);
        }
    }
}

/// Focuses the next connected and enabled display output
pub fn focus_next_output() {
    let outputs: Vec<_> = output::get_all_enabled().collect();
    if outputs.is_empty() {
        return;
    }
    let current_idx = outputs.iter().position(|op| op.focused()).unwrap_or(0);
    let next_idx = (current_idx + 1) % outputs.len();
    outputs[next_idx].focus();
    info!(output = %outputs[next_idx].name(), "Focused next display output");
}

/// Focuses the previous connected and enabled display output
pub fn focus_prev_output() {
    let outputs: Vec<_> = output::get_all_enabled().collect();
    if outputs.is_empty() {
        return;
    }
    let current_idx = outputs.iter().position(|op| op.focused()).unwrap_or(0);
    let prev_idx = if current_idx == 0 {
        outputs.len() - 1
    } else {
        current_idx - 1
    };
    outputs[prev_idx].focus();
    info!(output = %outputs[prev_idx].name(), "Focused previous display output");
}

/// Focuses a display output by connector name
pub fn focus_output_by_name(name: &str) {
    if let Some(op) = output::get_by_name(name) {
        op.focus();
        info!(output = %name, "Focused display output by name");
    }
}

/// Moves the currently focused window to the next connected display output
pub fn move_focused_to_next_output() {
    if let Some(focused_win) = window::get_focused() {
        let outputs: Vec<_> = output::get_all_enabled().collect();
        if outputs.len() < 2 {
            return;
        }
        let current_idx = outputs.iter().position(|op| op.focused()).unwrap_or(0);
        let next_idx = (current_idx + 1) % outputs.len();
        let target_op = &outputs[next_idx];
        focused_win.move_to_output(target_op);
        target_op.focus();
        focused_win.raise();
        focused_win.set_focused(true);
        info!(target = %target_op.name(), "Moved focused window to next display output");
    }
}

/// Moves the currently focused window to the previous connected display output
pub fn move_focused_to_prev_output() {
    if let Some(focused_win) = window::get_focused() {
        let outputs: Vec<_> = output::get_all_enabled().collect();
        if outputs.len() < 2 {
            return;
        }
        let current_idx = outputs.iter().position(|op| op.focused()).unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            outputs.len() - 1
        } else {
            current_idx - 1
        };
        let target_op = &outputs[prev_idx];
        focused_win.move_to_output(target_op);
        target_op.focus();
        focused_win.raise();
        focused_win.set_focused(true);
        info!(target = %target_op.name(), "Moved focused window to previous display output");
    }
}

/// Moves the currently focused window to a display output specified by connector name
pub fn move_focused_to_output_by_name(name: &str) {
    if let Some(focused_win) = window::get_focused() {
        if let Some(target_op) = output::get_by_name(name) {
            focused_win.move_to_output(&target_op);
            target_op.focus();
            focused_win.raise();
            focused_win.set_focused(true);
            info!(target = %name, "Moved focused window to display output");
        }
    }
}

/// Sets DPMS power state across all connected monitors
pub fn set_dpms(powered: bool) {
    info!(powered, "Setting DPMS display power state on all outputs");
    for op in output::get_all() {
        op.set_powered(powered);
    }
}

/// Toggles DPMS power state across all connected monitors
pub fn toggle_dpms() {
    info!("Toggling DPMS display power state on all outputs");
    for op in output::get_all() {
        op.toggle_powered();
    }
}
