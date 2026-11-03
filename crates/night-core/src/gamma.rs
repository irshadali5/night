//! 🌙 Night Light (Gamma) & Idle Inhibit Management
//! Manages blue-light reduction for late-night ergonomics and presentation/compile idle inhibition.

use crate::config::UserConfig;
use crate::exec::spawn_shell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tracing::info;

static CURRENT_TEMP: AtomicU32 = AtomicU32::new(6500);
static IS_NIGHT_MODE: AtomicBool = AtomicBool::new(false);
static IS_INHIBITED: AtomicBool = AtomicBool::new(false);

/// Returns the current active color temperature in Kelvin (e.g. 6500K, 3800K)
pub fn current_temperature() -> u32 {
    CURRENT_TEMP.load(Ordering::Relaxed)
}

/// Returns true if night light mode (warm color temperature) is currently enabled
pub fn is_night_mode() -> bool {
    IS_NIGHT_MODE.load(Ordering::Relaxed)
}

/// Returns true if idle inhibition (presentation/compile mode) is currently active
pub fn is_inhibited() -> bool {
    IS_INHIBITED.load(Ordering::Relaxed)
}

/// Sets screen color temperature in Kelvin
pub fn set_temperature(temp: u32) {
    let clamped = temp.clamp(1000, 10000);
    CURRENT_TEMP.store(clamped, Ordering::Relaxed);
    IS_NIGHT_MODE.store(clamped < 5500, Ordering::Relaxed);
    info!(temp = clamped, "Setting display color temperature (night light)");

    // Apply color temperature using available Wayland gamma tool
    if which::which("wlsunset").is_ok() {
        // Kill existing wlsunset instance and spawn with new fixed temperature
        spawn_shell(&format!(
            "killall -q wlsunset 2>/dev/null; wlsunset -t {clamped} -T {clamped} >/dev/null 2>&1 &"
        ));
    } else if which::which("gammastep").is_ok() {
        spawn_shell(&format!(
            "killall -q gammastep 2>/dev/null; gammastep -O {clamped} >/dev/null 2>&1 &"
        ));
    } else if which::which("hyprsunset").is_ok() {
        spawn_shell(&format!(
            "killall -q hyprsunset 2>/dev/null; hyprsunset -t {clamped} >/dev/null 2>&1 &"
        ));
    }
}

/// Toggles between day temperature (6500K) and configured night temperature (e.g. 3800K)
pub fn toggle_night_light() -> u32 {
    let cfg = UserConfig::load();
    let night_temp = cfg.gamma.night_temp;
    let day_temp = cfg.gamma.day_temp;

    let new_temp = if is_night_mode() {
        day_temp
    } else {
        night_temp
    };

    set_temperature(new_temp);
    new_temp
}

/// Toggles idle inhibition (presentation mode)
/// When inhibited, screen locking and DPMS sleep are suspended.
pub fn toggle_inhibit() -> bool {
    let current = IS_INHIBITED.load(Ordering::Relaxed);
    let new_val = !current;
    IS_INHIBITED.store(new_val, Ordering::Relaxed);

    if new_val {
        info!("Idle inhibition ENABLED (Presentation Mode active - sleep suspended)");
        if which::which("wayidle").is_ok() {
            spawn_shell("killall -STOP wayidle 2>/dev/null");
        }
    } else {
        info!("Idle inhibition DISABLED (Normal idle power saving restored)");
        if which::which("wayidle").is_ok() {
            spawn_shell("killall -CONT wayidle 2>/dev/null");
        }
    }

    new_val
}
