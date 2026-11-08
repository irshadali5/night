//! 🌙 Project Night - xtask Automation Runner
//! Native Rust workspace automation replacing Makefiles and shell scripts.
//! Built with `clap` for typed, declarative CLI argument parsing.
//! Usage: `cargo xtask <command>`

use clap::{Parser, Subcommand, ValueEnum};
use night_core::theme::ThemeVariant;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};

#[derive(Parser, Debug)]
#[command(
    name = "cargo xtask",
    author = "Night Desktop Team",
    version,
    about = "🌙 Night Workspace Task Runner (cargo xtask)",
    long_about = "Pure-Rust task runner managing builds, installations, Bihar Night theming, environment diagnostics, and IPC messages."
)]
struct XtaskCli {
    #[command(subcommand)]
    command: XtaskCommand,
}

#[derive(Subcommand, Debug)]
enum XtaskCommand {
    /// Builds release binaries (night-pinnacle, nightctl), copies to ~/.cargo/bin, links configs
    Install,
    /// Runs `cargo check` across all workspace crates
    Check,
    /// Builds `night-pinnacle` and `nightctl` in release mode
    Build,
    /// Builds and launches the Pinnacle compositor
    Run,
    /// Removes installed binaries and config symlinks
    Uninstall,
    /// Generates Bihar Night configurations for external suite (Alacritty, Ironbar, Anyrun, Veila)
    Theme {
        /// Optional theme variant (night, storm, moon, day)
        #[arg(value_enum)]
        variant: Option<XtaskThemeVariant>,
    },
    /// Performs system diagnostics on all 100% Rust-native components and environment
    Doctor,
    /// Installs Wayland session desktop entry for login managers (GDM, SDDM, Ly, greetd)
    Session,
    /// Configures 100% Rust-native TUI Display Manager (greetd + tuigreet)
    #[command(alias = "setup-dm")]
    Dm,
    /// Sets desktop wallpaper or selects random Bihar Night image
    Wallpaper {
        /// Optional path to wallpaper image
        path: Option<PathBuf>,
    },
    /// Takes interactive or fullscreen screenshot (wayshot + slurp + satty)
    Screenshot {
        /// Mode: "area" (default), "full", or "window"
        #[arg(default_value = "area")]
        mode: String,
    },
    /// Toggles hardware-accelerated screen recording (wl-screenrec)
    Record,
    /// Views or tails recent desktop logs (~/.local/state/night/logs/)
    #[command(alias = "log")]
    Logs {
        /// Target service: night, settings, ctl, session (default: "night")
        #[arg(default_value = "night")]
        service: String,
        /// Number of recent lines to display
        #[arg(short = 'n', long, default_value_t = 50)]
        lines: usize,
    },
    /// Sends an IPC command to running Night compositor
    #[command(alias = "ctl")]
    Msg {
        #[command(subcommand)]
        command: MsgCommand,
    },
}

