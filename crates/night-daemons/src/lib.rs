//! 🌙 Night Daemons Manager
//! Supervises the lifecycle of background daemons for the desktop environment using `which` for binary verification.

use night_core::exec::{spawn, spawn_shell};
use tracing::{info, warn};

pub mod config_watcher;
pub mod diurnal_watcher;
pub mod idle_watcher;
pub mod power_watcher;
pub mod sleep_watcher;
pub mod wallpaper;

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
    DaemonSpec {
        name: "Elephant Provider Service",
        command: "elephant",
        args: &[],
    },
    DaemonSpec {
        name: "Walker Launcher Service",
        command: "walker",
        args: &["--gapplication-service"],
    },
];

/// Checks whether a process with the given executable name is already running via `/proc`.
fn is_process_running(name: &str) -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let pid_str = file_name.to_string_lossy();
        if !pid_str.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let comm_path = entry.path().join("comm");
        if let Ok(comm) = std::fs::read_to_string(comm_path) {
            if comm.trim() == name {
                return true;
            }
        }
    }
    false
}

/// Starts all core desktop daemons and scripts, verifying binary availability via `which`
/// and skipping daemons that are already running.
pub async fn start_all() {
    info!("Starting Night background daemons...");

    for daemon in CORE_DAEMONS {
        if is_process_running(daemon.command) {
            info!(daemon = %daemon.name, binary = %daemon.command, "Daemon already running; skipping duplicate spawn");
            continue;
        }
        if which::which(daemon.command).is_ok() {
            info!(daemon = %daemon.name, "Initializing daemon");
            spawn(daemon.command, daemon.args);
        } else if daemon.command == "swww-daemon" && which::which("swaybg").is_ok() {
            info!("swww-daemon not installed; falling back to swaybg for wallpaper");
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

    // Launch pure-Rust Wayland clipboard daemon (rsclipd) or fallback recorder
    if which::which("rsclipd").is_ok() {
        info!("Starting pure-Rust Wayland clipboard daemon (rsclipd watch)");
        spawn_shell("rsclipd watch >/dev/null 2>&1 &");
    } else if which::which("wl-paste").is_ok() {
        info!("Starting Wayland clipboard history recorder via wl-paste fallback");
        spawn_shell("wl-paste --type text --watch nightctl clip-record >/dev/null 2>&1 &");
    }

    // Synchronize wallpaper with current Bihar Night theme variant
    wallpaper::sync_wallpaper_theme(user_cfg.general.theme);

    // Spawn background battery power monitoring & low-energy notification alerts
    tokio::spawn(async {
        power_watcher::supervise_power_alerts().await;
    });

    // Spawn background configuration file auto-reloader (rules.ron, monitors.ron, night.toml)
    tokio::spawn(async {
        config_watcher::supervise_config_watcher().await;
    });

    // Spawn background systemd-logind sleep and session lock supervisor
    tokio::spawn(async {
        sleep_watcher::supervise_sleep_signals().await;
    });

    // Spawn background smart idle & staged power reduction supervisor
    tokio::spawn(async {
        idle_watcher::supervise_idle().await;
    });

    // Spawn background diurnal day/night theme cycle supervisor
    tokio::spawn(async {
        diurnal_watcher::supervise_diurnal_schedule().await;
    });

    info!("Night daemon supervisor initialization finished.");
}
