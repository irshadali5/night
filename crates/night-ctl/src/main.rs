//! 🌙 nightctl: Lightning-fast pure-Rust CLI client and status streaming tool
//! Built with `clap` for typed, declarative CLI parsing.
//! Connects directly to the Night Wayland Compositor via Unix domain sockets.

use clap::{Parser, Subcommand, ValueEnum};
use night_core::ipc::{
    send_command, subscribe_status, DesktopStatus, DpmsAction, IpcCommand, IpcResponse, OutputNav,
};
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

    /// Query or cycle the dynamic tiling layout algorithm
    Layout {
        /// Optional action: pass "next" to cycle to next layout (MasterStack -> Dwindle -> Spiral -> Fair)
        action: Option<String>,
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

    /// Show the Snowcap Tokyo Night interactive visual window switcher
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

    /// Show the Snowcap Tokyo Night session menu modal
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
    /// Open the Snowcap Tokyo Night clipboard history modal
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
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
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

        Commands::Layout { action } => {
            if let Some(act) = action {
                if act == "next" {
                    match send_command(&IpcCommand::CycleLayout).await {
                        Ok(IpcResponse::Layout(l)) => println!("{l}"),
                        Ok(resp) => println!("{resp:?}"),
                        Err(e) => fatal(e),
                    }
                } else {
                    fatal(format!("Unknown layout action '{act}'. Use 'nightctl layout next' or omit argument to query."));
                }
            } else {
                match send_command(&IpcCommand::GetLayout).await {
                    Ok(IpcResponse::Layout(l)) => println!("{l}"),
                    Ok(resp) => println!("{resp:?}"),
                    Err(e) => fatal(e),
                }
            }
        }

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
                                    let clean = item.replace('\n', " ").replace('\t', " ");
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
    }
}

async fn run_cmd(cmd: IpcCommand) {
    match send_command(&cmd).await {
        Ok(IpcResponse::Ok) => (),
        Ok(other) => println!("{other:?}"),
        Err(e) => fatal(e),
    }
}

fn fatal(msg: String) -> ! {
    eprintln!("❌ nightctl error: {msg}");
    exit(1);
}

fn print_status_table(status: &DesktopStatus) {
    println!("🌙 Night Desktop Environment");
    println!("  Active Workspace Tag : {}", status.active_tag);
    println!("  Tiling Layout        : {}", status.layout);
    println!("  Gaps (Inner / Outer) : {:.0}px / {:.0}px", status.inner_gaps, status.outer_gaps);
    println!("  Connected Monitors   : {}", status.output_count);
    println!("  Focused Window       : {}", status.active_window.as_deref().unwrap_or("None"));
    println!("  Night Light (Gamma)  : {}K", status.gamma_temp.unwrap_or(6500));
    println!("  Idle Inhibition Mode : {}", if status.inhibited { "☕ Enabled (Sleep Suspended)" } else { "🌙 Normal Power Saving" });
    println!("  Last Reload (Epoch)  : {}", status.last_reload);
}
