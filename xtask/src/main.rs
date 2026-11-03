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
    println!("🔨 Building `night-pinnacle` in release mode...");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "night-pinnacle"])
        .current_dir(root)
        .status()
        .expect("Failed to run cargo build");

    if !status.success() {
        exit(status.code().unwrap_or(1));
    }
    println!("✅ Build complete: target/release/night-pinnacle");
}

fn task_install(root: &Path) {
    println!("🌙 Installing Project Night...");

    // 1. Compile in release mode
    task_build(root);

    let home = dirs::home_dir().expect("Could not determine user home directory");
    let cargo_bin = home.join(".cargo/bin");
    fs::create_dir_all(&cargo_bin).expect("Failed to create ~/.cargo/bin");

    // 2. Install binary to ~/.cargo/bin
    let source_bin = root.join("target/release/night-pinnacle");
    let target_bin = cargo_bin.join("night-pinnacle");

    println!("==> Copying binary to {}", target_bin.display());
    fs::copy(&source_bin, &target_bin).expect("Failed to copy binary to ~/.cargo/bin");

    // 3. Link pinnacle.toml to ~/.config/pinnacle/pinnacle.toml using `dirs::config_dir()`
    let config_dir = dirs::config_dir()
        .expect("Could not determine config directory")
        .join("pinnacle");
    fs::create_dir_all(&config_dir).expect("Failed to create ~/.config/pinnacle");

    let source_toml = root.join("pinnacle.toml");
    let target_toml = config_dir.join("pinnacle.toml");

    let _ = fs::remove_file(&target_toml);
    println!("==> Symlinking {} -> {}", source_toml.display(), target_toml.display());
    symlink(&source_toml, &target_toml).expect("Failed to create symlink for pinnacle.toml");

    // 4. Setup default user configurations (~/.config/night/)
    let night_user_dir = dirs::config_dir()
        .expect("Could not determine config directory")
        .join("night");
    fs::create_dir_all(&night_user_dir).expect("Failed to create ~/.config/night");

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

    println!("🎉 Night successfully installed! Launch your compositor by typing: pinnacle");
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

    let bin = home.join(".cargo/bin/night-pinnacle");
    if bin.exists() {
        let _ = fs::remove_file(&bin);
        println!("Removed {}", bin.display());
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
        other => {
            eprintln!("Unknown IPC command: '{other}'");
            exit(1);
        }
    };

    println!("📡 Sending IPC command to Night: {cmd:?}");
    match night_core::ipc::send_command(&cmd).await {
        Ok(resp) => println!("✅ Response: {resp:?}"),
        Err(e) => {
            eprintln!("❌ Failed to deliver command: {e}");
            exit(1);
        }
    }
}

fn print_help() {
    println!(
        r#"🌙 Night Workspace Task Runner (cargo xtask)

USAGE:
    cargo xtask <SUBCOMMAND>

SUBCOMMANDS:
    install      Builds release binary, copies to ~/.cargo/bin, links pinnacle.toml, deploys theme
    theme        Generates Tokyo Night configurations for external suite (Alacritty, Ironbar, Anyrun, Veila)
    check        Runs `cargo check` across all workspace crates
    build        Builds `night-pinnacle` in release mode
    run          Builds and launches the Pinnacle compositor
    msg <CMD>    Sends IPC command to running Night (reload, switch-tag, state, layout, gaps, spawn, close)
    uninstall    Removes the installed binary and config symlink
    help         Prints this help message
"#
    );
}
