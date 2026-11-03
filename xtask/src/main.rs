//! 🌙 Project Night - xtask Automation Runner
//! Native Rust workspace automation replacing Makefiles and shell scripts.
//! Uses standard `dirs` crate for cross-platform XDG resolution.
//! Usage: `cargo xtask <command>`

use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

#[tokio::main]
async fn main() {
    let mut args = env::args().skip(1);
    let task = args.next().unwrap_or_else(|| "help".to_string());

    let root_dir = project_root();

    match task.as_str() {
        "install" => task_install(&root_dir),
        "check" => task_check(&root_dir),
        "build" => task_build(&root_dir),
        "run" => task_run(&root_dir),
        "uninstall" => task_uninstall(&root_dir),
        "theme" => task_theme(&root_dir),
        "doctor" => task_doctor(),
        "session" => task_session(&root_dir),
        "msg" | "ctl" => task_msg(args.collect()).await,
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("Unknown task: '{other}'. Run `cargo xtask help` for usage.");
            exit(1);
        }
    }
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .expect("Failed to locate project root")
        .to_path_buf()
}

fn task_check(root: &Path) {
    println!("🔍 Running `cargo check` across entire workspace...");
    let status = Command::new("cargo")
        .arg("check")
        .arg("--workspace")
        .current_dir(root)
        .status()
        .expect("Failed to run cargo check");

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
    println!("✅ Workspace check passed.");
}

fn task_build(root: &Path) {
    println!("🔨 Building `night-pinnacle` and `nightctl` in release mode...");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "night-pinnacle", "-p", "nightctl"])
        .current_dir(root)
        .status()
        .expect("Failed to run cargo build");

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
    println!("✅ Build complete: target/release/night-pinnacle and target/release/nightctl");
}

fn task_install(root: &Path) {
    println!("🌙 Installing Project Night...");

    // 1. Compile in release mode
    task_build(root);

    let home = dirs::home_dir().expect("Could not determine user home directory");
    let cargo_bin = home.join(".cargo/bin");
    let _ = fs::create_dir_all(&cargo_bin);

    // 2. Install binaries to ~/.cargo/bin
    for bin_name in ["night-pinnacle", "nightctl"] {
        let source_bin = root.join("target/release").join(bin_name);
        let target_bin = cargo_bin.join(bin_name);
        println!("==> Copying binary to {}", target_bin.display());
        if let Err(e) = fs::copy(&source_bin, &target_bin) {
            println!("⚠️ Notice: Could not copy binary to {}: {e}", target_bin.display());
        }
    }

    // 3. Link pinnacle.toml to ~/.config/pinnacle/pinnacle.toml using `dirs::config_dir()`
    let config_dir = dirs::config_dir()
        .expect("Could not determine config directory")
        .join("pinnacle");
    let _ = fs::create_dir_all(&config_dir);

    let source_toml = root.join("pinnacle.toml");
    let target_toml = config_dir.join("pinnacle.toml");

    if target_toml.exists() || fs::symlink_metadata(&target_toml).is_ok() {
        let _ = fs::remove_file(&target_toml);
    }
    println!("==> Linking {} -> {}", source_toml.display(), target_toml.display());
    if let Err(e) = symlink(&source_toml, &target_toml) {
        if let Err(copy_err) = fs::copy(&source_toml, &target_toml) {
            println!("⚠️ Notice: Could not link or copy pinnacle.toml to {}: {e} / {copy_err}", target_toml.display());
        } else {
            println!("==> Copied pinnacle.toml to {}", target_toml.display());
        }
    }

    // 4. Setup default user configurations (~/.config/night/)
    let night_user_dir = dirs::config_dir()
        .expect("Could not determine config directory")
        .join("night");
    let _ = fs::create_dir_all(&night_user_dir);

    let default_config_dir = root.join("config");
    if default_config_dir.exists() {
        for file in ["night.toml", "rules.ron"] {
            let src = default_config_dir.join(file);
            let dst = night_user_dir.join(file);
            if src.exists() && !dst.exists() {
                println!("==> Initializing default config: {}", dst.display());
                let _ = fs::copy(&src, &dst);
            }
        }
    }

    // 5. Deploy Tokyo Night styling configs for desktop tools
    task_theme(root);

    if let Some(cfg) = dirs::config_dir() {
        let tool_deployments = [
            ("config/alacritty.toml", cfg.join("alacritty/alacritty.toml")),
            ("config/ironbar.toml", cfg.join("ironbar/config.toml")),
            ("config/ironbar.css", cfg.join("ironbar/style.css")),
            ("config/anyrun.css", cfg.join("anyrun/style.css")),
            ("config/veila.toml", cfg.join("veila/config.toml")),
        ];

        for (src_rel, target_path) in tool_deployments {
            let src = root.join(src_rel);
            if src.exists() && !target_path.exists() {
                if let Some(parent) = target_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                println!("==> Deploying Tokyo Night theme: {}", target_path.display());
                let _ = fs::copy(&src, &target_path);
            }
        }
    }

    // 6. Install Wayland session desktop entry
    task_session(root);

    println!("🎉 Night successfully installed! Launch your compositor by typing: pinnacle (or select Night in your display manager)");
}

