//! 🌙 night-session: Pure-Rust Native Wayland Desktop Environment Launcher
//! Replaces legacy shell scripts with a compiled, memory-safe ELF binary.
//! Manages display manager session initialization, systemd user integration, and teardown.

use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn log(msg: &str) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let line = format!("[night-session:{now}] {msg}\n");
    print!("{line}");

    // Log to ~/.local/state/night/logs/session.log and /tmp/night-session.log for foolproof diagnostics
    let state_log = dirs::state_dir()
        .map(|d| d.join("night/logs/session.log"))
        .unwrap_or_else(|| PathBuf::from("/tmp/night-session.log"));
    if let Some(parent) = state_log.parent() {
        let _ = fs::create_dir_all(parent);
    }
    for target in [&state_log, &PathBuf::from("/tmp/night-session.log")] {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(target)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

fn main() {
    log("================================================================================");
    log("🌙 Starting Project Night Desktop Environment (Pure-Rust Session Launcher)...");

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/irshad"));
    let user = env::var("USER").unwrap_or_else(|_| "irshad".into());
    log(&format!("Running as User: {user} | HOME: {}", home.display()));

    // 1. Ensure user and system PATH are comprehensively available
    let cargo_bin = home.join(".cargo/bin");
    let current_path = env::var("PATH").unwrap_or_default();
    let extended_path = format!("{}:/usr/local/bin:/usr/bin:/bin:{current_path}", cargo_bin.display());
    env::set_var("PATH", &extended_path);

    // 2. Ensure critical state and cache directories exist before starting
    let dirs_to_create = [
        home.join(".local/state/pinnacle"),
        home.join(".local/share/night"),
        home.join(".config/night"),
        home.join(".config/pinnacle"),
        home.join(".config/systemd/user"),
        home.join(".config/xdg-desktop-portal"),
    ];

    for dir in &dirs_to_create {
        if let Err(e) = fs::create_dir_all(dir) {
            log(&format!("⚠️ Notice creating {}: {e}", dir.display()));
        }
    }

    // 3. Clear out any leftover greeter display sockets so Pinnacle uses Udev/DRM hardware backend
    env::remove_var("WAYLAND_DISPLAY");
    env::remove_var("WAYLAND_SOCKET");
    env::remove_var("DISPLAY");

    // 4. Set desktop identifiers & Wayland native toolkit backends
    env::set_var("XDG_CURRENT_DESKTOP", "pinnacle:night");
    env::set_var("XDG_SESSION_TYPE", "wayland");
    env::set_var("XDG_SESSION_DESKTOP", "night");

    // Pure Wayland toolkit overrides
    env::set_var("MOZ_ENABLE_WAYLAND", "1");
    env::set_var("QT_QPA_PLATFORM", "wayland");
    env::set_var("GDK_BACKEND", "wayland,x11,*");
    env::set_var("CLUTTER_BACKEND", "wayland");
    env::set_var("SDL_VIDEODRIVER", "wayland");
    env::set_var("_JAVA_AWT_WM_NONREPARENTING", "1");

    log("Environment variables configured for native Wayland execution.");

    // 5. Systemd User Session Integration (Arch Linux / greetd standard)
    let has_systemctl = which::which("systemctl").is_ok();
    if has_systemctl {
        log("systemd detected; synchronizing user environment...");

        // Reset failed state of user units
        let _ = Command::new("systemctl")
            .args(["--user", "reset-failed"])
            .output();

        // Import environment into systemd user manager
        let import_vars = [
            "PATH",
            "HOME",
            "USER",
            "XDG_CURRENT_DESKTOP",
            "XDG_SESSION_TYPE",
            "XDG_SESSION_DESKTOP",
            "MOZ_ENABLE_WAYLAND",
            "QT_QPA_PLATFORM",
            "GDK_BACKEND",
            "CLUTTER_BACKEND",
            "SDL_VIDEODRIVER",
            "_JAVA_AWT_WM_NONREPARENTING",
        ];

        let _ = Command::new("systemctl")
            .arg("--user")
            .arg("import-environment")
            .args(import_vars)
            .output();

        // Update DBus activation environment
        if which::which("dbus-update-activation-environment").is_ok() {
            let _ = Command::new("dbus-update-activation-environment")
                .arg("--all")
                .output();
        }

        // Check if night.service or pinnacle.service exists
        let user_service_dir = home.join(".config/systemd/user");
        let has_night_unit = user_service_dir.join("night.service").exists()
            || PathBuf::from("/usr/lib/systemd/user/night.service").exists();
        let has_pinnacle_unit = user_service_dir.join("pinnacle.service").exists()
            || PathBuf::from("/usr/lib/systemd/user/pinnacle.service").exists();

        let target_unit = if has_night_unit {
            Some("night.service")
        } else if has_pinnacle_unit {
            Some("pinnacle.service")
        } else {
            None
        };

        if let Some(unit) = target_unit {
            log(&format!("Starting {unit} via systemctl --user --wait..."));
            let status = Command::new("systemctl")
                .args(["--user", "--wait", "start", unit])
                .status();

            log(&format!("Compositor unit {unit} exited with: {status:?}"));

            // Teardown graphical-session
            log("Tearing down graphical user session...");
            let _ = Command::new("systemctl")
                .args(["--user", "start", "--job-mode=replace-irreversibly", "night-shutdown.target"])
                .output();

            let _ = Command::new("systemctl")
                .args([
                    "--user",
                    "unset-environment",
                    "WAYLAND_DISPLAY",
                    "DISPLAY",
                    "XDG_SESSION_TYPE",
                    "XDG_CURRENT_DESKTOP",
                    "PINNACLE_GRPC_SOCKET",
                ])
                .output();

            log("🌙 Project Night Desktop Session Finished Cleanly.");
            return;
        }
    }

    // 6. Direct fallback if systemd user services are not present
    log("Launching Pinnacle compositor directly with --session...");
    let pinnacle_bin = which::which("pinnacle")
        .unwrap_or_else(|_| cargo_bin.join("pinnacle"));

    let mut child = Command::new(&pinnacle_bin)
        .arg("--session")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| {
            log(&format!("❌ Fatal: Failed to spawn {}: {e}", pinnacle_bin.display()));
            exit(1);
        });

    let exit_status = child.wait().unwrap_or_else(|e| {
        log(&format!("❌ Error waiting for pinnacle: {e}"));
        exit(1);
    });

    log(&format!("🌙 Pinnacle exited with code: {exit_status:?}"));
}

