//! 🌙 Asynchronous process spawning, PATH validation (`which`), and safe argument tokenization (`shell-words`).
//! Decoupled execution architecture guaranteeing reliable launching across Wayland, Pinnacle compositor,
//! Slint GUI control centers, and standalone background daemons.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tracing::{debug, error, info, warn};

/// Ensures essential Wayland and Cargo environment variables are initialized for all child processes.
/// Automatically prepends `~/.cargo/bin` and `~/.local/bin` to `PATH` if not already present,
/// and auto-detects `WAYLAND_DISPLAY` if absent from the current process environment.
pub fn ensure_desktop_env() {
    // 1. Augment PATH with standard Cargo and local user bin directories
    if let Ok(current_path) = env::var("PATH") {
        let mut dirs_to_prepend: Vec<PathBuf> = Vec::new();

        if let Some(home) = dirs::home_dir() {
            let cargo_bin = home.join(".cargo/bin");
            if cargo_bin.is_dir() && !current_path.split(':').any(|p| Path::new(p) == cargo_bin) {
                dirs_to_prepend.push(cargo_bin);
            }
            let local_bin = home.join(".local/bin");
            if local_bin.is_dir() && !current_path.split(':').any(|p| Path::new(p) == local_bin) {
                dirs_to_prepend.push(local_bin);
            }
        }

        let usr_local_bin = PathBuf::from("/usr/local/bin");
        if usr_local_bin.is_dir() && !current_path.split(':').any(|p| Path::new(p) == usr_local_bin) {
            dirs_to_prepend.push(usr_local_bin);
        }

        if !dirs_to_prepend.is_empty() {
            let prepended = dirs_to_prepend
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(":");
            let new_path = format!("{prepended}:{current_path}");
            unsafe {
                env::set_var("PATH", &new_path);
            }
            debug!(path = %new_path, "Augmented process PATH with Cargo/user bin directories");
        }
    }

    // 2. Auto-detect WAYLAND_DISPLAY if unset
    if env::var("WAYLAND_DISPLAY").is_err() {
        if let Some(runtime_dir) = dirs::runtime_dir() {
            for num in 0..=8 {
                let candidate = format!("wayland-{num}");
                if runtime_dir.join(&candidate).exists() {
                    unsafe {
                        env::set_var("WAYLAND_DISPLAY", &candidate);
                    }
                    info!(socket = %candidate, "Auto-detected and exported WAYLAND_DISPLAY");
                    break;
                }
            }
        }
    }

    // 3. Ensure XDG desktop and session variables are present
    if env::var("XDG_CURRENT_DESKTOP").is_err() {
        unsafe {
            env::set_var("XDG_CURRENT_DESKTOP", "pinnacle:night");
        }
    }
    if env::var("XDG_SESSION_TYPE").is_err() {
        unsafe {
            env::set_var("XDG_SESSION_TYPE", "wayland");
        }
    }
}

/// Resolves an executable binary to an absolute path, searching PATH and cargo/system directories
pub fn resolve_executable(program: &str) -> Option<PathBuf> {
    ensure_desktop_env();

    // Direct path specified (e.g. /usr/bin/firefox or ./target/release/night-settings)
    let p = Path::new(program);
    if p.is_file() {
        return Some(p.to_path_buf());
    }

    // Standard PATH lookup via which
    if let Ok(resolved) = which::which(program) {
        return Some(resolved);
    }

    // Lookups in common directories
    if let Some(home) = dirs::home_dir() {
        let cargo_bin = home.join(".cargo/bin").join(program);
        if cargo_bin.is_file() {
            return Some(cargo_bin);
        }
        let local_bin = home.join(".local/bin").join(program);
        if local_bin.is_file() {
            return Some(local_bin);
        }
    }

    // Current working directory target/release or target/debug
    for sub in &["target/release", "target/debug"] {
        let cand = Path::new(sub).join(program);
        if cand.is_file() {
            if let Ok(canon) = fs::canonicalize(&cand) {
                return Some(canon);
            }
            return Some(cand);
        }
    }

    // Check system directories
    for sys in &["/usr/local/bin", "/usr/bin", "/bin"] {
        let cand = Path::new(sys).join(program);
        if cand.is_file() {
            return Some(cand);
        }
    }

    None
}

