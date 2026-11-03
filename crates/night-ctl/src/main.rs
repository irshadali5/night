//! 🌙 nightctl: Lightning-fast pure-Rust CLI client and status streaming tool
//! Connects directly to the Night Wayland Compositor via Unix domain sockets.
//! Provides instant remote control and zero-allocation event streaming for status bars.

use night_core::ipc::{send_command, subscribe_status, DesktopStatus, IpcCommand, IpcResponse};
use std::env;
use std::io::{self, Write};
use std::process::exit;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
        "status" => {
            let json_mode = args.iter().any(|a| a == "--json" || a == "-j");
            match send_command(&IpcCommand::GetStatus).await {
                Ok(IpcResponse::Status(status)) => {
                    if json_mode {
                        println!("{}", status.to_json());
                    } else {
                        print_status_table(&status);
                    }
                }
                Ok(other) => {
                    eprintln!("Unexpected response: {other:?}");
                    exit(1);
                }
                Err(e) => {
                    eprintln!("❌ Error communicating with Night compositor: {e}");
                    exit(1);
                }
            }
        }

        "stream" | "watch" | "subscribe" => {
            let json_mode = args.iter().any(|a| a == "--json" || a == "-j");
            let format_flag = args
                .windows(2)
                .find(|w| w[0] == "--format" || w[0] == "-f")
                .map(|w| w[1].clone());

            let result = subscribe_status(move |status| {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                if json_mode {
                    let _ = writeln!(handle, "{}", status.to_json());
                } else if let Some(ref tmpl) = format_flag {
                    let _ = writeln!(handle, "{}", status.format_template(tmpl));
                } else {
                    let win = status.active_window.as_deref().unwrap_or("Desktop");
                    let _ = writeln!(handle, "[{}] Tag {} | {}", status.layout, status.active_tag, win);
                }
                let _ = handle.flush();
            })
            .await;

            if let Err(e) = result {
                eprintln!("❌ Event stream disconnected: {e}");
                exit(1);
            }
        }

        "tag" | "switch-tag" => {
            if args.len() < 2 {
                eprintln!("Usage: nightctl tag <NAME> (e.g. `nightctl tag 2`)");
                exit(1);
            }
            let tag_name = args[1].clone();
            run_cmd(IpcCommand::SwitchTag(tag_name)).await;
        }

        "layout" => {
            if args.len() > 1 && args[1] == "next" {
                match send_command(&IpcCommand::CycleLayout).await {
                    Ok(IpcResponse::Layout(l)) => println!("{l}"),
                    Ok(resp) => println!("{resp:?}"),
                    Err(e) => fatal(e),
                }
            } else {
                match send_command(&IpcCommand::GetLayout).await {
                    Ok(IpcResponse::Layout(l)) => println!("{l}"),
                    Ok(resp) => println!("{resp:?}"),
                    Err(e) => fatal(e),
                }
            }
        }

        "gaps" => {
            if args.len() < 2 {
                eprintln!("Usage: nightctl gaps <DELTA> (e.g. `nightctl gaps +2` or `nightctl gaps -2`)");
                exit(1);
            }
            let delta: f32 = args[1].parse().unwrap_or_else(|_| {
                eprintln!("Invalid gaps delta: '{}'", args[1]);
                exit(1);
            });
            run_cmd(IpcCommand::AdjustGaps(delta)).await;
        }

        "scratchpad" => {
            run_cmd(IpcCommand::ToggleScratchpad).await;
        }

        "reload" => {
            run_cmd(IpcCommand::ReloadConfig).await;
            println!("🌙 Configuration reloaded.");
        }

        "close" => {
            run_cmd(IpcCommand::CloseFocused).await;
        }

        "spawn" => {
            if args.len() < 2 {
                eprintln!("Usage: nightctl spawn <COMMAND...>");
                exit(1);
            }
            let cmd_str = args[1..].join(" ");
            run_cmd(IpcCommand::Spawn(cmd_str)).await;
        }

        "overlay" | "binds" => {
            run_cmd(IpcCommand::ShowBinds).await;
        }

        "quit-prompt" | "prompt-quit" => {
            run_cmd(IpcCommand::ShowQuit).await;
        }

        "help" | "-h" | "--help" => {
            print_help();
        }

        unknown => {
            eprintln!("Unknown nightctl command: '{unknown}'. Run `nightctl help` for available commands.");
            exit(1);
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
    println!("  Last Reload (Epoch)  : {}", status.last_reload);
}

fn print_help() {
    println!(
        r#"🌙 nightctl: Remote Control & Streaming Client for Night Wayland Desktop

USAGE:
    nightctl <COMMAND> [OPTIONS]

COMMANDS:
    status [--json]                 Query current desktop status (workspace, layout, window)
    stream [--json] [--format FMT]  Stream real-time status updates (ideal for status bars)
    tag <NAME>                      Switch active workspace tag (e.g. `nightctl tag 2`)
    layout [next]                   Query or cycle tiling layout (MasterStack, Spiral, Dwindle, Fair)
    gaps <DELTA>                    Adjust tiling gaps (e.g. `nightctl gaps +2` or `nightctl gaps -2`)
    scratchpad                      Toggle Quake dropdown terminal
    overlay                         Show Snowcap interactive keybindings cheatsheet modal
    quit-prompt                     Show Snowcap quit confirmation prompt
    close                           Close the currently focused window
    spawn <COMMAND...>              Spawn a command through the compositor
    reload                          Reload the compositor configuration
    help                            Show this help message

STREAM TEMPLATE VARIABLES:
    {{layout}}          Active tiling layout algorithm name
    {{active_tag}}      Current active tag name (e.g. 1)
    {{active_window}}   Focused window title and application ID
    {{inner_gaps}}      Inner gap size in pixels
    {{outer_gaps}}      Outer gap size in pixels
    {{output_count}}    Number of active connected monitors

EXAMPLES:
    nightctl status --json
    nightctl stream --format "󰕰 {{layout}} | 󱂬 {{active_tag}} | {{active_window}}"
    nightctl layout next
    nightctl gaps +2
    nightctl scratchpad
"#
    );
}