fn task_theme(root: &Path) {
    println!("🎨 Generating Tokyo Night styling configurations for desktop tools...");
    let config_dir = root.join("config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    // 1. Alacritty
    let alacritty_cfg = r##"# 🌙 Tokyo Night theme for Alacritty generated by Night Desktop
[colors.primary]
background = "#1a1b26"
foreground = "#c0caf5"

[colors.normal]
black = "#15161e"
red = "#f7768e"
green = "#9ece6a"
yellow = "#e0af68"
blue = "#7aa2f7"
magenta = "#bb9af7"
cyan = "#7dcfff"
white = "#a9b1d6"

[colors.bright]
black = "#414868"
red = "#f7768e"
green = "#9ece6a"
yellow = "#e0af68"
blue = "#7aa2f7"
magenta = "#bb9af7"
cyan = "#7dcfff"
white = "#c0caf5"
"##;
    fs::write(config_dir.join("alacritty.toml"), alacritty_cfg).expect("Failed to write alacritty.toml");

    // 2. Ironbar
    let ironbar_css = r##"/* 🌙 Tokyo Night GTK4 stylesheet for Ironbar generated by Night Desktop */
* {
    font-family: "JetBrainsMono Nerd Font", monospace;
    font-size: 13px;
    border: none;
    border-radius: 0;
}

window#ironbar {
    background-color: rgba(26, 27, 38, 0.95);
    color: #c0caf5;
    border-bottom: 2px solid #24283b;
}

.widget {
    background-color: #24283b;
    color: #c0caf5;
    border-radius: 8px;
    margin: 4px 3px;
    padding: 2px 10px;
}

.widget:hover {
    background-color: #414868;
}

.workspaces .item.focused {
    background-color: #7aa2f7;
    color: #15161e;
    font-weight: bold;
}
"##;
    fs::write(config_dir.join("ironbar.css"), ironbar_css).expect("Failed to write ironbar.css");

    // 2b. Ironbar TOML Configuration
    let ironbar_toml = r##"# 🌙 Tokyo Night status bar configuration for Ironbar (GTK4-layer-shell)
position = "top"
height = 34
icon_theme = "Papirus-Dark"

[start]
workspaces = { type = "workspaces", all_monitors = false }
layout = { type = "custom", bar = ["label"], exec = "nightctl stream --format '󰕰 {layout}'", on_click_left = "nightctl layout next" }
focused = { type = "focused", show_icon = true, max_length = 40 }

[center]
clock = { type = "clock", format = "%a %d %b  %H:%M" }

[end]
sys_info = { type = "sys_info", interval = 3 }
volume = { type = "volume", format = "{icon} {percentage}%", max_volume = 100 }
tray = { type = "tray", icon_size = 16 }
"##;
    fs::write(config_dir.join("ironbar.toml"), ironbar_toml).expect("Failed to write ironbar.toml");

    // 3. Anyrun
    let anyrun_css = r##"/* 🌙 Tokyo Night GTK4 stylesheet for Anyrun generated by Night Desktop */
#window {
    background-color: transparent;
}

