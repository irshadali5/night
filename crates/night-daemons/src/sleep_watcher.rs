//! 🌙 Tokyo Night Sleep & Session Lock Daemon
//! Monitors systemd-logind D-Bus signals for system sleep/suspend events (`PrepareForSleep`).
//! Automatically locks the Wayland session with Veila and blanks displays before suspending,
//! and wakes outputs and refreshes hardware telemetry upon system resume.

use night_core::config::UserConfig;
use night_core::exec::spawn_shell;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use tracing::{debug, error, info, warn};

/// Immediately triggers the configured screen locker
pub fn lock_session() {
    let user_cfg = UserConfig::load();
    let lock_cmd = user_cfg.apps.lock_cmd;
    info!(cmd = %lock_cmd, "Engaging desktop screen locker");
    spawn_shell(&lock_cmd);
}

/// Supervises systemd-logind sleep signals and manages automatic session locking
pub async fn supervise_sleep_signals() {
    info!("Starting systemd-logind sleep and session lock supervisor...");

    // Check if busctl is available for system D-Bus monitoring
    if which::which("busctl").is_err() {
        warn!("busctl not found; system sleep-lock integration will be passive.");
        return;
    }

    tokio::task::spawn_blocking(|| {
        let mut child = match Command::new("busctl")
            .args(["--system", "monitor", "org.freedesktop.login1"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                error!(error = %e, "Failed to start D-Bus monitor for systemd-logind");
                return;
            }
        };

        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => return,
        };

        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            // Check for PrepareForSleep signal
            if line.contains("PrepareForSleep") {
                debug!(raw = %line, "Received logind sleep signal");
                let is_preparing_for_sleep = line.contains("true") || line.contains("b 1");
                let is_resuming = line.contains("false") || line.contains("b 0");

                if is_preparing_for_sleep {
                    info!("System is preparing for sleep/suspend -> triggering lock and DPMS blanking");
                    lock_session();
                    // Optional DPMS standby
                    spawn_shell("nightctl dpms off 2>/dev/null || true");
                } else if is_resuming {
                    info!("System has resumed from sleep/suspend -> waking displays and updating state");
                    spawn_shell("nightctl dpms on 2>/dev/null || true");
                }
            }
        }
    });

    info!("Sleep and session lock supervisor active.");
}

