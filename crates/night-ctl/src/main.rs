//! 🌙 nightctl: Lightning-fast pure-Rust CLI client and status streaming tool
//! Built with `clap` for typed, declarative CLI parsing.
//! Connects directly to the Night Wayland Compositor via Unix domain sockets.

use clap::{Parser, Subcommand, ValueEnum};
use night_core::audio::VolumeAction;
use night_core::ipc::{
    send_command, subscribe_status, DesktopStatus, DpmsAction, IpcCommand, IpcResponse, OutputNav,
};
use night_core::theme::ThemeVariant;
use std::io::{self, Write};
use std::process::exit;

#[derive(Parser, Debug)]
#[command(
    name = "nightctl",
    author = "Night Desktop Team",
    version,
    about = "🌙 Lightning-fast pure-Rust CLI client and status streaming tool for the Night Wayland Desktop",
    long_about = "Connects directly to the Night Wayland Compositor via Unix domain sockets.\nProvides instant remote control, multi-monitor navigation, layout management, and zero-allocation event streaming for status bars."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Query current desktop status (workspace, layout, window, gaps, monitors)
    Status {
        /// Emit status output as raw JSON string
        #[arg(short, long)]
        json: bool,
    },

    /// Stream real-time status updates without polling lag (ideal for status bars)
    Stream {
        /// Emit streaming updates as raw JSON strings
        #[arg(short, long)]
        json: bool,

        /// Format output using a template string (e.g. "󰕰 {layout} | 󱂬 {active_tag} | {active_window}")
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Switch active workspace tag on the focused monitor
    #[command(alias = "switch-tag")]
    Tag {
        /// Target tag name (e.g. "1", "2", "3")
        name: String,
    },

    /// Query, set, or cycle the dynamic tiling layout algorithm (MasterStack, Monocle, Dwindle, Spiral, Fair)
    Layout {
        #[command(subcommand)]
        action: Option<LayoutCommand>,
    },

    /// Adjust or set tiling layout gaps in real-time
    Gaps {
        /// Delta value in pixels (e.g. 2.0, -2.0)
        delta: Option<f32>,
        /// Explicit inner gap in pixels
        #[arg(long)]
        inner: Option<f32>,
        /// Explicit outer gap in pixels
        #[arg(long)]
        outer: Option<f32>,
    },

    /// Launch the Slint-based GUI Control Center & Settings app
    Settings,

    /// Show the Snowcap Bihar Night interactive visual window switcher
    #[command(alias = "switch-window")]
    Switcher,

    /// Manage night light & display color temperature for late-night ergonomics
    Gamma {
        #[command(subcommand)]
        action: GammaCommand,
    },

    /// Toggle idle inhibition (presentation and long compile mode)
    Inhibit,

    /// Manage multi-monitor focus and window placement across displays
    #[command(alias = "monitor")]
    Output {
        #[command(subcommand)]
        action: OutputCommand,
    },

    /// Control DPMS display power across connected monitors
    Dpms {
        /// Power action to execute (on, off, toggle)
        #[arg(value_enum, default_value = "toggle")]
        action: DpmsArg,
    },

    /// Toggle a named scratchpad drawer (term, music, notes, fm)
    Scratchpad {
        /// Drawer name: "term" (default), "music", "notes", or "fm"
        name: Option<String>,
    },

    /// Manage Wayland clipboard history and interactive modal
    Clip {
        #[command(subcommand)]
        action: Option<ClipCommand>,
    },

    /// Internal helper to record clipboard text from argument or standard input
    #[command(name = "clip-record", hide = true)]
    ClipRecord {
        /// Text snippet to record
        text: Option<String>,
    },

    /// Control MPRIS media players and query active track info
    Media {
        #[command(subcommand)]
        action: MediaCommand,
    },

    /// Show the Snowcap interactive keybindings cheatsheet modal
    #[command(alias = "binds")]
    Overlay,

    /// Show the Snowcap quit confirmation prompt modal
    #[command(alias = "prompt-quit")]
    QuitPrompt,

    /// Show the Snowcap Bihar Night session menu modal
    #[command(alias = "power-menu")]
    SessionMenu,

    /// Perform window operations on the currently focused window
    Window {
        /// Window operation to execute
        #[arg(value_enum)]
        op: WindowOpArg,
    },

    /// Close the currently focused window
    Close,

    /// Spawn an application or shell command through the compositor
    Spawn {
        /// Command string to execute
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },

    /// Reload the compositor configuration
    Reload,

    /// Interactive Bihar Night snipping tool and screen recorder
    Snip {
        /// Optional direct capture mode (region, window, full, copy, record, gif)
        #[arg(value_enum)]
        mode: Option<SnipMode>,
    },

    /// Manage Bihar Night theme variants (Ganga Midnight, Rajgir Mist, Bodh Gaya Serenity, Nalanda Dawn)
    Theme {
        #[command(subcommand)]
        action: Option<ThemeCommand>,
    },

    /// Manage audio volume and mute status (PipeWire / PulseAudio)
    Volume {
        #[command(subcommand)]
        action: Option<VolumeCommand>,
    },

    /// Query battery capacity and manage Bihar Night battery conservation mode
    Power {
        #[command(subcommand)]
        action: Option<PowerCommand>,
    },

    /// Show the Snowcap Bihar Night application launcher modal
    #[command(alias = "run")]
    Launcher,

    /// Query network connectivity status and active interfaces
    Net,

    /// Display Bihar Night real-time system performance monitor (CPU, RAM, Temp, Disk)
    #[command(alias = "sys")]
    Top,

    /// Query Bluetooth controller and connected wireless peripherals
    Bt,

    /// Manage audio output sinks (headphones, speakers, HDMI, USB DAC)
    Sink {
        #[command(subcommand)]
        action: Option<SinkCommand>,
    },

    /// Manage display backlight brightness
    #[command(alias = "bl")]
    Brightness {
        #[command(subcommand)]
        action: Option<BrightnessCommand>,
    },

    /// Toggle or query Do Not Disturb (DND) mode
    Dnd {
        /// Optional action: "toggle", "on", "off", or "status"
        action: Option<String>,
    },

    /// Inspect notification history or open notification center
    #[command(alias = "notifications")]
    Notify {
        #[command(subcommand)]
        action: Option<NotifyCommand>,
    },

    /// Save or restore workspace window session state
    Session {
        #[command(subcommand)]
        action: SessionCommand,
    },

    /// Query or set Bihar Night desktop wallpaper
    Wallpaper {
        /// Optional path to wallpaper image file
        path: Option<String>,
    },

    /// Inspect or follow recent desktop logs (night, settings, ctl, session)
    #[command(alias = "logs")]
    Log {
        /// Target service: night, settings, ctl, session (default: "night")
        #[arg(default_value = "night")]
        service: String,

        /// Number of recent lines to display (default: 50)
        #[arg(short = 'n', long, default_value_t = 50)]
        lines: usize,

        /// Follow log file output continuously in real time
        #[arg(short = 'f', long)]
        follow: bool,
    },

    /// Trigger on-screen display (OSD) visual HUD overlay
    Osd {
        /// Target: volume, brightness, dnd, gamma, layout, or profile name
        #[arg(default_value = "volume")]
        target: String,
    },

    /// Switch or list workspace layout profiles (coding, reading, multitask, ultrawide)
    Profile {
        /// Profile name: coding, reading, multitask, ultrawide
        name: Option<String>,
    },

    /// Trigger the interactive Bihar Night Command Palette modal
    Palette,

    /// Trigger the interactive Bihar Night Window Switcher & Overview modal
    Overview,

    /// Trigger the interactive Bihar Night 3x3 Workspace Grid (Exposé) modal
    Grid,

    /// Snap the focused window to a geometric target (center, left, right, top, bottom, top-left, top-right, bottom-left, bottom-right, pip, max)
    Snap {
        /// Geometric snap position
        position: String,
    },

    /// Inspect and manage smart idle daemon and inactivity timers
    Idle {
        #[command(subcommand)]
        action: Option<IdleCommand>,
    },

    /// Manage window focus borders and titlebar decorations
    #[command(alias = "borders")]
    Border {
        #[command(subcommand)]
        action: BorderCommand,
    },
}