#main {
    background-color: rgba(26, 27, 38, 0.96);
    border: 2px solid #7aa2f7;
    border-radius: 12px;
    padding: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
}

#entry {
    background-color: #24283b;
    color: #c0caf5;
    border-radius: 8px;
    padding: 8px 12px;
    font-size: 16px;
}

#match.selected {
    background-color: #414868;
    color: #7dcfff;
    border-radius: 6px;
}
"##;
    fs::write(config_dir.join("anyrun.css"), anyrun_css).expect("Failed to write anyrun.css");

    // 4. Veila
    let veila_cfg = r##"# 🌙 Tokyo Night configuration for Veila (Pure-Rust Wayland Screen Locker)
background_color = "#1a1b26"
text_color = "#c0caf5"
accent_color = "#7aa2f7"
fail_color = "#f7768e"
font = "JetBrainsMono Nerd Font"
"##;
    fs::write(config_dir.join("veila.toml"), veila_cfg).expect("Failed to write veila.toml");

    println!("✅ Tokyo Night suite configs exported to ./config/");
}

fn task_run(root: &Path) {
    task_build(root);
    println!("🚀 Launching Pinnacle...");
    let _ = Command::new("pinnacle").current_dir(root).status();
}

fn task_uninstall(_root: &Path) {
    println!("🗑️ Uninstalling Project Night...");
    let home = dirs::home_dir().expect("Could not determine user home directory");

    let cargo_bin = home.join(".cargo/bin");
    for bin_name in ["night-pinnacle", "nightctl"] {
        let bin = cargo_bin.join(bin_name);
        if bin.exists() {
            let _ = fs::remove_file(&bin);
            println!("Removed {}", bin.display());
        }
    }

    if let Some(cfg) = dirs::config_dir() {
        let toml = cfg.join("pinnacle/pinnacle.toml");
        if toml.exists() {
            let _ = fs::remove_file(&toml);
            println!("Removed {}", toml.display());
        }
    }

    println!("✅ Night uninstalled.");
}

