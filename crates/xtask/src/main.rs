//! 🌙 Project Night - xtask Automation Runner
//! Native Rust workspace automation replacing Makefiles and shell scripts.
//! Uses standard `dirs` crate for cross-platform XDG resolution.
//! Usage: `cargo xtask <command>`

use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

fn main() {
    let mut args = env::args().skip(1);
    let task = args.next().unwrap_or_else(|| "help".to_string());

    let root_dir = project_root();

    match task.as_str() {
        "install" => task_install(&root_dir),
        "check" => task_check(&root_dir),
        "build" => task_build(&root_dir),
        "run" => task_run(&root_dir),
        "uninstall" => task_uninstall(&root_dir),
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
        .nth(2)
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

    println!("🎉 Night successfully installed! Launch your compositor by typing: pinnacle");
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

fn print_help() {
    println!(
        r#"🌙 Night Workspace Task Runner (cargo xtask)

USAGE:
    cargo xtask <SUBCOMMAND>

SUBCOMMANDS:
    install      Builds release binary, copies to ~/.cargo/bin, and links pinnacle.toml
    check        Runs `cargo check` across all workspace crates
    build        Builds `night-pinnacle` in release mode
    run          Builds and launches the Pinnacle compositor
    uninstall    Removes the installed binary and config symlink
    help         Prints this help message
"#
    );
}
