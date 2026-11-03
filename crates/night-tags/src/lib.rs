//! 🌙 Night Tag & Workspace Management
//! Configures AwesomeWM-style dynamic tags and screen assignments.

use pinnacle_api::{output, tag};
use tracing::info;

pub const DEFAULT_TAG_NAMES: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

/// Initializes all desktop tags on all connected outputs
pub fn setup_tags() {
    info!("Registering default tags [1..9] on outputs...");

    let outputs = output::get_all().collect::<Vec<_>>();
    if outputs.is_empty() {
        if let Some(focused) = output::get_focused() {
            tag::add(&focused, DEFAULT_TAG_NAMES).for_each(drop);
        }
    } else {
        for op in outputs {
            tag::add(&op, DEFAULT_TAG_NAMES).for_each(drop);
        }
    }

    info!("All default tags registered.");
}

/// Switches to the specified tag on the focused output
pub fn switch_to_tag(name: &str) {
    if let Some(t) = tag::get(name) {
        t.switch_to();
    }
}