#[derive(Subcommand, Debug)]
enum MsgCommand {
    /// Reloads the compositor configuration
    Reload,
    /// Switches to the specified tag name
    #[command(alias = "switch-tag")]
    Tag {
        /// Target tag name (e.g. 1..9)
        name: String,
    },
    /// Queries the current session state
    State,
    /// Spawns a command through the compositor
    Spawn {
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },
    /// Closes the focused window
    Close,
    /// Queries full desktop status
    Status,
    /// Toggles the interactive Quake-style dropdown scratchpad terminal
    Scratchpad,
    /// Queries or cycles the dynamic tiling layout algorithm
    Layout {
        /// Optional action: "next" to cycle to next layout
        action: Option<String>,
    },
    /// Adjusts layout gaps by a delta value in pixels
    Gaps { delta: f32 },
    /// Triggers the Snowcap keybindings overlay
    #[command(alias = "binds")]
    Overlay,
    /// Triggers the Snowcap quit confirmation prompt
    #[command(alias = "prompt-quit")]
    QuitPrompt,
    /// Triggers the Snowcap Bihar Night interactive visual window switcher
    #[command(alias = "switch-window")]
    Switcher,
    /// Toggles or sets night light color temperature
    Gamma {
        /// Optional temperature in Kelvin (e.g. 3800, 6500)
        temp: Option<u32>,
    },
    /// Toggles idle inhibition (presentation mode)
    Inhibit,
    /// Triggers the Snowcap Bihar Night clipboard history modal
    #[command(alias = "clipboard")]
    Clip,
    /// Controls MPRIS media player playback
    Media,
    /// Triggers the Snowcap Bihar Night interactive snipping modal
    Snip,
    /// Sets or queries Bihar Night theme variant
    Theme {
        /// Optional variant to set (night, storm, moon, day)
        variant: Option<String>,
    },
    /// Manages audio volume (raise, lower, mute, status)
    Volume {
        /// Volume action: "raise", "lower", "mute", or omit to query
        action: Option<String>,
    },
    /// Queries battery and power supply status
    Power,
    /// Triggers the Snowcap Bihar Night application launcher modal
    #[command(alias = "run")]
    Launcher,
    /// Queries network connectivity status
    Net,
    /// Queries real-time hardware performance metrics (CPU, RAM, Temp, Disk)
    Top,
    /// Queries Bluetooth controller and connected wireless peripherals
    Bt,
    /// Manages audio output sinks (headphones, speakers, HDMI, USB DAC)
    Sink {
        /// Optional action: "list", "cycle", "menu", or ID number
        action: Option<String>,
    },
    /// Manages display brightness ("raise", "lower", or percentage)
    Brightness {
        action: Option<String>,
    },
    /// Toggles Do Not Disturb (DND) mode
    Dnd,
    /// Triggers Snowcap notification center modal
    Notify,
    /// Saves or restores desktop window session layout ("save", "restore")
    Session {
        action: String,
    },
    /// Sets or queries desktop wallpaper via swww
    Wallpaper {
        /// Optional path to wallpaper file
        path: Option<String>,
    },
    /// Subscribes to real-time status updates stream
    Stream,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum XtaskThemeVariant {
    Night,
    Storm,
    Moon,
    Day,
}

impl From<XtaskThemeVariant> for ThemeVariant {
    fn from(v: XtaskThemeVariant) -> Self {
        match v {
            XtaskThemeVariant::Night => ThemeVariant::Night,
            XtaskThemeVariant::Storm => ThemeVariant::Storm,
            XtaskThemeVariant::Moon => ThemeVariant::Moon,
            XtaskThemeVariant::Day => ThemeVariant::Day,
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = XtaskCli::parse();
    let root_dir = project_root();

    match cli.command {
        XtaskCommand::Install => task_install(&root_dir),
        XtaskCommand::Check => task_check(&root_dir),
        XtaskCommand::Build => task_build(&root_dir),
        XtaskCommand::Run => task_run(&root_dir),
        XtaskCommand::Uninstall => task_uninstall(&root_dir),
        XtaskCommand::Theme { variant } => task_theme(&root_dir, variant.map(Into::into).unwrap_or(ThemeVariant::Night)),
        XtaskCommand::Doctor => task_doctor(),
        XtaskCommand::Session => task_session(&root_dir),
        XtaskCommand::Dm => task_dm(&root_dir),
        XtaskCommand::Wallpaper { path } => task_wallpaper(path),
        XtaskCommand::Screenshot { mode } => task_screenshot(&mode),
        XtaskCommand::Record => task_record(),
        XtaskCommand::Logs { service, lines } => task_logs(&service, lines),
        XtaskCommand::Msg { command } => task_msg(command).await,
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
    println!("🔨 Building `night-pinnacle`, `nightctl`, `night-settings`, and `night-session` in release mode...");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "night-pinnacle", "-p", "nightctl", "-p", "night-settings", "-p", "night-session"])
        .current_dir(root)
        .status()
        .expect("Failed to run cargo build");

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
    println!("✅ Build complete: target/release/night-pinnacle, nightctl, night-settings, night-session");
}

fn task_install(root: &Path) {
    println!("🌙 Installing Project Night...");

    // 1. Compile in release mode
    task_build(root);

    let home = dirs::home_dir().expect("Could not determine user home directory");
    let cargo_bin = home.join(".cargo/bin");
    let _ = fs::create_dir_all(&cargo_bin);

    // 2. Install binaries to ~/.cargo/bin
    for bin_name in ["night-pinnacle", "nightctl", "night-settings", "night-session"] {
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

    let source_toml = if root.join("config/pinnacle.toml").exists() {
        root.join("config/pinnacle.toml")
    } else {
        root.join("pinnacle.toml")
    };
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
        for file in ["night.toml", "rules.ron", "monitors.ron"] {
            let src = default_config_dir.join(file);
            let dst = night_user_dir.join(file);
            if src.exists() && !dst.exists() {
                println!("==> Initializing default config: {}", dst.display());
                let _ = fs::copy(&src, &dst);
            }
        }
    }

    // 4b. Deploy Bihar Night default wallpapers (~/.config/night/wallpapers/)
    let wallpapers_user_dir = night_user_dir.join("wallpapers");
    let _ = fs::create_dir_all(&wallpapers_user_dir);
    let wallpapers_src_dir = root.join("assets/wallpapers");
    if wallpapers_src_dir.exists() {
        if let Ok(entries) = fs::read_dir(&wallpapers_src_dir) {
            for entry in entries.flatten() {
                let src = entry.path();
                if src.is_file() {
                    let dst = wallpapers_user_dir.join(entry.file_name());
                    if !dst.exists() {
                        println!("==> Deploying Bihar Night wallpaper: {}", dst.display());
                        let _ = fs::copy(&src, &dst);
                    }
                }
            }
        }
    }

    // 5. Deploy Bihar Night styling configs for desktop tools
    task_theme(root, night_core::config::UserConfig::load().general.theme);

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
                println!("==> Deploying Bihar Night theme: {}", target_path.display());
                let _ = fs::copy(&src, &target_path);
            }
        }
    }

    // 6. Install Wayland session desktop entry
    task_session(root);

    println!("🎉 Night successfully installed! Launch your compositor by typing: pinnacle (or select Night in your display manager)");
}

