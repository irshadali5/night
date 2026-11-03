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
        config_dir.join("quickshell/ii/assets/images/default_wallpaper.png"),
        home.join(format!("Pictures/Wallpapers/tokyo-night-{variant_name}.png")),
        home.join("Pictures/Wallpapers/wallpaper.png"),
        home.join("Pictures/Wallpapers/wallpaper.jpg"),
        home.join("Pictures/wallpaper.png"),
        home.join("Pictures/wallpaper.jpg"),
        config_dir.join(format!("swww/tokyo-night-{variant_name}.png")),
        PathBuf::from(format!("/usr/share/backgrounds/night/tokyo-night-{variant_name}.png")),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

/// Synchronizes desktop wallpaper with active Tokyo Night theme variant using `swww` or `swaybg`
pub fn sync_wallpaper_theme(variant: ThemeVariant) {
    let p = variant.palette();
    let hex = p.bg_darkest.trim_start_matches('#');

    // 1. If swww is available, use smooth wipe shaders
    if which::which("swww").is_ok() {
        if let Some(wallpaper_path) = resolve_variant_wallpaper(variant) {
            set_wallpaper(wallpaper_path.to_string_lossy().as_ref());
        } else {
            info!(variant = %variant.name(), hex = %hex, "Clearing wallpaper to Tokyo Night palette base via swww");
            let _ = Command::new("swww")
                .args(["clear", hex])
                .status();
        }
        return;
    }

    // 2. If swaybg is available (standard Wayland background tool), set wallpaper image or solid color
    if which::which("swaybg").is_ok() {
        let _ = Command::new("pkill").args(["-x", "swaybg"]).status();
        if let Some(wallpaper_path) = resolve_variant_wallpaper(variant) {
            info!(variant = %variant.name(), path = ?wallpaper_path, "Setting wallpaper via swaybg");
            night_core::exec::spawn_shell(&format!(
                "swaybg -i \"{}\" -m fill >/dev/null 2>&1 &",
                wallpaper_path.display()
            ));
        } else {
            let color = format!("#{hex}");
            info!(variant = %variant.name(), color = %color, "Setting solid Tokyo Night background via swaybg");
            night_core::exec::spawn_shell(&format!("swaybg -c \"{color}\" >/dev/null 2>&1 &"));
        }
    }
}

/// Applies a specific image wallpaper with a smooth transition (swww or swaybg)
pub fn set_wallpaper(path_str: &str) {
    let path = Path::new(path_str);
    if !path.exists() {
        return;
    }

    if which::which("swww").is_ok() {
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
    } else if which::which("swaybg").is_ok() {
        let _ = Command::new("pkill").args(["-x", "swaybg"]).status();
        info!(path = %path_str, "Applying wallpaper with swaybg");
        night_core::exec::spawn_shell(&format!("swaybg -i \"{path_str}\" -m fill >/dev/null 2>&1 &"));
    }
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

