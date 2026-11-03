//! 🌙 night-session: Pure-Rust Native Wayland Desktop Environment Session Manager
//! Replaces legacy shell scripts with a compiled, memory-safe ELF binary.
//! Manages display manager session initialization, systemd user integration,
//! screen locking, graceful teardown, and signal handling.

use clap::{Parser, Subcommand};
use night_core::config::UserConfig;
use night_core::exec::{ensure_desktop_env, resolve_executable, spawn_shell};
use night_core::logging::init_logger;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "night-session",
    author = "Night Desktop Team",
    version,
    about = "🌙 Pure-Rust Native Wayland Desktop Environment Session Manager",
    long_about = "Manages the lifecycle of Project Night under display managers (greetd, Ly, GDM, SDDM) or TTY.\nHandles environment propagation, systemd user session coordination, screen locking, and teardown."
)]
struct Cli {
    #[command(subcommand)]
    action: Option<SessionSubcommand>,
}

#[derive(Subcommand, Debug)]
enum SessionSubcommand {
    /// Launch the Wayland desktop environment session (default)
    Run,
    /// Immediately lock the desktop session using the configured screen locker (Veila)
    Lock,
    /// Gracefully terminate and tear down the desktop session
    #[command(alias = "exit")]
    Logout,
    /// Query the status of the active desktop session and compositor
    Status,
}

#[tokio::main]
async fn main() {
    let _log_guard = init_logger("session");
    ensure_desktop_env();

    let cli = Cli::parse();
    match cli.action.unwrap_or(SessionSubcommand::Run) {
        SessionSubcommand::Run => run_session().await,
        SessionSubcommand::Lock => lock_session(),
        SessionSubcommand::Logout => logout_session(),
        SessionSubcommand::Status => print_session_status(),
    }
}

/// Immediately triggers the configured screen locker
fn lock_session() {
    let cfg = UserConfig::load();
    let lock_cmd = cfg.apps.lock_cmd;
    info!(cmd = %lock_cmd, "night-session: Locking active session");
    spawn_shell(&lock_cmd);
}

/// Gracefully terminates and tears down the active desktop session
fn logout_session() {
    info!("night-session: Initiating graceful desktop session logout");
    teardown_session();

    // If nightctl is available, send quit command to compositor
    let _ = Command::new("nightctl")
        .args(["msg", "close"])
        .status();

    exit(0);
}

/// Displays live session diagnostics
fn print_session_status() {
    println!("🌙 Project Night Session Status");
    println!("  User                 : {}", env::var("USER").unwrap_or_else(|_| "unknown".into()));
    println!("  Home Directory       : {}", dirs::home_dir().map(|h| h.display().to_string()).unwrap_or_default());
    println!("  XDG_CURRENT_DESKTOP  : {}", env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "unset".into()));
    println!("  XDG_SESSION_TYPE     : {}", env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "unset".into()));
    println!("  WAYLAND_DISPLAY      : {}", env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "none".into()));

    let socket = dirs::runtime_dir().map(|r| r.join("night.sock"));
    if let Some(ref s) = socket {
        println!("  IPC Socket Path      : {} ({})", s.display(), if s.exists() { "ACTIVE" } else { "OFFLINE" });
    }

    if let Ok(output) = Command::new("systemctl").args(["--user", "is-active", "night.service"]).output() {
        let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
        println!("  systemd night.service: {}", if status.is_empty() { "inactive/not installed" } else { &status });
    }
}

/// Performs complete graphical session teardown and unsets environment variables
fn teardown_session() {
    info!("Tearing down graphical user session...");

    if which::which("systemctl").is_ok() {
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
    }

    info!("🌙 Project Night Desktop Session Finished Cleanly.");
}

/// Launches and supervises the full Wayland desktop session
async fn run_session() {
    info!("================================================================================");
    info!("🌙 Starting Project Night Desktop Environment (Pure-Rust Session Manager)...");

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let user = env::var("USER").unwrap_or_else(|_| "user".into());
    info!(user = %user, home = %home.display(), "Initializing user session");

    // 1. Ensure user and system PATH are comprehensively available
    ensure_desktop_env();

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
            warn!(path = %dir.display(), error = %e, "Could not create state directory");
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

    info!("Environment variables configured for native Wayland execution.");

    // 5. Systemd User Session Integration (Arch Linux / greetd standard)
    let has_systemctl = which::which("systemctl").is_ok();
    if has_systemctl {
        info!("systemd detected; synchronizing user environment...");

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
            info!(unit = %unit, "Starting compositor service via systemctl --user --wait");

            let mut sys_child = Command::new("systemctl")
                .args(["--user", "--wait", "start", unit])
                .spawn()
                .unwrap_or_else(|e| {
                    error!(error = %e, "Failed to spawn systemctl wait");
                    exit(1);
                });

            // Signal handler for SIGINT (Ctrl+C)
            tokio::select! {
                res = tokio::task::spawn_blocking(move || sys_child.wait()) => {
                    info!("Compositor systemd service exited with status: {res:?}");
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("Received interrupt signal (SIGINT); stopping session...");
                }
            }

            teardown_session();
            return;
        }
    }

    // 6. Direct fallback if systemd user services are not present
    info!("Launching Pinnacle compositor directly with --session...");
    let pinnacle_bin = resolve_executable("pinnacle")
        .unwrap_or_else(|| PathBuf::from("/usr/local/bin/pinnacle"));

    let mut child = Command::new(&pinnacle_bin)
        .arg("--session")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap_or_else(|e| {
            error!(path = %pinnacle_bin.display(), error = %e, "Fatal: Failed to spawn Pinnacle compositor");
            exit(1);
        });

    tokio::select! {
        res = tokio::task::spawn_blocking(move || child.wait()) => {
            info!("Pinnacle compositor process exited with: {res:?}");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received interrupt signal (SIGINT); terminating compositor...");
        }
    }

    teardown_session();
}