fn task_theme(root: &Path, variant: ThemeVariant) {
    println!(
        "🎨 Generating Bihar Night {} ({}) styling configurations for desktop tools...",
        variant.name(),
        variant.bihar_name()
    );
    let config_dir = root.join("config");
    fs::create_dir_all(&config_dir).expect("Failed to create config directory");

    let p = variant.palette();

    // 1. Alacritty
    let alacritty_cfg = format!(
        r##"# 🌙 Bihar Night {} ({}) theme for Alacritty generated by Night Desktop
[colors.primary]
background = "{}"
foreground = "{}"

[colors.normal]
black = "{}"
red = "{}"
green = "{}"
yellow = "{}"
blue = "{}"
magenta = "{}"
cyan = "{}"
white = "{}"

[colors.bright]
black = "{}"
red = "{}"
green = "{}"
yellow = "{}"
blue = "{}"
magenta = "{}"
cyan = "{}"
white = "{}"

[font]
size = 11.0

[window]
opacity = 0.95
padding = {{ x = 8, y = 8 }}
"##,
        variant.name(),
        variant.bihar_name(),
        p.bg_primary,
        p.fg_primary,
        p.bg_darkest,
        p.error_red,
        p.success_green,
        p.warning_yellow,
        p.accent_blue,
        p.accent_magenta,
        p.accent_cyan,
        p.fg_muted,
        p.bg_highlight,
        p.error_red,
        p.success_green,
        p.warning_yellow,
        p.accent_blue,
        p.accent_magenta,
        p.accent_cyan,
        p.fg_primary,
    );
    fs::write(config_dir.join("alacritty.toml"), alacritty_cfg).expect("Failed to write alacritty.toml");

    // 2. Ironbar CSS Stylesheet
    let ironbar_css = format!(
        r##"/* 🌙 Bihar Night {} ({}) GTK4 stylesheet for Ironbar generated by Night Desktop */
* {{
    font-family: "JetBrainsMono Nerd Font", monospace;
    font-size: 13px;
    border: none;
    border-radius: 0;
}}

window#ironbar {{
    background-color: {};
    color: {};
    border-bottom: 2px solid {};
}}

.widget {{
    background-color: {};
    color: {};
    border-radius: 8px;
    margin: 4px 3px;
    padding: 2px 10px;
}}

.widget:hover {{
    background-color: {};
}}

.workspaces .item.focused {{
    background-color: {};
    color: {};
    font-weight: bold;
}}
"##,
        variant.name(),
        variant.bihar_name(),
        p.bg_primary,
        p.fg_primary,
        p.bg_secondary,
        p.bg_secondary,
        p.fg_primary,
        p.bg_highlight,
        p.accent_blue,
        p.bg_darkest,
    );
    fs::write(config_dir.join("ironbar.css"), ironbar_css).expect("Failed to write ironbar.css");

    // 2b. Ironbar TOML Configuration
    let ironbar_toml = r##"# 🌙 Bihar Night status bar configuration for Ironbar (GTK4-layer-shell)
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
    let anyrun_css = r##"/* 🌙 Bihar Night GTK4 stylesheet for Anyrun generated by Night Desktop */
#window {
    background-color: transparent;
}

#main {
    background-color: rgba(26, 27, 38, 0.96);
    border: 2px solid #7aa2f7;
    border-radius: 12px;
    padding: 10px;
}

#entry {
    background-color: #24283b;
    color: #c0caf5;
    border-radius: 8px;
    padding: 6px 12px;
}

#match.selected {
    background-color: #7aa2f7;
    color: #15161e;
    border-radius: 6px;
}
"##;
    fs::write(config_dir.join("anyrun.css"), anyrun_css).expect("Failed to write anyrun.css");

    // 4. Veila (Pure-Rust Wayland Lockscreen)
    let veila_toml = r##"# 🌙 Bihar Night theme for Veila Screen Locker
[general]
indicator = true
indicator_radius = 45.0
indicator_thickness = 5.0

[colors]
background = "1a1b26"
text = "c0caf5"
ring = "24283b"
inside = "15161e"
key_hl = "7aa2f7"
bs_hl = "f7768e"
separator = "414868"
verify = "bb9af7"
wrong = "f7768e"
"##;
    fs::write(config_dir.join("veila.toml"), veila_toml).expect("Failed to write veila.toml");

    println!("✅ Bihar Night suite configs exported to ./config/");
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