#[derive(Subcommand, Debug)]
enum BorderCommand {
    /// Toggle window focus border decorations on or off
    Toggle,
    /// Toggle window titlebars on or off
    Titlebars,
    /// Set window border thickness in pixels (1..16)
    Width { width: u32 },
}

#[derive(Subcommand, Debug)]
enum IdleCommand {
    /// Query current inactivity status and sleep/lock thresholds
    Status,
    /// Reset inactivity timers and wake display power
    Reset,
}

#[derive(Subcommand, Debug)]
enum BrightnessCommand {
    /// Raise display brightness by percentage (default 5%)
    Raise { delta: Option<u8> },
    /// Lower display brightness by percentage (default 5%)
    Lower { delta: Option<u8> },
    /// Set explicit display brightness percentage (0..100)
    Set { percentage: u8 },
    /// Query current brightness level
    Status,
}

#[derive(Subcommand, Debug)]
enum NotifyCommand {
    /// List recent desktop notification history
    List,
    /// Clear all notification history
    Clear,
    /// Open the Snowcap interactive notification center modal
    Menu,
}

#[derive(Subcommand, Debug)]
enum SessionCommand {
    /// Save current running window session layout
    Save,
    /// Restore saved window session layout
    Restore,
    /// Clear saved window session file
    Clear,
    /// Immediately lock the desktop session using Veila
    Lock,
    /// Gracefully terminate and tear down the desktop session
    #[command(alias = "exit")]
    Logout,
    /// Query the status of the active desktop session and compositor
    Status,
}

#[derive(Subcommand, Debug)]
enum SinkCommand {
    /// List all discovered audio output sinks
    List,
    /// Cycle to the next audio output sink
    #[command(alias = "next")]
    Cycle,
    /// Set default audio output sink by ID
    Set { id: u32 },
    /// Open the Snowcap interactive audio output selector modal
    Menu,
}

