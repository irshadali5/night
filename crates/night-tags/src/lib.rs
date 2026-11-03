//! 🌙 Night Tag & Workspace Management
//! Configures AwesomeWM-style dynamic tags and screen assignments with multi-monitor hotplugging.

use pinnacle_api::{output, tag};
use tracing::info;

pub const DEFAULT_TAG_NAMES: [&str; 9] = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

/// Initializes dynamic desktop tags across all connected outputs and monitors plugged in later.
pub fn setup_tags() {
    info!("Configuring dynamic tags [1..9] with multi-monitor hotplug detection...");

    // `for_each_output` automatically registers tags on existing outputs
    // AND attaches a listener for newly connected displays at runtime.
    output::for_each_output(|op| {
        let name = op.name();
        info!(output = %name, "Initializing workspace tags [1..9] on display output");
        tag::add(op, DEFAULT_TAG_NAMES).for_each(drop);
    });

    info!("Dynamic multi-monitor tag system initialized.");
}

/// Switches to the specified tag on the focused output
pub fn switch_to_tag(name: &str) {
    if let Some(t) = tag::get(name) {
        t.switch_to();
    }
}