async fn task_msg(command: MsgCommand) {
    let cmd = match command {
        MsgCommand::Reload => night_core::ipc::IpcCommand::ReloadConfig,
        MsgCommand::Tag { name } => night_core::ipc::IpcCommand::SwitchTag(name),
        MsgCommand::State => night_core::ipc::IpcCommand::QueryState,
        MsgCommand::Spawn { command } => night_core::ipc::IpcCommand::Spawn(command.join(" ")),
        MsgCommand::Close => night_core::ipc::IpcCommand::CloseFocused,
        MsgCommand::Status => night_core::ipc::IpcCommand::GetStatus,
        MsgCommand::Scratchpad => night_core::ipc::IpcCommand::ToggleScratchpad,
        MsgCommand::Layout { action } => {
            if let Some(act) = action {
                if act == "next" {
                    night_core::ipc::IpcCommand::CycleLayout
                } else {
                    night_core::ipc::IpcCommand::GetLayout
                }
            } else {
                night_core::ipc::IpcCommand::GetLayout
            }
        }
        MsgCommand::Gaps { delta } => night_core::ipc::IpcCommand::AdjustGaps(delta),
        MsgCommand::Overlay => night_core::ipc::IpcCommand::ShowBinds,
        MsgCommand::QuitPrompt => night_core::ipc::IpcCommand::ShowQuit,
        MsgCommand::Switcher => night_core::ipc::IpcCommand::ShowSwitcher,
        MsgCommand::Gamma { temp } => {
            if let Some(t) = temp {
                night_core::ipc::IpcCommand::SetGamma(t)
            } else {
                night_core::ipc::IpcCommand::ToggleGamma
            }
        }
        MsgCommand::Inhibit => night_core::ipc::IpcCommand::ToggleInhibit,
        MsgCommand::Clip => night_core::ipc::IpcCommand::ShowClipboard,
        MsgCommand::Media => night_core::ipc::IpcCommand::Media(night_core::ipc::MediaAction::PlayPause),
        MsgCommand::Snip => night_core::ipc::IpcCommand::ShowSnipping,
        MsgCommand::Theme { variant } => {
            if let Some(v_str) = variant {
                if let Some(v) = ThemeVariant::parse_variant(&v_str) {
                    night_core::ipc::IpcCommand::SetTheme(v)
                } else {
                    eprintln!("Unknown variant '{v_str}'. Use Night, Storm, Moon, or Day.");
                    exit(1);
                }
            } else {
                night_core::ipc::IpcCommand::GetTheme
            }
        }
        MsgCommand::Volume { action } => {
            let act = match action.as_deref() {
                Some("raise") | Some("up") => night_core::audio::VolumeAction::Raise(5),
                Some("lower") | Some("down") => night_core::audio::VolumeAction::Lower(5),
                Some("mute") | Some("toggle") => night_core::audio::VolumeAction::MuteToggle,
                _ => night_core::audio::VolumeAction::Query,
            };
            night_core::ipc::IpcCommand::Volume(act)
        }
        MsgCommand::Power => night_core::ipc::IpcCommand::GetBattery,
        MsgCommand::Launcher => night_core::ipc::IpcCommand::ShowLauncher,
        MsgCommand::Net => night_core::ipc::IpcCommand::GetNetwork,
        MsgCommand::Top => night_core::ipc::IpcCommand::GetSystemStats,
        MsgCommand::Bt => night_core::ipc::IpcCommand::GetBluetooth,
        MsgCommand::Sink { action } => match action.as_deref() {
            Some("cycle") | Some("next") => night_core::ipc::IpcCommand::CycleAudioSink,
            Some("menu") => night_core::ipc::IpcCommand::ShowAudioSinkMenu,
            Some(id_str) if id_str.parse::<u32>().is_ok() => {
                night_core::ipc::IpcCommand::SetAudioSink(id_str.parse::<u32>().unwrap())
            }
            _ => night_core::ipc::IpcCommand::GetAudioSinks,
        },
        MsgCommand::Brightness { action } => {
            let act = match action.as_deref() {
                Some("raise") | Some("up") => night_core::backlight::BacklightAction::Raise(5),
                Some("lower") | Some("down") => night_core::backlight::BacklightAction::Lower(5),
                Some(pct_str) if pct_str.parse::<u8>().is_ok() => {
                    night_core::backlight::BacklightAction::Set(pct_str.parse::<u8>().unwrap())
                }
                _ => night_core::backlight::BacklightAction::Query,
            };
            night_core::ipc::IpcCommand::Backlight(act)
        }
        MsgCommand::Dnd => night_core::ipc::IpcCommand::ToggleDnd,
        MsgCommand::Notify => night_core::ipc::IpcCommand::ShowNotifications,
        MsgCommand::Session { action } => match action.as_str() {
            "restore" => night_core::ipc::IpcCommand::RestoreSession,
            _ => night_core::ipc::IpcCommand::SaveSession,
        },
        MsgCommand::Wallpaper { path } => {
            if let Some(p) = path {
                night_core::ipc::IpcCommand::Spawn(format!("swww img \"{p}\" --transition-type wipe --transition-duration 1.5"))
            } else {
                night_core::ipc::IpcCommand::Spawn("swww query".into())
            }
        }
        MsgCommand::Stream => {
            println!("📡 Subscribing to real-time Night status stream...");
            let res = night_core::ipc::subscribe_status(|s| {
                println!(
                    "[{}] Tag {} | Window: {}",
                    s.layout,
                    s.active_tag,
                    s.active_window.as_deref().unwrap_or("None")
                );
            })
            .await;
            if let Err(e) = res {
                eprintln!("❌ Stream disconnected: {e}");
            }
            return;
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
            println!(
                "  Focused Window       : {}",
                s.active_window.unwrap_or_else(|| "None".into())
            );
            if let Some(ref v) = s.volume {
                println!("  Audio Volume         : {}", v.display_string());
            }
            if let Some(ref b) = s.battery {
                println!("  Battery Power        : {} ({})", b.display_string(), b.status);
            }
            if let Some(ref n) = s.network {
                println!("  Network Connectivity : {}", n.display_string());
            }
            if let Some(ref sys) = s.system {
                println!("  System Resources     : CPU: {}% | RAM: {}% ({:.1}/{:.1} GiB)", sys.cpu_pct, sys.mem_pct, sys.mem_used_mb as f32 / 1024.0, sys.mem_total_mb as f32 / 1024.0);
            }
            if let Some(ref bt) = s.bluetooth {
                println!("  Bluetooth Status     : {}", bt.display_string());
            }
            if let Some(ref bl) = s.backlight {
                println!("  Display Backlight    : {}", bl.display_string());
            }
            println!("  Do Not Disturb (DND) : {}", if s.dnd { "󰂛 Enabled (Muted)" } else { "󰂚 Disabled (Normal)" });
            println!("  Last Reload Timestamp: {}", s.last_reload);
        }
        Ok(night_core::ipc::IpcResponse::Battery(Some(bat))) => {
            println!("⚡ Battery Status: {} ({})", bat.display_string(), bat.status);
        }
        Ok(night_core::ipc::IpcResponse::Battery(None)) => {
            println!("⚡ Running on AC Power (no battery detected)");
        }
        Ok(night_core::ipc::IpcResponse::Volume(Some(vol))) => {
            println!("🔊 Audio Volume: {}", vol.display_string());
        }
        Ok(night_core::ipc::IpcResponse::Volume(None)) => {
            println!("🔊 Audio: No active sink");
        }
        Ok(night_core::ipc::IpcResponse::Network(Some(net))) => {
            println!("🌐 Network Status: {}", net.display_string());
            println!("  Interface       : {}", net.interface);
            println!("  Type            : {:?}", net.kind);
            if let Some(ref ssid) = net.ssid {
                println!("  SSID            : {ssid}");
            }
            if let Some(sig) = net.signal_pct {
                println!("  Signal Quality  : {sig}%");
            }
            if let Some(ref ip) = net.ip_addr {
                println!("  Local IPv4      : {ip}");
            }
            println!("  Online          : {}", if net.is_online { "Yes" } else { "No" });
        }
        Ok(night_core::ipc::IpcResponse::Network(None)) => {
            println!("🌐 Network: Disconnected / No active interface");
        }
        Ok(night_core::ipc::IpcResponse::SystemStats(s)) => {
            println!("🌙 Bihar Night System Resource Monitor:");
            println!("   CPU Usage    : {}%", s.cpu_pct);
            println!("  󰍛 Memory (RAM) : {:.1} / {:.1} GiB ({}%)", s.mem_used_mb as f32 / 1024.0, s.mem_total_mb as f32 / 1024.0, s.mem_pct);
            println!("  󰋊 Root Disk    : {} / {} GiB ({}%)", s.disk_used_gb, s.disk_total_gb, s.disk_pct);
            if let Some(t) = s.cpu_temp_c {
                println!("  󰔏 Temperature  : {}°C", t);
            }
        }
        Ok(night_core::ipc::IpcResponse::Bluetooth(Some(bt))) => {
            println!("󰂯 Bluetooth Status: {}", bt.display_string());
            println!("  Powered   : {}", if bt.is_powered { "Yes" } else { "No" });
            println!("  Connected : {}", if bt.is_connected { "Yes" } else { "No" });
            if !bt.devices.is_empty() {
                println!("  Connected Wireless Peripherals:");
                for dev in &bt.devices {
                    let bat = dev.battery_pct.map(|b| format!("({b}%)")).unwrap_or_default();
                    println!("    {} {:<28} {} [{}]", dev.icon, dev.name, bat, dev.mac);
                }
            }
        }
        Ok(night_core::ipc::IpcResponse::Bluetooth(None)) => {
            println!("󰂲 Bluetooth: Controller not detected or uninitialized");
        }
        Ok(night_core::ipc::IpcResponse::AudioSinks(sinks)) => {
            println!("🔊 Audio Output Sinks (PipeWire / PulseAudio):");
            if sinks.is_empty() {
                println!("  No active audio output sinks found.");
            } else {
                for sink in sinks {
                    let marker = if sink.is_default { "▶ [ACTIVE]" } else { "          " };
                    println!("  {} [{}] {} {}", marker, sink.id, sink.icon(), sink.name);
                }
            }
        }
        Ok(night_core::ipc::IpcResponse::Backlight(Some(bl))) => {
            println!("🔆 Display Backlight: {} (device: {})", bl.display_string(), bl.device_name);
        }
        Ok(night_core::ipc::IpcResponse::Backlight(None)) => {
            println!("🔆 Display Backlight: No controllable hardware backlight detected");
        }
        Ok(night_core::ipc::IpcResponse::Dnd(dnd)) => {
            let icon = night_core::notifications::NotificationHistory::glyph_dnd(dnd);
            println!("{icon} Do Not Disturb: {}", if dnd { "ENABLED (Notifications Muted)" } else { "DISABLED (Normal)" });
        }
        Ok(night_core::ipc::IpcResponse::SavedSession(Some(s))) => {
            println!("🔄 Restored window session from timestamp {} ({} windows)", s.timestamp, s.windows.len());
        }
        Ok(night_core::ipc::IpcResponse::SavedSession(None)) => {
            println!("🔄 No saved session file found on disk");
        }
        Ok(resp) => println!("✅ Response: {resp:?}"),
        Err(e) => {
            eprintln!("❌ Failed to deliver command: {e}");
            exit(1);
        }
    }
}