#[derive(Subcommand, Debug)]
enum VolumeCommand {
    /// Raise volume by delta percentage (default 5%)
    Raise { delta: Option<u8> },
    /// Lower volume by delta percentage (default 5%)
    Lower { delta: Option<u8> },
    /// Set explicit volume percentage (0..100)
    Set { percentage: u8 },
    /// Toggle mute state
    Mute,
    /// Query current volume and mute state
    Status,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum SnipMode {
    /// Interactive region snip piped to Satty annotation editor
    Region,
    /// Snip focused window coordinates
    Window,
    /// Full screen screenshot saved to ~/Pictures/Screenshots/
    Full,
    /// Full screen screenshot copied directly to clipboard
    Copy,
    /// Hardware-accelerated 60fps MP4 recording
    Record,
    /// Animated screencast recording saved as GIF
    Gif,
}

#[derive(Subcommand, Debug)]
enum LayoutCommand {
    /// Cycle to the next layout
    Next,
    /// Explicitly set layout algorithm (e.g. monocle, masterstack, dwindle, spiral, fair)
    Set {
        /// Layout algorithm name
        name: String,
    },
    /// Direct switch to Monocle fullscreen-focus layout
    Monocle,
    /// Direct switch to Master-Stack layout
    Masterstack,
    /// Direct switch to Dwindle layout
    Dwindle,
    /// Direct switch to Spiral layout
    Spiral,
    /// Direct switch to Fair layout
    Fair,
    /// Query active layout algorithm
    Status,
}

#[derive(Subcommand, Debug)]
enum ThemeCommand {
    /// Set specific Bihar Night variant
    Set {
        /// Variant name or Bihar alias: "night" ("ganga"), "storm" ("rajgir"), "moon" ("bodhgaya"), "day" ("nalanda")
        #[arg(value_enum)]
        variant: ThemeVariantArg,
    },
    /// Cycle to the next Bihar Night theme variant
    Toggle,
    /// Query active Bihar Night variant
    Status,
    /// Control diurnal auto-scheduling across Nalanda Dawn, Rajgir Mist, and Ganga Midnight
    Schedule {
        /// Schedule action: on (enable), off (disable), toggle, or status (default)
        #[arg(value_enum)]
        action: Option<ScheduleArg>,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum ScheduleArg {
    #[value(alias = "enable")]
    On,
    #[value(alias = "disable")]
    Off,
    Toggle,
    Status,
}

#[derive(Subcommand, Debug)]
enum PowerCommand {
    /// Query battery status, charging level, and health
    Status,
    /// Manage Bihar Night battery conservation mode (drops brightness to 30%, switches to Ganga Midnight)
    Saver {
        /// Saver action: on (enable), off (disable), toggle, or status (default)
        #[arg(value_enum)]
        action: Option<PowerSaverArg>,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum PowerSaverArg {
    #[value(alias = "enable")]
    On,
    #[value(alias = "disable")]
    Off,
    Toggle,
    Status,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum ThemeVariantArg {
    #[value(alias = "ganga", alias = "mithila", alias = "patna")]
    Night,
    #[value(alias = "rajgir", alias = "mist")]
    Storm,
    #[value(alias = "bodhgaya", alias = "bodhi")]
    Moon,
    #[value(alias = "nalanda", alias = "dawn")]
    Day,
}

impl From<ThemeVariantArg> for ThemeVariant {
    fn from(arg: ThemeVariantArg) -> Self {
        match arg {
            ThemeVariantArg::Night => ThemeVariant::Night,
            ThemeVariantArg::Storm => ThemeVariant::Storm,
            ThemeVariantArg::Moon => ThemeVariant::Moon,
            ThemeVariantArg::Day => ThemeVariant::Day,
        }
    }
}

#[derive(Subcommand, Debug)]
enum OutputCommand {
    /// Focus the next connected display output
    Next,
    /// Focus the previous connected display output
    Prev,
    /// Focus a display output by connector name (e.g. "DP-1")
    Focus { name: String },
    /// Move the currently focused window to next, prev, or named output
    MoveWindow {
        /// Target output ("next", "prev", or connector name)
        target: String,
    },
    /// Swap active workspaces and windows between connected monitors
    Swap,
    /// Mirror active workspace tag to the secondary display output
    Mirror,
}

#[derive(Subcommand, Debug)]
enum GammaCommand {
    /// Toggle night light between day (6500K) and warm night temperature
    Toggle,
    /// Set explicit color temperature in Kelvin (e.g. 3800, 4500, 6500)
    Set { temp: u32 },
    /// Set neutral daylight color temperature (6500K)
    Day,
    /// Set warm night light color temperature (3800K)
    Night,
    /// Query current color temperature
    Status,
}

#[derive(Subcommand, Debug)]
enum ClipCommand {
    /// List stored clipboard history entries
    List,
    /// Open the Snowcap Bihar Night clipboard history modal
    Show,
    /// Clear all stored clipboard history
    Clear,
    /// Copy a specific history index back into active system clipboard
    Copy { index: usize },
    /// Record text snippet into history
    Record { text: Option<String> },
}

#[derive(Subcommand, Debug)]
enum MediaCommand {
    /// Toggle play/pause on active media player
    #[command(alias = "play", alias = "pause")]
    PlayPause,
    /// Skip to next media track
    Next,
    /// Return to previous media track
    #[command(alias = "previous")]
    Prev,
    /// Stop media playback
    Stop,
    /// Query current playing media track info
    Status,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum DpmsArg {
    On,
    Off,
    Toggle,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum WindowOpArg {
    Sticky,
    Pip,
    Center,
    Float,
    Fullscreen,
    Maximize,
    Swallow,
    Unswallow,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    night_core::exec::ensure_desktop_env();
    let cli = Cli::parse();

    match cli.command {
        Commands::Status { json } => match send_command(&IpcCommand::GetStatus).await {
            Ok(IpcResponse::Status(status)) => {
                if json {
                    println!("{}", status.to_json());
                } else {
                    print_status_table(&status);
                }
            }
            Ok(other) => {
                eprintln!("Unexpected response: {other:?}");
                exit(1);
            }
            Err(e) => fatal(format!("Error communicating with Night compositor: {e}")),
        },

        Commands::Stream { json, format } => {
            let result = subscribe_status(move |status| {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                if json {
                    let _ = writeln!(handle, "{}", status.to_json());
                } else if let Some(ref tmpl) = format {
                    let _ = writeln!(handle, "{}", status.format_template(tmpl));
                } else {
                    let win = status.active_window.as_deref().unwrap_or("Desktop");
                    let _ = writeln!(handle, "[{}] Tag {} | {}", status.layout, status.active_tag, win);
                }
                let _ = handle.flush();
            })
            .await;

            if let Err(e) = result {
                fatal(format!("Event stream disconnected: {e}"));
            }
        }

        Commands::Tag { name } => {
            run_cmd(IpcCommand::SwitchTag(name)).await;
        }

        Commands::Layout { action } => match action {
            None | Some(LayoutCommand::Status) => match send_command(&IpcCommand::GetLayout).await {
                Ok(IpcResponse::Layout(l)) => println!("{l}"),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
            Some(LayoutCommand::Next) => match send_command(&IpcCommand::CycleLayout).await {
                Ok(IpcResponse::Layout(l)) => println!("{l}"),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
            Some(LayoutCommand::Set { name }) => match send_command(&IpcCommand::SetLayout(name)).await {
                Ok(IpcResponse::Layout(l)) => println!("📐 Switched tiling layout to: {l}"),
                Ok(IpcResponse::Error(e)) => fatal(e),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
            Some(LayoutCommand::Monocle) => match send_command(&IpcCommand::SetLayout("monocle".into())).await {
                Ok(IpcResponse::Layout(l)) => println!("📐 Switched tiling layout to: {l}"),
                Ok(IpcResponse::Error(e)) => fatal(e),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
            Some(LayoutCommand::Masterstack) => match send_command(&IpcCommand::SetLayout("masterstack".into())).await {
                Ok(IpcResponse::Layout(l)) => println!("📐 Switched tiling layout to: {l}"),
                Ok(IpcResponse::Error(e)) => fatal(e),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
            Some(LayoutCommand::Dwindle) => match send_command(&IpcCommand::SetLayout("dwindle".into())).await {
                Ok(IpcResponse::Layout(l)) => println!("📐 Switched tiling layout to: {l}"),
                Ok(IpcResponse::Error(e)) => fatal(e),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
            Some(LayoutCommand::Spiral) => match send_command(&IpcCommand::SetLayout("spiral".into())).await {
                Ok(IpcResponse::Layout(l)) => println!("📐 Switched tiling layout to: {l}"),
                Ok(IpcResponse::Error(e)) => fatal(e),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
            Some(LayoutCommand::Fair) => match send_command(&IpcCommand::SetLayout("fair".into())).await {
                Ok(IpcResponse::Layout(l)) => println!("📐 Switched tiling layout to: {l}"),
                Ok(IpcResponse::Error(e)) => fatal(e),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
        },

        Commands::Gaps { delta, inner, outer } => {
            if inner.is_some() || outer.is_some() {
                run_cmd(IpcCommand::SetGaps { inner, outer }).await;
            } else if let Some(d) = delta {
                run_cmd(IpcCommand::AdjustGaps(d)).await;
            } else {
                eprintln!("Error: Specify a delta (e.g. `nightctl gaps 5`) or `--inner`/`--outer`");
            }
        }

        Commands::Settings => {
            run_cmd(IpcCommand::Spawn("night-settings".into())).await;
        }

        Commands::Switcher => {
            run_cmd(IpcCommand::ShowSwitcher).await;
        }

        Commands::Gamma { action } => match action {
            GammaCommand::Toggle => match send_command(&IpcCommand::ToggleGamma).await {
                Ok(IpcResponse::Gamma(t)) => println!("🌙 Night light toggled: {t}K"),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
            GammaCommand::Set { temp } => match send_command(&IpcCommand::SetGamma(temp)).await {
                Ok(IpcResponse::Gamma(t)) => println!("🌙 Night light set to {t}K"),
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
            GammaCommand::Day => match send_command(&IpcCommand::SetGamma(6500)).await {
                Ok(_) => println!("☀️ Daylight temperature set (6500K)"),
                Err(e) => fatal(e),
            },
            GammaCommand::Night => match send_command(&IpcCommand::SetGamma(3800)).await {
                Ok(_) => println!("🌙 Warm night light set (3800K)"),
                Err(e) => fatal(e),
            },
            GammaCommand::Status => match send_command(&IpcCommand::GetStatus).await {
                Ok(IpcResponse::Status(s)) => {
                    let temp = s.gamma_temp.unwrap_or(6500);
                    println!("🌙 Current color temperature: {temp}K");
                }
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
        },

        Commands::Inhibit => match send_command(&IpcCommand::ToggleInhibit).await {
            Ok(IpcResponse::Inhibited(inh)) => {
                if inh {
                    println!("☕ Idle inhibition ENABLED (Presentation & compile mode active)");
                } else {
                    println!("🌙 Idle inhibition DISABLED (Normal power saving active)");
                }
            }
            Ok(resp) => println!("{resp:?}"),
            Err(e) => fatal(e),
        },

        Commands::Output { action } => match action {
            OutputCommand::Next => run_cmd(IpcCommand::FocusOutput(OutputNav::Next)).await,
            OutputCommand::Prev => run_cmd(IpcCommand::FocusOutput(OutputNav::Prev)).await,
            OutputCommand::Focus { name } => {
                run_cmd(IpcCommand::FocusOutput(OutputNav::Name(name))).await
            }
            OutputCommand::MoveWindow { target } => {
                let nav = match target.to_lowercase().as_str() {
                    "next" => OutputNav::Next,
                    "prev" => OutputNav::Prev,
                    _ => OutputNav::Name(target),
                };
                run_cmd(IpcCommand::MoveWindowToOutput(nav)).await;
            }
            OutputCommand::Swap => {
                run_cmd(IpcCommand::FocusOutput(OutputNav::Swap)).await;
                println!("🔄 Swapped active workspaces between displays.");
            }
            OutputCommand::Mirror => {
                run_cmd(IpcCommand::FocusOutput(OutputNav::Mirror)).await;
                println!("📽️ Mirrored active workspace tag to secondary display.");
            }
        },

        Commands::Dpms { action } => {
            let act = match action {
                DpmsArg::On => DpmsAction::On,
                DpmsArg::Off => DpmsAction::Off,
                DpmsArg::Toggle => DpmsAction::Toggle,
            };
            run_cmd(IpcCommand::Dpms(act)).await;
            println!("🌙 DPMS display power updated: {action:?}");
        }

        Commands::Scratchpad { name } => {
            let drawer = name.unwrap_or_else(|| "term".into());
            run_cmd(IpcCommand::ToggleNamedScratchpad(drawer)).await;
        }

        Commands::Clip { action } => {
            let act = action.unwrap_or(ClipCommand::Show);
            match act {
                ClipCommand::Show => {
                    run_cmd(IpcCommand::ShowClipboard).await;
                }
                ClipCommand::List => {
                    match send_command(&IpcCommand::Clipboard(night_core::ipc::ClipboardAction::List)).await {
                        Ok(IpcResponse::Clipboard(items)) => {
                            if items.is_empty() {
                                println!("Clipboard history is empty.");
                            } else {
                                println!("📋 Night Wayland Clipboard History ({} items):", items.len());
                                for (idx, item) in items.iter().take(15).enumerate() {
                                    let clean = item.replace(['\n', '\t'], " ");
                                    let preview = if clean.chars().count() > 60 {
                                        let s: String = clean.chars().take(57).collect();
                                        format!("{s}...")
                                    } else {
                                        clean
                                    };
                                    println!("  [{:>2}] ({} chars) {}", idx + 1, item.len(), preview);
                                }
                            }
                        }
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    }
                }
                ClipCommand::Clear => {
                    run_cmd(IpcCommand::Clipboard(night_core::ipc::ClipboardAction::Clear)).await;
                    println!("📋 Clipboard history cleared.");
                }
                ClipCommand::Copy { index } => {
                    let zero_idx = index.saturating_sub(1);
                    run_cmd(IpcCommand::Clipboard(night_core::ipc::ClipboardAction::Copy(zero_idx))).await;
                    println!("📋 Copied history item #{index} to active system clipboard.");
                }
                ClipCommand::Record { text } => {
                    let content = text.unwrap_or_else(|| {
                        use std::io::Read;
                        let mut buf = String::new();
                        let _ = io::stdin().read_to_string(&mut buf);
                        buf
                    });
                    if !content.trim().is_empty() {
                        run_cmd(IpcCommand::Clipboard(night_core::ipc::ClipboardAction::Record(content))).await;
                    }
                }
            }
        }

        Commands::ClipRecord { text } => {
            let content = text.unwrap_or_else(|| {
                use std::io::Read;
                let mut buf = String::new();
                let _ = io::stdin().read_to_string(&mut buf);
                buf
            });
            if !content.trim().is_empty() {
                night_core::clipboard::ClipboardHistory::record_text(&content);
            }
        }

        Commands::Media { action } => match action {
            MediaCommand::PlayPause => {
                run_cmd(IpcCommand::Media(night_core::ipc::MediaAction::PlayPause)).await;
            }
            MediaCommand::Next => {
                run_cmd(IpcCommand::Media(night_core::ipc::MediaAction::Next)).await;
            }
            MediaCommand::Prev => {
                run_cmd(IpcCommand::Media(night_core::ipc::MediaAction::Previous)).await;
            }
            MediaCommand::Stop => {
                run_cmd(IpcCommand::Media(night_core::ipc::MediaAction::Stop)).await;
            }
            MediaCommand::Status => match send_command(&IpcCommand::GetStatus).await {
                Ok(IpcResponse::Status(s)) => {
                    if let Some(media) = s.media_info {
                        println!("🎵 Now Playing: {media}");
                    } else {
                        println!("🎵 No active media player detected.");
                    }
                }
                Ok(resp) => println!("{resp:?}"),
                Err(e) => fatal(e),
            },
        },

        Commands::Overlay => {
            run_cmd(IpcCommand::ShowBinds).await;
        }

        Commands::QuitPrompt => {
            run_cmd(IpcCommand::ShowQuit).await;
        }

        Commands::SessionMenu => {
            run_cmd(IpcCommand::ShowSessionMenu).await;
        }

        Commands::Window { op } => {
            let action = match op {
                WindowOpArg::Sticky => night_core::ipc::WindowAction::Sticky,
                WindowOpArg::Pip => night_core::ipc::WindowAction::Pip,
                WindowOpArg::Center => night_core::ipc::WindowAction::Center,
                WindowOpArg::Float => night_core::ipc::WindowAction::ToggleFloat,
                WindowOpArg::Fullscreen => night_core::ipc::WindowAction::ToggleFullscreen,
                WindowOpArg::Maximize => night_core::ipc::WindowAction::ToggleMaximize,
                WindowOpArg::Swallow => night_core::ipc::WindowAction::Swallow,
                WindowOpArg::Unswallow => night_core::ipc::WindowAction::Unswallow,
            };
            run_cmd(IpcCommand::WindowOp(action)).await;
        }

        Commands::Close => {
            run_cmd(IpcCommand::CloseFocused).await;
        }

        Commands::Spawn { command } => {
            let cmd_str = command.join(" ");
            run_cmd(IpcCommand::Spawn(cmd_str)).await;
        }

        Commands::Reload => {
            run_cmd(IpcCommand::ReloadConfig).await;
            println!("🌙 Configuration reloaded.");
        }

        Commands::Snip { mode } => {
            match mode {
                None => {
                    run_cmd(IpcCommand::ShowSnipping).await;
                }
                Some(SnipMode::Region) => {
                    let cmd = "wayshot -s \"$(slurp)\" --stdout | satty --filename -".to_string();
                    run_cmd(IpcCommand::Spawn(cmd)).await;
                }
                Some(SnipMode::Window) => {
                    // Triggers the window capture directly through snipping modal
                    run_cmd(IpcCommand::ShowSnipping).await;
                }
                Some(SnipMode::Full) => {
                    let cmd = "mkdir -p ~/Pictures/Screenshots && wayshot -f ~/Pictures/Screenshots/Screenshot_$(date +%Y-%m-%d_%H-%M-%S).png".to_string();
                    run_cmd(IpcCommand::Spawn(cmd)).await;
                    println!("📸 Full screenshot saved to ~/Pictures/Screenshots");
                }
                Some(SnipMode::Copy) => {
                    let cmd = "wayshot --stdout | rsclipd store --mime image/png".to_string();
                    run_cmd(IpcCommand::Spawn(cmd)).await;
                    println!("📋 Full screenshot copied to clipboard (pure-Rust rsclip)");
                }
                Some(SnipMode::Record) => {
                    let cmd = "mkdir -p ~/Videos/Recordings && wl-screenrec -f ~/Videos/Recordings/rec_$(date +%Y-%m-%d_%H-%M-%S).mp4".to_string();
                    run_cmd(IpcCommand::Spawn(cmd)).await;
                    println!("🎥 Recording started: ~/Videos/Recordings (MP4 60fps)");
                }
                Some(SnipMode::Gif) => {
                    let cmd = "mkdir -p ~/Videos/Recordings && wl-screenrec -g \"$(slurp)\" --output-format gif -f ~/Videos/Recordings/rec_$(date +%Y-%m-%d_%H-%M-%S).gif".to_string();
                    run_cmd(IpcCommand::Spawn(cmd)).await;
                    println!("🎞️ Animated GIF recording started: ~/Videos/Recordings");
                }
            }
        }

        Commands::Theme { action } => match action {
            None | Some(ThemeCommand::Status) => match send_command(&IpcCommand::GetTheme).await {
                Ok(IpcResponse::Theme(v)) => {
                    println!("🌙 Active Bihar Night Theme: {} ({})", v.name(), v.bihar_name());
                    println!("   Inspiration: {}", v.inspiration());
                }
                Ok(other) => println!("{other:?}"),
                Err(e) => fatal(e),
            },
            Some(ThemeCommand::Set { variant }) => {
                let v: ThemeVariant = variant.into();
                match send_command(&IpcCommand::SetTheme(v)).await {
                    Ok(IpcResponse::Theme(updated)) => {
                        println!(
                            "🌙 Bihar Night theme switched to: {} ({})",
                            updated.name(),
                            updated.bihar_name()
                        );
                    }
                    Ok(other) => println!("{other:?}"),
                    Err(e) => fatal(e),
                }
            }
            Some(ThemeCommand::Toggle) => match send_command(&IpcCommand::GetTheme).await {
                Ok(IpcResponse::Theme(current)) => {
                    let next = current.next();
                    match send_command(&IpcCommand::SetTheme(next)).await {
                        Ok(IpcResponse::Theme(updated)) => {
                            println!(
                                "🌙 Bihar Night theme toggled to: {} ({})",
                                updated.name(),
                                updated.bihar_name()
                            );
                        }
                        Ok(other) => println!("{other:?}"),
                        Err(e) => fatal(e),
                    }
                }
                Ok(other) => println!("{other:?}"),
                Err(e) => fatal(e),
            },
            Some(ThemeCommand::Schedule { action }) => {
                let act = action.unwrap_or(ScheduleArg::Status);
                match act {
                    ScheduleArg::Status => match send_command(&IpcCommand::GetAutoTheme).await {
                        Ok(IpcResponse::AutoTheme(active)) => {
                            if active {
                                println!("☀️ Diurnal Bihar Night theme schedule: ENABLED (06:00 Nalanda Dawn -> 18:00 Rajgir Mist -> 22:00 Ganga Midnight)");
                            } else {
                                println!("☀️ Diurnal Bihar Night theme schedule: DISABLED (Manual theme control active)");
                            }
                        }
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    },
                    ScheduleArg::On => match send_command(&IpcCommand::SetAutoTheme(true)).await {
                        Ok(IpcResponse::AutoTheme(_)) => {
                            println!("☀️ Diurnal Bihar Night theme schedule: ENABLED");
                        }
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    },
                    ScheduleArg::Off => match send_command(&IpcCommand::SetAutoTheme(false)).await {
                        Ok(IpcResponse::AutoTheme(_)) => {
                            println!("☀️ Diurnal Bihar Night theme schedule: DISABLED");
                        }
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    },
                    ScheduleArg::Toggle => match send_command(&IpcCommand::ToggleAutoTheme).await {
                        Ok(IpcResponse::AutoTheme(active)) => {
                            println!(
                                "☀️ Diurnal Bihar Night theme schedule toggled to: {}",
                                if active { "ENABLED" } else { "DISABLED" }
                            );
                        }
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    },
                }
            }
        },

        Commands::Volume { action } => {
            let act = match action {
                None | Some(VolumeCommand::Status) => VolumeAction::Query,
                Some(VolumeCommand::Raise { delta }) => VolumeAction::Raise(delta.unwrap_or(5)),
                Some(VolumeCommand::Lower { delta }) => VolumeAction::Lower(delta.unwrap_or(5)),
                Some(VolumeCommand::Set { percentage }) => VolumeAction::Set(percentage),
                Some(VolumeCommand::Mute) => VolumeAction::MuteToggle,
            };

            match send_command(&IpcCommand::Volume(act)).await {
                Ok(IpcResponse::Volume(Some(vol))) => {
                    println!("🔊 Audio Volume: {}", vol.display_string());
                }
                Ok(IpcResponse::Volume(None)) => {
                    println!("🔊 Audio service query returned no active default sink.");
                }
                Ok(other) => println!("{other:?}"),
                Err(e) => fatal(e),
            }
        }

        Commands::Power { action } => match action {
            None | Some(PowerCommand::Status) => match send_command(&IpcCommand::GetBattery).await {
                Ok(IpcResponse::Battery(Some(bat))) => {
                    println!("⚡ Battery Status: {} ({})", bat.display_string(), bat.status);
                }
                Ok(IpcResponse::Battery(None)) => {
                    println!("⚡ Running on AC Power (no battery detected).");
                }
                Ok(other) => println!("{other:?}"),
                Err(e) => fatal(e),
            },
            Some(PowerCommand::Saver { action }) => {
                let act = action.unwrap_or(PowerSaverArg::Status);
                match act {
                    PowerSaverArg::Status => match send_command(&IpcCommand::GetPowerSaver).await {
                        Ok(IpcResponse::PowerSaver(active)) => {
                            if active {
                                println!("🔋 Battery conservation mode: ACTIVE (Brightness 30%, OLED Ganga Midnight)");
                            } else {
                                println!("🔋 Battery conservation mode: INACTIVE (Full performance)");
                            }
                        }
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    },
                    PowerSaverArg::On => match send_command(&IpcCommand::SetPowerSaver(true)).await {
                        Ok(IpcResponse::PowerSaver(_)) => {
                            println!("🔋 Battery conservation mode: ENABLED (Brightness 30%, OLED Ganga Midnight)");
                        }
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    },
                    PowerSaverArg::Off => match send_command(&IpcCommand::SetPowerSaver(false)).await {
                        Ok(IpcResponse::PowerSaver(_)) => {
                            println!("🔋 Battery conservation mode: DISABLED (Full performance)");
                        }
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    },
                    PowerSaverArg::Toggle => match send_command(&IpcCommand::TogglePowerSaver).await {
                        Ok(IpcResponse::PowerSaver(active)) => {
                            println!(
                                "🔋 Battery conservation mode toggled to: {}",
                                if active { "ENABLED" } else { "DISABLED" }
                            );
                        }
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    },
                }
            }
        },

        Commands::Launcher => {
            run_cmd(IpcCommand::ShowLauncher).await;
        }

        Commands::Net => match send_command(&IpcCommand::GetNetwork).await {
            Ok(IpcResponse::Network(Some(net))) => {
                println!("🌐 Network Status: {}", net.display_string());
                println!("  Interface       : {}", net.interface);
                println!("  Type            : {:?}", net.kind);
                if let Some(ssid) = net.ssid {
                    println!("  SSID            : {ssid}");
                }
                if let Some(sig) = net.signal_pct {
                    println!("  Signal Quality  : {sig}%");
                }
                if let Some(ip) = net.ip_addr {
                    println!("  Local IPv4      : {ip}");
                }
                println!("  Online          : {}", if net.is_online { "Yes" } else { "No" });
            }
            Ok(IpcResponse::Network(None)) => {
                println!("🌐 Network: Disconnected / No active interface.");
            }
            Ok(other) => println!("{other:?}"),
            Err(e) => fatal(e),
        },

        Commands::Top => match send_command(&IpcCommand::GetSystemStats).await {
            Ok(IpcResponse::SystemStats(s)) => print_system_stats(&s),
            Ok(other) => println!("{other:?}"),
            Err(_) => {
                // IPC not reachable (e.g. compositor not started) - fallback to direct reading
                let s = night_core::sysinfo::read_system_stats();
                print_system_stats(&s);
            }
        },

        Commands::Bt => match send_command(&IpcCommand::GetBluetooth).await {
            Ok(IpcResponse::Bluetooth(Some(bt))) => {
                println!("󰂯 Bluetooth Subsystem: {}", bt.display_string());
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
            Ok(IpcResponse::Bluetooth(None)) => {
                println!("󰂲 Bluetooth: Controller not detected or uninitialized");
            }
            Ok(other) => println!("{other:?}"),
            Err(e) => fatal(e),
        },

        Commands::Sink { action } => match action {
            Some(SinkCommand::List) | None => {
                match send_command(&IpcCommand::GetAudioSinks).await {
                    Ok(IpcResponse::AudioSinks(sinks)) => {
                        println!("🔊 Discovered Audio Output Sinks (PipeWire / PulseAudio):");
                        if sinks.is_empty() {
                            println!("  No active output sinks found.");
                        } else {
                            for sink in sinks {
                                let marker = if sink.is_default { "▶ [ACTIVE]" } else { "          " };
                                println!("  {} [{}] {} {}", marker, sink.id, sink.icon(), sink.name);
                            }
                        }
                    }
                    Ok(other) => println!("{other:?}"),
                    Err(e) => fatal(e),
                }
            }
            Some(SinkCommand::Cycle) => {
                run_cmd(IpcCommand::CycleAudioSink).await;
                println!("🔊 Cycled to next audio output sink");
            }
            Some(SinkCommand::Set { id }) => {
                run_cmd(IpcCommand::SetAudioSink(id)).await;
                println!("🔊 Default audio output sink set to ID: {id}");
            }
            Some(SinkCommand::Menu) => {
                run_cmd(IpcCommand::ShowAudioSinkMenu).await;
            }
        },

        Commands::Brightness { action } => {
            let act = match action {
                Some(BrightnessCommand::Raise { delta }) => {
                    night_core::backlight::BacklightAction::Raise(delta.unwrap_or(5))
                }
                Some(BrightnessCommand::Lower { delta }) => {
                    night_core::backlight::BacklightAction::Lower(delta.unwrap_or(5))
                }
                Some(BrightnessCommand::Set { percentage }) => {
                    night_core::backlight::BacklightAction::Set(percentage)
                }
                Some(BrightnessCommand::Status) | None => night_core::backlight::BacklightAction::Query,
            };

            match send_command(&IpcCommand::Backlight(act)).await {
                Ok(IpcResponse::Backlight(Some(bl))) => {
                    println!("🔆 Display Backlight: {} (device: {})", bl.display_string(), bl.device_name);
                }
                Ok(IpcResponse::Backlight(None)) => {
                    println!("🔆 Display Backlight: No controllable hardware backlight detected.");
                }
                Ok(other) => println!("{other:?}"),
                Err(e) => fatal(e),
            }
        }

        Commands::Dnd { action } => {
            let toggle = match action.as_deref() {
                Some("toggle") | None => true,
                Some("status") => false,
                Some("on") | Some("enable") => {
                    !night_core::notifications::NotificationHistory::is_dnd()
                }
                Some("off") | Some("disable") => {
                    night_core::notifications::NotificationHistory::is_dnd()
                }
                _ => true,
            };

            if toggle {
                match send_command(&IpcCommand::ToggleDnd).await {
                    Ok(IpcResponse::Dnd(dnd)) => {
                        let icon = night_core::notifications::NotificationHistory::glyph_dnd(dnd);
                        println!("{icon} Do Not Disturb: {}", if dnd { "ENABLED (Notifications Muted)" } else { "DISABLED (Normal)" });
                    }
                    Ok(other) => println!("{other:?}"),
                    Err(e) => fatal(e),
                }
            } else {
                match send_command(&IpcCommand::GetDnd).await {
                    Ok(IpcResponse::Dnd(dnd)) => {
                        let icon = night_core::notifications::NotificationHistory::glyph_dnd(dnd);
                        println!("{icon} Do Not Disturb: {}", if dnd { "ENABLED" } else { "DISABLED" });
                    }
                    Ok(other) => println!("{other:?}"),
                    Err(e) => fatal(e),
                }
            }
        }

        Commands::Notify { action } => match action {
            Some(NotifyCommand::List) | None => {
                let history = night_core::notifications::NotificationHistory::load();
                println!("󰂚 Notification History ({} items • DND: {}):", history.items.len(), if history.dnd_enabled { "ON" } else { "OFF" });
                if history.items.is_empty() {
                    println!("  No notifications recorded.");
                } else {
                    for item in history.items.iter().take(10) {
                        println!("  • [{}] {} — {}", item.app_name, item.summary, item.body.replace('\n', " "));
                    }
                }
            }
            Some(NotifyCommand::Clear) => {
                night_core::notifications::NotificationHistory::clear_all();
                println!("󰂚 Notification history cleared.");
            }
            Some(NotifyCommand::Menu) => {
                run_cmd(IpcCommand::ShowNotifications).await;
            }
        },

        Commands::Session { action } => match action {
            SessionCommand::Save => {
                run_cmd(IpcCommand::SaveSession).await;
                println!("💾 Current window session and workspace assignments saved to disk.");
            }
            SessionCommand::Restore => match send_command(&IpcCommand::RestoreSession).await {
                Ok(IpcResponse::SavedSession(Some(s))) => {
                    println!("🔄 Restored window session from timestamp {} ({} windows)", s.timestamp, s.windows.len());
                }
                Ok(IpcResponse::SavedSession(None)) => {
                    println!("🔄 No saved session file found on disk.");
                }
                Ok(other) => println!("{other:?}"),
                Err(e) => fatal(e),
            },
            SessionCommand::Clear => {
                night_core::session_restore::SavedSession::clear();
                println!("🗑️ Saved window session file removed.");
            }
            SessionCommand::Lock => {
                let lock_cmd = night_core::config::UserConfig::load().apps.lock_cmd;
                println!("🔒 Locking desktop session via {lock_cmd}...");
                night_core::exec::spawn_shell(&lock_cmd);
            }
            SessionCommand::Logout => {
                println!("🚪 Initiating graceful desktop session logout...");
                let _ = std::process::Command::new("night-session")
                    .arg("logout")
                    .status();
            }
            SessionCommand::Status => {
                let _ = std::process::Command::new("night-session")
                    .arg("status")
                    .status();
            }
        },

        Commands::Wallpaper { path } => {
            if let Some(p) = path {
                let cmd = format!("swww img \"{p}\" --transition-type wipe --transition-angle 30 --transition-duration 1.5");
                run_cmd(IpcCommand::Spawn(cmd)).await;
                println!("🖼️ Wallpaper updated to: {p}");
            } else {
                let current = std::process::Command::new("swww")
                    .arg("query")
                    .output()
                    .ok()
                    .and_then(|out| {
                        if out.status.success() {
                            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
                        } else {
                            None
                        }
                    });

                if let Some(c) = current {
                    println!("🖼️ Current Wallpaper: {c}");
                } else {
                    println!("🖼️ swww daemon not responding or no wallpaper set.");
                }
            }
        }

        Commands::Log { service, lines, follow } => {
            let path = night_core::logging::log_file_path(&service);
            println!("🌙 Project Night Logs — Service: `{service}`");
            println!("📁 File: {}\n", path.display());

            let recent = night_core::logging::read_recent_logs(&service, lines);
            for line in recent {
                println!("{line}");
            }

            if follow {
                println!("\n📡 Following live logs (Ctrl+C to stop)...");
                let mut last_len = match std::fs::metadata(&path) {
                    Ok(m) => m.len(),
                    Err(_) => 0,
                };
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        let current_len = metadata.len();
                        if current_len > last_len {
                            use std::io::{BufRead, Seek, SeekFrom};
                            if let Ok(mut file) = std::fs::File::open(&path) {
                                if file.seek(SeekFrom::Start(last_len)).is_ok() {
                                    let reader = std::io::BufReader::new(file);
                                    for line in reader.lines().map_while(Result::ok) {
                                        println!("{line}");
                                    }
                                }
                            }
                            last_len = current_len;
                        } else if current_len < last_len {
                            last_len = 0;
                        }
                    }
                }
            }
        }

        Commands::Osd { target } => {
            run_cmd(IpcCommand::ShowOsd(target)).await;
        }

        Commands::Profile { name } => {
            if let Some(n) = name {
                match send_command(&IpcCommand::SetLayoutProfile(n.clone())).await {
                    Ok(IpcResponse::Profile(applied)) => {
                        println!("📐 Applied Layout Profile: {applied}");
                    }
                    Ok(IpcResponse::Error(err)) => {
                        eprintln!("❌ Layout profile error: {err}");
                        exit(1);
                    }
                    Ok(other) => println!("{other:?}"),
                    Err(e) => fatal(e),
                }
            } else {
                println!("📐 Available Bihar Night Workspace Layout Profiles:");
                for p in night_layout::LayoutProfile::all() {
                    println!("  {} {:<14} : {}", p.icon(), p.name(), p.description());
                }
            }
        }
        Commands::Palette => {
            run_cmd(IpcCommand::ShowPalette).await;
        }
        Commands::Overview => {
            run_cmd(IpcCommand::ShowOverview).await;
        }
        Commands::Grid => {
            run_cmd(IpcCommand::ShowGrid).await;
        }
        Commands::Snap { position } => {
            if let Some(pos) = night_core::ipc::SnapPosition::parse(&position) {
                run_cmd(IpcCommand::SnapWindow(pos)).await;
                println!("🪟 Snapped focused window to {}", pos.name());
            } else {
                eprintln!("❌ Unknown snap position '{}'. Valid: center, left, right, top, bottom, top-left, top-right, bottom-left, bottom-right, pip, max", position);
                exit(1);
            }
        }
        Commands::Idle { action } => match action.unwrap_or(IdleCommand::Status) {
            IdleCommand::Status => {
                let is_inh = night_core::gamma::is_inhibited();
                println!("🌙 Project Night Smart Idle Status:");
                println!("   Inhibition Active:  {}", if is_inh { "YES (Suppressing Dim/Lock)" } else { "NO" });
                println!("   Dim Threshold:      300s (5m)");
                println!("   Lock Threshold:     600s (10m)");
                println!("   DPMS Threshold:     900s (15m)");
            }
            IdleCommand::Reset => {
                let _ = send_command(&IpcCommand::Dpms(night_core::ipc::DpmsAction::On)).await;
                println!("✅ Woke displays and signaled user activity.");
            }
        },
        Commands::Border { action } => match action {
            BorderCommand::Toggle => {
                run_cmd(IpcCommand::ToggleDecorations).await;
                println!("🖼️ Window focus border decorations toggled.");
            }
            BorderCommand::Titlebars => {
                run_cmd(IpcCommand::ToggleTitlebars).await;
                println!("🪟 Window titlebars toggled.");
            }
            BorderCommand::Width { width } => {
                run_cmd(IpcCommand::SetBorderWidth(width)).await;
                println!("📏 Window border thickness set to {width}px.");
            }
        },
    }
}

async fn run_cmd(cmd: IpcCommand) {
    match send_command(&cmd).await {
        Ok(IpcResponse::Ok) => (),
        Ok(other) => println!("{other:?}"),
        Err(e) => fatal(e),
    }
}

fn fatal<E: std::fmt::Display>(msg: E) -> ! {
    eprintln!("❌ nightctl error: {msg}");
    exit(1);
}

fn ascii_bar(pct: u8, width: usize) -> String {
    let filled = ((pct as usize * width) / 100).min(width);
    let empty = width.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn print_system_stats(s: &night_core::sysinfo::SystemStats) {
    println!("🌙 Bihar Night System Resource Monitor");
    println!("   CPU Usage    : {:<22} {:>3}%", ascii_bar(s.cpu_pct, 20), s.cpu_pct);
    println!(
        "  󰍛 Memory (RAM) : {:<22} {:>3}% ({:.1} / {:.1} GiB)",
        ascii_bar(s.mem_pct, 20),
        s.mem_pct,
        s.mem_used_mb as f32 / 1024.0,
        s.mem_total_mb as f32 / 1024.0
    );
    println!(
        "  󰋊 Root Disk    : {:<22} {:>3}% ({} / {} GiB)",
        ascii_bar(s.disk_pct, 20),
        s.disk_pct,
        s.disk_used_gb,
        s.disk_total_gb
    );
    if let Some(temp) = s.cpu_temp_c {
        println!("  󰔏 Temperature  : {}°C", temp);
    }
}

fn print_status_table(status: &DesktopStatus) {
    let glyph = status.active_tag_glyph.as_deref().unwrap_or("󰣆");
    let theme = status.theme.as_deref().unwrap_or("Night");
    let bihar_variant = night_core::theme::ThemeVariant::parse_variant(theme);
    let bihar_label = bihar_variant.map(|v| format!("{} ({})", v.name(), v.bihar_name())).unwrap_or_else(|| theme.to_string());
    println!("🌙 Night Desktop Environment");
    println!("  Active Workspace Tag : {} ({glyph})", status.active_tag);
    println!("  Bihar Night Theme    : {bihar_label}");
    println!("  Tiling Layout        : {}", status.layout);
    println!("  Gaps (Inner / Outer) : {:.0}px / {:.0}px", status.inner_gaps, status.outer_gaps);
    println!("  Connected Monitors   : {}", status.output_count);
    println!("  Focused Window       : {}", status.active_window.as_deref().unwrap_or("None"));
    if let Some(ref v) = status.volume {
        println!("  Audio Volume         : {}", v.display_string());
    }
    if let Some(ref b) = status.battery {
        println!("  Battery Power        : {} ({})", b.display_string(), b.status);
    }
    if let Some(ref n) = status.network {
        println!("  Network Connectivity : {}", n.display_string());
    }
    if let Some(ref s) = status.system {
        println!("  System Resources     : CPU: {}% | RAM: {}% ({:.1}/{:.1} GiB)", s.cpu_pct, s.mem_pct, s.mem_used_mb as f32 / 1024.0, s.mem_total_mb as f32 / 1024.0);
    }
    if let Some(ref bt) = status.bluetooth {
        println!("  Bluetooth Status     : {}", bt.display_string());
    }
    if let Some(ref bl) = status.backlight {
        println!("  Display Backlight    : {}", bl.display_string());
    }
    println!("  Do Not Disturb (DND) : {}", if status.dnd { "󰂛 Enabled (Muted)" } else { "󰂚 Disabled (Normal)" });
    println!("  Diurnal Auto-Schedule: {}", if status.auto_theme { "☀️ Enabled (Solar cycle)" } else { "🌑 Disabled (Manual)" });
    println!("  Battery Saver Mode   : {}", if status.power_saver { "🔋 Active (Conservation)" } else { "⚡ Inactive (Full power)" });
    println!("  Night Light (Gamma)  : {}K", status.gamma_temp.unwrap_or(6500));
    println!("  Idle Inhibition Mode : {}", if status.inhibited { "☕ Enabled (Sleep Suspended)" } else { "🌙 Normal Power Saving" });
    if let Some(ref m) = status.media_info {
        println!("  Media Track          : {m}");
    }
    println!("  Last Reload (Epoch)  : {}", status.last_reload);
}