/// Spawns a binary directly, verifying its existence and isolating its standard I/O.
/// Tries Pinnacle compositor process manager when running inside Pinnacle,
/// with seamless, resilient fallback to `std::process::Command` detached process execution.
pub fn spawn(program: &str, args: &[&str]) {
    ensure_desktop_env();

    let resolved_path = match resolve_executable(program) {
        Some(path) => path,
        None => {
            warn!(
                program = %program,
                "Binary '{}' could not be resolved in PATH or ~/.cargo/bin; skipping spawn",
                program
            );
            return;
        }
    };

    let prog_str = resolved_path.to_string_lossy().to_string();
    info!(program = %program, path = %prog_str, args = ?args, "Spawning desktop process");

    // Attempt to spawn via Pinnacle API if running in compositor context
    let pinnacle_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pinnacle_api::process::Command::new(&prog_str)
            .args(args)
            .spawn()
    }));

    if let Ok(Some(_child)) = pinnacle_result {
        debug!(program = %program, "Successfully spawned process via Pinnacle compositor manager");
        return;
    }

    // Fallback: spawn decoupled child process via standard library
    let mut cmd = std::process::Command::new(&resolved_path);
    cmd.args(args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());

    // Pass all current environment variables (including WAYLAND_DISPLAY and augmented PATH)
    cmd.envs(env::vars());

    match cmd.spawn() {
        Ok(child) => {
            info!(
                program = %program,
                pid = child.id(),
                "Successfully spawned process via decoupled system launcher"
            );
        }
        Err(e) => {
            error!(
                program = %program,
                path = %prog_str,
                error = %e,
                "Failed to spawn process"
            );
        }
    }
}

/// Spawns a command string safely. Uses `shell-words` for argument tokenization,
/// or `/bin/sh` if shell features (pipes, redirection, backgrounding) are required.
pub fn spawn_shell(command_str: &str) {
    ensure_desktop_env();

    // If command uses shell syntax (pipes, redirection, chaining), delegate to sh
    if command_str.contains('|') || command_str.contains('&') || command_str.contains('>') || command_str.contains('<') {
        info!(command = %command_str, "Spawning via shell interpreter (/bin/sh -c)");

        let pinnacle_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            pinnacle_api::process::Command::new("sh")
                .args(["-c", command_str])
                .spawn()
        }));

        if let Ok(Some(_child)) = pinnacle_result {
            return;
        }

        // Decoupled sh execution
        let mut cmd = std::process::Command::new("sh");
        cmd.args(["-c", command_str]);
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        cmd.envs(env::vars());

        if let Err(e) = cmd.spawn() {
            error!(command = %command_str, error = %e, "Failed to spawn shell command");
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
            let mut cmd = std::process::Command::new("sh");
            cmd.args(["-c", command_str]);
            cmd.stdin(Stdio::null());
            cmd.stdout(Stdio::null());
            cmd.stderr(Stdio::null());
            cmd.envs(env::vars());
            let _ = cmd.spawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensure_desktop_env() {
        ensure_desktop_env();
        assert!(env::var("PATH").is_ok());
        assert!(env::var("XDG_CURRENT_DESKTOP").is_ok());
        assert!(env::var("XDG_SESSION_TYPE").is_ok());
    }

    #[test]
    fn test_resolve_executable() {
        ensure_desktop_env();
        let sh = resolve_executable("sh");
        assert!(sh.is_some());
        assert!(sh.unwrap().is_file());
    }

    #[test]
    fn test_resolve_nonexistent() {
        let none = resolve_executable("nonexistent_binary_xyz_123");
        assert!(none.is_none());
    }
}