fn task_logs(service: &str, lines: usize) {
    let recent = night_core::logging::read_recent_logs(service, lines);
    let path = night_core::logging::log_file_path(service);
    println!("🌙 Project Night Logs — Service: `{service}`");
    println!("📁 File: {}\n", path.display());
    for line in recent {
        println!("{line}");
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
        ("Vivi Media Player", "vivi", "vivi-bin"),
        ("Ironbar Status Bar", "ironbar", "ironbar-bin"),
        ("Wired Notification Daemon", "wired", "wired-notify-bin"),
        ("SwayOSD Overlay Server", "swayosd-server", "swayosd-git"),
        ("swww Wallpaper Daemon", "swww-daemon", "swww"),
        ("Wayshot Screenshot CLI", "wayshot", "wayshot"),
        ("Satty Annotation Tool", "satty", "satty-bin"),
        ("wl-screenrec Recorder", "wl-screenrec", "wl-screenrec"),
        ("wayidle Idle Daemon", "wayidle", "wayidle"),
        ("Veila Screen Locker", "veila", "veila-bin"),
        ("wlsunset Night Light", "wlsunset", "wlsunset"),
        ("rsclip Clipboard Manager", "rsclip", "rsclip-git"),
    ];

    let mut missing = Vec::new();
    println!("📦 External 100% Rust-Native Desktop Suite:");
    for (name, bin, pkg) in required_tools {
        let found = if bin == "zen-browser" {
            which::which("zen-browser").is_ok() || which::which("zen").is_ok()
        } else if bin == "rsclip" {
            which::which("rsclip").is_ok() || which::which("rsclipd").is_ok()
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
            ("Monitor Profiles (RON)", cfg.join("night/monitors.ron")),
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

    // 1. Install night-session compiled binary to ~/.cargo/bin
    let bin_dir = home.join(".cargo/bin");
    let _ = fs::create_dir_all(&bin_dir);
    let session_src = root.join("target/release/night-session");
    let session_dst = bin_dir.join("night-session");
    if session_src.exists() {
        if let Ok(_) = fs::copy(&session_src, &session_dst) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = fs::metadata(&session_dst) {
                    let mut perms = meta.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&session_dst, perms);
                }
            }
            println!("==> Installed pure-Rust session binary to {}", session_dst.display());
        }
    } else {
        // Fallback to assets launcher if release binary not yet built
        let fallback_src = root.join("assets/night-session");
        if fallback_src.exists() {
            let _ = fs::copy(&fallback_src, &session_dst);
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

    // 3. Install systemd user services to ~/.config/systemd/user/
    let systemd_user_dir = home.join(".config/systemd/user");
    if let Ok(_) = fs::create_dir_all(&systemd_user_dir) {
        let service_src = root.join("assets/night.service");
        if service_src.exists() {
            let _ = fs::copy(&service_src, systemd_user_dir.join("night.service"));
            let _ = fs::copy(&service_src, systemd_user_dir.join("pinnacle.service"));
            println!("==> Installed systemd user units: night.service & pinnacle.service");
        }
        let shutdown_src = root.join("assets/night-shutdown.target");
        if shutdown_src.exists() {
            let _ = fs::copy(&shutdown_src, systemd_user_dir.join("night-shutdown.target"));
            let _ = fs::copy(&shutdown_src, systemd_user_dir.join("pinnacle-shutdown.target"));
            println!("==> Installed systemd user units: night-shutdown.target & pinnacle-shutdown.target");
        }
    }

    // 4. Install XDG desktop portal configurations
    let portal_dir = home.join(".config/xdg-desktop-portal");
    if let Ok(_) = fs::create_dir_all(&portal_dir) {
        let portal_src = root.join("assets/night-portals.conf");
        if portal_src.exists() {
            let _ = fs::copy(&portal_src, portal_dir.join("night-portals.conf"));
            let _ = fs::copy(&portal_src, portal_dir.join("pinnacle-portals.conf"));
            println!("==> Installed XDG desktop portal configs for Night & Pinnacle");
        }
    }
}

fn task_dm(root: &Path) {
    println!("🌙 Configuring 100% Rust-Native TUI Display Manager (greetd + tuigreet)...");

    let is_root = Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
        .unwrap_or(false);

    if !is_root {
        println!("ℹ️ Notice: Administrative commands will invoke `sudo`.");
    }

    // 1. Disable cosmic-greeter
    println!("==> Disabling cosmic-greeter.service...");
    let mut cmd = if is_root { Command::new("systemctl") } else { Command::new("sudo") };
    if !is_root { cmd.arg("systemctl"); }
    let _ = cmd.args(["stop", "cosmic-greeter.service"]).output();

    let mut cmd = if is_root { Command::new("systemctl") } else { Command::new("sudo") };
    if !is_root { cmd.arg("systemctl"); }
    let _ = cmd.args(["disable", "cosmic-greeter.service"]).output();

    // 2. Install greetd & greetd-tuigreet
    println!("==> Installing greetd and greetd-tuigreet via pacman...");
    let mut cmd = if is_root { Command::new("pacman") } else { Command::new("sudo") };
    if !is_root { cmd.arg("pacman"); }
    let _ = cmd.args(["-S", "--needed", "--noconfirm", "greetd", "greetd-tuigreet"]).status();

    // 3. Create cache directory for tuigreet
    println!("==> Configuring /var/cache/tuigreet for session persistence...");
    let mut cmd = if is_root { Command::new("mkdir") } else { Command::new("sudo") };
    if !is_root { cmd.arg("mkdir"); }
    let _ = cmd.args(["-p", "/var/cache/tuigreet"]).output();

    let mut cmd = if is_root { Command::new("chown") } else { Command::new("sudo") };
    if !is_root { cmd.arg("chown"); }
    let _ = cmd.args(["-R", "greeter:greeter", "/var/cache/tuigreet"]).output();

    // 4. Deploy Bihar Night /etc/greetd/config.toml
    println!("==> Deploying Bihar Night /etc/greetd/config.toml...");
    let src_config = root.join("assets/greetd-tuigreet.toml");
    let mut cmd = if is_root { Command::new("cp") } else { Command::new("sudo") };
    if !is_root { cmd.arg("cp"); }
    let _ = cmd.args([src_config.to_str().unwrap_or("assets/greetd-tuigreet.toml"), "/etc/greetd/config.toml"]).output();

    // 5. Update Wayland desktop entry and systemd service
    println!("==> Updating /usr/share/wayland-sessions/night.desktop...");
    let src_desktop = root.join("assets/night.desktop");
    let mut cmd = if is_root { Command::new("cp") } else { Command::new("sudo") };
    if !is_root { cmd.arg("cp"); }
    let _ = cmd.args([src_desktop.to_str().unwrap_or("assets/night.desktop"), "/usr/share/wayland-sessions/night.desktop"]).output();

    let src_svc = root.join("assets/night.service");
    let mut cmd = if is_root { Command::new("cp") } else { Command::new("sudo") };
    if !is_root { cmd.arg("cp"); }
    let _ = cmd.args([src_svc.to_str().unwrap_or("assets/night.service"), "/usr/lib/systemd/user/night.service"]).output();

    // 6. Enable greetd.service
    println!("==> Enabling greetd.service as primary display manager...");
    let mut cmd = if is_root { Command::new("systemctl") } else { Command::new("sudo") };
    if !is_root { cmd.arg("systemctl"); }
    let _ = cmd.args(["enable", "greetd.service"]).status();

    // 7. Remove cosmic-greeter package
    println!("==> Removing cosmic-greeter package...");
    let mut cmd = if is_root { Command::new("pacman") } else { Command::new("sudo") };
    if !is_root { cmd.arg("pacman"); }
    let _ = cmd.args(["-Rdd", "--noconfirm", "cosmic-greeter"]).output();

    println!("✨ Rust-native TUI display manager (tuigreet) is installed and active!");
    println!("   You can switch to it right now with: sudo systemctl restart greetd");
}

fn task_wallpaper(target: Option<PathBuf>) {
    println!("🖼️ Managing Bihar Night wallpaper in pure Rust...");

    // 1. Ensure swww-daemon is running if swww is present
    if which::which("swww").is_ok() {
        let is_running = Command::new("pgrep")
            .args(["-x", "swww-daemon"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !is_running {
            println!("==> Starting swww-daemon background daemon...");
            let _ = Command::new("swww-daemon").spawn();
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }

    let wallpaper_path = if let Some(p) = target {
        if p.exists() {
            Some(p)
        } else {
            eprintln!("⚠️ Specified wallpaper not found: {}", p.display());
            None
        }
    } else {
        // Search standard directories with priority for authentic Bihar Night wallpapers
        let home = dirs::home_dir().unwrap_or_default();
        let config_wp = dirs::config_dir().unwrap_or_else(|| home.join(".config")).join("night/wallpapers");
        let repo_wp = PathBuf::from("assets/wallpapers");

        let priority_names = [
            "bihar-night.jpg",
            "bihar-night-ganga.jpg",
            "bihar-night-bodhgaya.jpg",
            "bihar-night.png",
        ];

        let mut found_priority = None;
        for name in &priority_names {
            for dir in [&config_wp, &repo_wp] {
                let candidate = dir.join(name);
                if candidate.exists() {
                    found_priority = Some(candidate);
                    break;
                }
            }
            if found_priority.is_some() {
                break;
            }
        }

        if let Some(p) = found_priority {
            Some(p)
        } else {
            let search_dirs = [
                config_wp,
                repo_wp,
                home.join("Pictures/Wallpapers"),
                home.join(".config/quickshell/ii/assets/images"),
            ];

            let mut candidates = Vec::new();
            for dir in &search_dirs {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            let ext = ext.to_lowercase();
                            if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp") {
                                candidates.push(path);
                            }
                        }
                    }
                }
            }

            if !candidates.is_empty() {
                let idx = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as usize)
                    % candidates.len();
                Some(candidates[idx].clone())
            } else {
                None
            }
        }
    };

    if let Some(img) = wallpaper_path {
        println!("==> Setting Bihar Night wallpaper: {}", img.display());
        if which::which("swww").is_ok() {
            let _ = Command::new("swww")
                .args([
                    "img",
                    img.to_str().unwrap(),
                    "--transition-type",
                    "grow",
                    "--transition-pos",
                    "0.5,0.5",
                    "--transition-step",
                    "90",
                    "--transition-fps",
                    "60",
                    "--transition-duration",
                    "1.5",
                ])
                .status();
        } else if which::which("swaybg").is_ok() {
            let _ = Command::new("pkill").args(["-x", "swaybg"]).status();
            let _ = Command::new("swaybg")
                .args(["-i", img.to_str().unwrap(), "-m", "fill"])
                .spawn();
        }
    } else {
        println!("==> No wallpaper found; clearing to solid Bihar Night (#1a1b26)");
        if which::which("swww").is_ok() {
            let _ = Command::new("swww").args(["clear", "1a1b26"]).status();
        } else if which::which("swaybg").is_ok() {
            let _ = Command::new("pkill").args(["-x", "swaybg"]).status();
            let _ = Command::new("swaybg").args(["-c", "#1a1b26"]).spawn();
        }
    }
}

