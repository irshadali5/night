//! 🌙 Tokyo Night Wallpaper Synchronization Engine
//! Coordinates seamless wallpaper transitions with `swww` matching the active Tokyo Night theme variant.

use night_core::theme::ThemeVariant;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

/// Locates or suggests wallpaper for a specific Tokyo Night theme variant
pub fn resolve_variant_wallpaper(variant: ThemeVariant) -> Option<PathBuf> {
    let variant_name = variant.name().to_lowercase();
    let home = dirs::home_dir()?;
    let config_dir = dirs::config_dir().unwrap_or_else(|| home.join(".config"));

    let candidates = [
        config_dir.join(format!("night/wallpapers/tokyo-night-{variant_name}.png")),
        config_dir.join(format!("night/wallpapers/tokyo-night-{variant_name}.jpg")),
        config_dir.join(format!("swww/tokyo-night-{variant_name}.png")),
        home.join(format!("Pictures/Wallpapers/tokyo-night-{variant_name}.png")),
        PathBuf::from(format!("/usr/share/backgrounds/night/tokyo-night-{variant_name}.png")),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

/// Synchronizes desktop wallpaper with active Tokyo Night theme variant using `swww`
pub fn sync_wallpaper_theme(variant: ThemeVariant) {
    if which::which("swww").is_err() {
        return;
    }

    if let Some(wallpaper_path) = resolve_variant_wallpaper(variant) {
        set_wallpaper(wallpaper_path.to_string_lossy().as_ref());
    } else {
        // Fallback: clear with variant base background color
        let p = variant.palette();
        let hex = p.bg_darkest.trim_start_matches('#');
        info!(variant = %variant.name(), hex = %hex, "Clearing wallpaper to Tokyo Night palette base");
        let _ = Command::new("swww")
            .args(["clear", hex])
            .status();
    }
}

/// Applies a specific image wallpaper with a smooth wipe transition via `swww`
pub fn set_wallpaper(path_str: &str) {
    let path = Path::new(path_str);
    if !path.exists() {
        return;
    }

    info!(path = %path_str, "Applying wallpaper with swww transition");
    let _ = Command::new("swww")
        .args([
            "img",
            path_str,
            "--transition-type",
            "wipe",
            "--transition-angle",
            "30",
            "--transition-duration",
            "1.5",
        ])
        .status();
}

/// Queries currently applied wallpaper path via `swww query`
pub fn current_wallpaper() -> Option<String> {
    if which::which("swww").is_ok() {
        if let Ok(out) = Command::new("swww").arg("query").output() {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // swww query outputs lines like: "DP-1: /path/to/wallpaper.png"
                if let Some(line) = stdout.lines().next() {
                    if let Some(path) = line.split(": ").nth(1) {
                        return Some(path.trim().to_string());
                    }
                }
            }
        }
    }

    None
}

