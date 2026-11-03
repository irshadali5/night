//! 🌙 Night Daemons Manager
//! Supervises the lifecycle of background daemons for the desktop environment using `which` for binary verification.

use night_core::exec::{spawn, spawn_shell};
use tracing::{info, warn};

/// Background daemon specification
pub struct DaemonSpec {
    pub name: &'static str,
    pub command: &'static str,
    pub args: &'static [&'static str],
}

/// The core list of background daemons to maintain
pub const CORE_DAEMONS: &[DaemonSpec] = &[
    DaemonSpec {
        name: "Wallpaper Daemon (swww)",
        command: "swww-daemon",
        args: &[],
    },
    DaemonSpec {
        name: "Status Bar (Ironbar)",
        command: "ironbar",
        args: &[],
    },
    DaemonSpec {
        name: "Notification Daemon (Wired)",
        command: "wired",
        args: &[],
    },
    DaemonSpec {
        name: "OSD Server (SwayOSD)",
        command: "swayosd-server",
        args: &[],
    },
];

/// Starts all core desktop daemons and scripts, verifying binary availability via `which`
pub async fn start_all() {
    info!("Starting Night background daemons...");

    for daemon in CORE_DAEMONS {
        if which::which(daemon.command).is_ok() {
            info!(daemon = %daemon.name, "Initializing daemon");
            spawn(daemon.command, daemon.args);
        } else {
            warn!(
                daemon = %daemon.name,
                binary = %daemon.command,
                "Binary not found on system PATH; skipping daemon autostart."
            );
        }
    }

    // Run wallpaper transition script if swww is installed
    if which::which("swww-daemon").is_ok() {
        spawn_shell("~/.config/swww/swww.sh 2>/dev/null");
    }

    // Launch Wayland idle monitor (wayidle) if installed
    let user_cfg = night_core::UserConfig::load();
    if which::which("wayidle").is_ok() {
        let lock_cmd = &user_cfg.apps.lock_cmd;
        let idle_cmd = format!("wayidle -t 300 -s \"{lock_cmd}\" 2>/dev/null || ~/.config/wayidle/wayidle.sh 2>/dev/null");
        info!(command = %idle_cmd, "Starting wayidle session watcher");
        spawn_shell(&idle_cmd);
    }

    // Initialize Night Light / Gamma temperature if enabled in night.toml
    if user_cfg.gamma.enabled {
        info!(
            night_temp = user_cfg.gamma.night_temp,
            "Initializing automated night light gamma schedule"
        );
        night_core::gamma::set_temperature(user_cfg.gamma.night_temp);
    }

    info!("Night daemon supervisor initialization finished.");
}