fn task_screenshot(mode: &str) {
    println!("📸 Project Night Screen Capture ({mode})...");
    let home = dirs::home_dir().unwrap_or_default();
    let pictures_dir = dirs::picture_dir()
        .unwrap_or_else(|| home.join("Pictures"))
        .join("Screenshots");
    let _ = fs::create_dir_all(&pictures_dir);

    let ts = Command::new("date")
        .arg("+%Y-%m-%d_%H-%M-%S")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "capture".into());

    let target_file = pictures_dir.join(format!("Screenshot_{ts}.png"));

    match mode {
        "area" => {
            println!("==> Interactive region selection (drag with mouse)...");
            let slurp_res = Command::new("slurp")
                .args(["-b", "#1a1b26cc", "-c", "#7aa2f7", "-w", "2", "-s", "#7aa2f733"])
                .output();

            if let Ok(output) = slurp_res {
                let geom = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !geom.is_empty() {
                    let wayshot = Command::new("wayshot")
                        .args(["-s", &geom, "--stdout"])
                        .stdout(Stdio::piped())
                        .spawn();

                    if let Ok(wayshot_child) = wayshot {
                        if let Some(stdout) = wayshot_child.stdout {
                            let _ = Command::new("satty")
                                .args(["--filename", "-", "--output-filename", target_file.to_str().unwrap()])
                                .stdin(stdout)
                                .status();
                        }
                    }
                }
            }
        }
        "full" => {
            println!("==> Capturing full screen to {}...", target_file.display());
            let _ = Command::new("wayshot")
                .args(["-f", target_file.to_str().unwrap()])
                .status();

            if which::which("rsclipd").is_ok() {
                if let Ok(file) = fs::File::open(&target_file) {
                    let _ = Command::new("rsclipd")
                        .args(["store", "--mime", "image/png"])
                        .stdin(file)
                        .status();
                }
            }

            if which::which("notify-send").is_ok() {
                let _ = Command::new("notify-send")
                    .args([
                        "-a",
                        "Night Desktop",
                        "Screenshot Captured",
                        &format!("Saved to {}", target_file.display()),
                    ])
                    .status();
            }
            println!("✅ Screenshot saved to {}", target_file.display());
        }
        "window" => {
            println!("==> Click window to capture...");
            let slurp_res = Command::new("slurp").output();
            if let Ok(output) = slurp_res {
                let geom = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !geom.is_empty() {
                    let wayshot = Command::new("wayshot")
                        .args(["-s", &geom, "--stdout"])
                        .stdout(Stdio::piped())
                        .spawn();

                    if let Ok(wayshot_child) = wayshot {
                        if let Some(stdout) = wayshot_child.stdout {
                            let _ = Command::new("satty")
                                .args(["--filename", "-", "--output-filename", target_file.to_str().unwrap()])
                                .stdin(stdout)
                                .status();
                        }
                    }
                }
            }
        }
        _ => eprintln!("❌ Unknown screenshot mode '{mode}'. Options: area, full, window"),
    }
}