async fn task_msg(args: Vec<String>) {
    if args.is_empty() {
        eprintln!("Usage: cargo xtask msg <reload|switch-tag <TAG>|state|spawn <CMD>|close>");
        exit(1);
    }

    let cmd = match args[0].as_str() {
        "reload" => night_core::ipc::IpcCommand::ReloadConfig,
        "switch-tag" => {
            if args.len() < 2 {
                eprintln!("Error: 'switch-tag' requires a tag name (e.g. `cargo xtask msg switch-tag 2`)");
                exit(1);
            }
            night_core::ipc::IpcCommand::SwitchTag(args[1].clone())
        }
        "state" => night_core::ipc::IpcCommand::QueryState,
        "spawn" => {
            if args.len() < 2 {
                eprintln!("Error: 'spawn' requires a command string (e.g. `cargo xtask msg spawn alacritty`)");
                exit(1);
            }
            night_core::ipc::IpcCommand::Spawn(args[1..].join(" "))
        }
        "close" => night_core::ipc::IpcCommand::CloseFocused,
        "status" => night_core::ipc::IpcCommand::GetStatus,
        "scratchpad" => night_core::ipc::IpcCommand::ToggleScratchpad,
        "layout" => {
            if args.len() > 1 && args[1] == "next" {
                night_core::ipc::IpcCommand::CycleLayout
            } else {
                night_core::ipc::IpcCommand::GetLayout
            }
        }
        "gaps" => {
            if args.len() < 2 {
                eprintln!("Error: 'gaps' requires a numeric delta (e.g. `cargo xtask msg gaps 2.0`)");
                exit(1);
            }
            let delta: f32 = args[1].parse().unwrap_or(2.0);
            night_core::ipc::IpcCommand::AdjustGaps(delta)
        }
        "overlay" | "binds" => night_core::ipc::IpcCommand::ShowBinds,
        "quit-prompt" => night_core::ipc::IpcCommand::ShowQuit,
        "stream" => {
            println!("📡 Subscribing to real-time Night status stream...");
            let res = night_core::ipc::subscribe_status(|s| {
                println!("[{}] Tag {} | Window: {}", s.layout, s.active_tag, s.active_window.as_deref().unwrap_or("None"));
            }).await;
            if let Err(e) = res {
                eprintln!("❌ Stream disconnected: {e}");
            }
            return;
        }
        other => {
            eprintln!("Unknown IPC command: '{other}'");
            exit(1);
        }
    };

    println!("📡 Sending IPC command to Night: {cmd:?}");
    match night_core::ipc::send_command(&cmd).await {
        Ok(night_core::ipc::IpcResponse::Status(s)) => {
            println!("🌙 Night Desktop Environment Status:");
            println!("  Active Workspace Tag : {}", s.active_tag);
            println!("  Tiling Layout        : {}", s.layout);
            println!("  Gaps (Inner / Outer) : {}px / {}px", s.inner_gaps, s.outer_gaps);
            println!("  Connected Monitors   : {}", s.output_count);
            println!("  Focused Window       : {}", s.active_window.unwrap_or_else(|| "None".into()));
            println!("  Last Reload Timestamp: {}", s.last_reload);
        }
        Ok(resp) => println!("✅ Response: {resp:?}"),
        Err(e) => {
            eprintln!("❌ Failed to deliver command: {e}");
            exit(1);
        }
    }
}

fn task_doctor() {
    println!("🩺 Running Project Night Environment & Suite Doctor...\n");

    let required_tools = [
        ("Pinnacle Compositor", "pinnacle", "pinnacle-comp-git"),
        ("Alacritty Terminal", "alacritty", "alacritty"),
        ("Rio Terminal", "rio", "rio"),
        ("Anyrun Launcher", "anyrun", "anyrun-git"),
        ("Yofi Launcher", "yofi", "yofi"),
        ("Favnyr GUI File Manager", "favnyr", "favnyr-git"),
        ("Yazi TUI File Manager", "yazi", "yazi"),
        ("Zen Browser", "zen-browser", "zen-browser-bin"),
        ("Brave Browser", "brave", "brave-bin"),
        ("Ironbar Status Bar", "ironbar", "ironbar-bin"),
        ("Wired Notification Daemon", "wired", "wired-notify-bin"),
        ("SwayOSD Overlay Server", "swayosd-server", "swayosd-git"),
        ("swww Wallpaper Daemon", "swww-daemon", "swww"),
        ("Wayshot Screenshot CLI", "wayshot", "wayshot"),
        ("Satty Annotation Tool", "satty", "satty-bin"),
        ("wl-screenrec Recorder", "wl-screenrec", "wl-screenrec"),
        ("wayidle Idle Daemon", "wayidle", "wayidle"),
        ("Veila Screen Locker", "veila", "veila-bin"),
    ];

    let mut missing = Vec::new();
    println!("📦 External 100% Rust-Native Desktop Suite:");
    for (name, bin, pkg) in required_tools {
        let found = if bin == "zen-browser" {
            which::which("zen-browser").is_ok() || which::which("zen").is_ok()
        } else {
            which::which(bin).is_ok()
        };

        if found {
            println!("  ✅ {:<28} (found `{}`)", name, bin);
        } else {
            println!("  ❌ {:<28} (missing `{}` -> package: {})", name, bin, pkg);
            missing.push(pkg);
        }
    }

    println!("\n📁 Configuration Files:");
    if let Some(cfg) = dirs::config_dir() {
        let configs = [
            ("Night Settings (TOML)", cfg.join("night/night.toml")),
            ("Window Rules (RON)", cfg.join("night/rules.ron")),
            ("Pinnacle Manifest", cfg.join("pinnacle/pinnacle.toml")),
        ];

        for (desc, path) in configs {
            if path.exists() {
                println!("  ✅ {:<24} ({})", desc, path.display());
            } else {
                println!("  ⚠️ {:<24} (missing at {})", desc, path.display());
            }
        }
    }

    println!("\n🌐 Wayland Session Environment:");
    for var in ["WAYLAND_DISPLAY", "XDG_CURRENT_DESKTOP", "XDG_SESSION_TYPE"] {
        match std::env::var(var) {
            Ok(val) => println!("  ✅ {:<22} = {}", var, val),
            Err(_) => println!("  ℹ️ {:<22} (not set in current subshell)", var),
        }
    }

    if !missing.is_empty() {
        println!("\n💡 To install missing desktop suite components on Arch Linux:");
        println!("   yay -S --needed {}", missing.join(" "));
    } else {
        println!("\n🎉 All 100% Rust-native desktop components are installed and ready!");
    }
}

