//! Asynchronous process spawning, PATH validation (`which`), and safe argument tokenization (`shell-words`).

use pinnacle_api::process;
use tracing::{error, info, warn};

/// Spawns a binary directly, verifying its existence in PATH via `which`
pub fn spawn(program: &str, args: &[&str]) {
    if which::which(program).is_err() {
        warn!(program = %program, "Binary not found in PATH; skipping spawn");
        return;
    }

    info!(program = %program, args = ?args, "Spawning process");
    if process::Command::new(program).args(args).spawn().is_none() {
        error!(program = %program, "Failed to spawn process");
    }
}

/// Spawns a command string. Uses `shell-words` for safe tokenization, or `/bin/sh` if shell features are needed.
pub fn spawn_shell(command_str: &str) {
    // If command uses pipes, redirection, or chaining, delegate to sh
    if command_str.contains('|') || command_str.contains('&') || command_str.contains('>') {
        info!(command = %command_str, "Spawning via shell interpreter");
        if process::Command::new("sh").args(["-c", command_str]).spawn().is_none() {
            error!(command = %command_str, "Failed to spawn shell pipeline");
        }
        return;
    }

    // Otherwise, safely tokenize arguments using the battle-tested `shell-words` crate
    match shell_words::split(command_str) {
        Ok(words) if !words.is_empty() => {
            let prog = &words[0];
            let args: Vec<&str> = words[1..].iter().map(|s| s.as_str()).collect();
            spawn(prog, &args);
        }
        _ => {
            // Fallback to sh if tokenization fails
            let _ = process::Command::new("sh").args(["-c", command_str]).spawn();
        }
    }
}