fn task_record() {
    println!("🎥 Hardware-Accelerated Screen Recording Toggle...");
    let home = dirs::home_dir().unwrap_or_default();
    let videos_dir = dirs::video_dir()
        .unwrap_or_else(|| home.join("Videos"))
        .join("Recordings");
    let _ = fs::create_dir_all(&videos_dir);

    let is_running = Command::new("pgrep")
        .args(["-x", "wl-screenrec"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if is_running {
        println!("==> Stopping screen recording...");
        let _ = Command::new("pkill")
            .args(["-SIGINT", "-x", "wl-screenrec"])
            .status();

        if which::which("notify-send").is_ok() {
            let _ = Command::new("notify-send")
                .args([
                    "-a",
                    "Night Desktop",
                    "Screen Recording Saved",
                    &format!("Video saved to {}", videos_dir.display()),
                ])
                .status();
        }
        println!("✅ Screen recording stopped.");
    } else {
        let ts = Command::new("date")
            .arg("+%Y-%m-%d_%H-%M-%S")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "recording".into());

        let target_file = videos_dir.join(format!("Recording_{ts}.mp4"));
        println!("==> Starting recording to {}...", target_file.display());

        if which::which("notify-send").is_ok() {
            let _ = Command::new("notify-send")
                .args([
                    "-a",
                    "Night Desktop",
                    "Screen Recording Started",
                    &format!("Capturing to {}", target_file.display()),
                ])
                .status();
        }

        let _ = Command::new("wl-screenrec")
            .args([
                "--audio",
                "--encode-resolution",
                "1920x1080",
                "-f",
                target_file.to_str().unwrap(),
            ])
            .spawn();
        println!("✅ Screen recording active in background.");
    }
}