fn task_session(root: &Path) {
    println!("📦 Installing Wayland desktop session entry...");
    let home = dirs::home_dir().expect("Failed to locate home directory");

    // 1. Install night-session and desktop scripts to ~/.cargo/bin
    let bin_dir = home.join(".cargo/bin");
    let _ = fs::create_dir_all(&bin_dir);
    for script_name in ["night-session", "wallpaper.sh", "screenshot.sh", "record.sh"] {
        let script_src = root.join("assets").join(script_name);
        let script_dst = bin_dir.join(script_name);
        if script_src.exists() {
            if let Ok(_) = fs::copy(&script_src, &script_dst) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(meta) = fs::metadata(&script_dst) {
                        let mut perms = meta.permissions();
                        perms.set_mode(0o755);
                        let _ = fs::set_permissions(&script_dst, perms);
                    }
                }
                println!("==> Installed launcher script to {}", script_dst.display());
            }
        }
    }

    // 2. Install night.desktop to ~/.local/share/wayland-sessions/night.desktop
    let wayland_sessions_dir = home.join(".local/share/wayland-sessions");
    if let Err(e) = fs::create_dir_all(&wayland_sessions_dir) {
        println!("ℹ️ Note: Could not write to {} ({}).", wayland_sessions_dir.display(), e);
        println!("   To install for your display manager, copy assets/night.desktop to /usr/share/wayland-sessions/");
    } else {
        let desktop_src = root.join("assets/night.desktop");
        let desktop_dst = wayland_sessions_dir.join("night.desktop");
        if desktop_src.exists() {
            fs::copy(&desktop_src, &desktop_dst).expect("Failed to install night.desktop");
            println!("==> Installed Wayland session entry to {}", desktop_dst.display());
            println!("✨ Night is now selectable from your login display manager (GDM, SDDM, Ly, greetd)!");
        }
    }
}

fn print_help() {
    println!(
        r#"🌙 Night Workspace Task Runner (cargo xtask)

USAGE:
    cargo xtask <SUBCOMMAND>

SUBCOMMANDS:
    install      Builds release binaries (night-pinnacle, nightctl), copies to ~/.cargo/bin, links configs
    doctor       Performs system diagnostics on all 100% Rust-native components and environment
    session      Installs Wayland session desktop entry for login managers (GDM, SDDM, Ly, greetd)
    theme        Generates Tokyo Night configurations for external suite (Alacritty, Ironbar, Anyrun, Veila)
    check        Runs `cargo check` across all workspace crates
    build        Builds `night-pinnacle` and `nightctl` in release mode
    run          Builds and launches the Pinnacle compositor
    msg <CMD>    Sends IPC command to running Night (status, stream, tag, layout, gaps, scratchpad, overlay, quit-prompt)
    uninstall    Removes installed binaries and config symlinks
    help         Prints this help message
"#
    );
}
