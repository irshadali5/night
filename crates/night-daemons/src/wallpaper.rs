//! 🌙 Bihar Night Wallpaper Synchronization Engine
//! Coordinates seamless wallpaper transitions with `swww` matching the active Bihar Night theme variant.

use night_core::theme::ThemeVariant;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

/// Locates or suggests wallpaper for a specific Bihar Night theme variant
pub fn resolve_variant_wallpaper(variant: ThemeVariant) -> Option<PathBuf> {
    let variant_name = variant.name().to_lowercase();
    let home = dirs::home_dir()?;
    let config_dir = dirs::config_dir().unwrap_or_else(|| home.join(".config"));

    let candidates = [
        config_dir.join(format!("night/wallpapers/bihar-night-{variant_name}.jpg")),
        config_dir.join(format!("night/wallpapers/bihar-night-{variant_name}.png")),
        config_dir.join("night/wallpapers/bihar-night.jpg"),
        config_dir.join("night/wallpapers/bihar-night-ganga.jpg"),
        config_dir.join("night/wallpapers/bihar-night-bodhgaya.jpg"),
        home.join(format!("Pictures/Wallpapers/bihar-night-{variant_name}.jpg")),
        home.join("Pictures/Wallpapers/bihar-night.jpg"),
        home.join(format!("Pictures/bihar-night-{variant_name}.jpg")),
        home.join("Pictures/bihar-night.jpg"),
        PathBuf::from(format!("assets/wallpapers/bihar-night-{variant_name}.jpg")),
        PathBuf::from("assets/wallpapers/bihar-night.jpg"),
        PathBuf::from("assets/wallpapers/bihar-night-ganga.jpg"),
        PathBuf::from("assets/wallpapers/bihar-night-bodhgaya.jpg"),
        config_dir.join(format!("night/wallpapers/tokyo-night-{variant_name}.png")),
        config_dir.join(format!("night/wallpapers/tokyo-night-{variant_name}.jpg")),
        config_dir.join("quickshell/ii/assets/images/default_wallpaper.png"),
        home.join("Pictures/Wallpapers/wallpaper.png"),
        home.join("Pictures/Wallpapers/wallpaper.jpg"),
        home.join("Pictures/wallpaper.png"),
        home.join("Pictures/wallpaper.jpg"),
        config_dir.join(format!("swww/bihar-night-{variant_name}.png")),
        PathBuf::from(format!("/usr/share/backgrounds/night/bihar-night-{variant_name}.jpg")),
    ];

    candidates.into_iter().find(|candidate| candidate.exists())
}

/// Pluggable Wayland wallpaper backend (`swww`, `swaybg`, etc.).
pub trait WallpaperBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn set_image(&self, path: &str);
    fn set_color(&self, hex: &str);
    fn current_image(&self) -> Option<String> {
        None
    }
}

/// Smooth-transition wallpaper backend powered by `swww`.
pub struct SwwwBackend;

impl WallpaperBackend for SwwwBackend {
    fn name(&self) -> &'static str {
        "swww"
    }

    fn set_image(&self, path: &str) {
        info!(path = %path, "Applying wallpaper with swww transition");
        let _ = Command::new("swww")
            .args([
                "img",
                path,
                "--transition-type",
                "wipe",
                "--transition-angle",
                "30",
                "--transition-duration",
                "1.5",
            ])
            .status();
    }

    fn set_color(&self, hex: &str) {
        info!(hex = %hex, "Clearing wallpaper to Bihar Night palette base via swww");
        let _ = Command::new("swww").args(["clear", hex]).status();
    }

    fn current_image(&self) -> Option<String> {
        let out = Command::new("swww").arg("query").output().ok()?;
        if !out.status.success() {
            return None;
        }
        let stdout = String::from_utf8_lossy(&out.stdout);
        // swww query outputs lines like: "DP-1: /path/to/wallpaper.png"
        stdout
            .lines()
            .next()
            .and_then(|line| line.split(": ").nth(1))
            .map(|p| p.trim().to_string())
    }
}

/// Fallback static Wayland wallpaper backend powered by `swaybg`.
pub struct SwaybgBackend;

impl WallpaperBackend for SwaybgBackend {
    fn name(&self) -> &'static str {
        "swaybg"
    }

    fn set_image(&self, path: &str) {
        let _ = Command::new("pkill").args(["-x", "swaybg"]).status();
        info!(path = %path, "Applying wallpaper with swaybg");
        night_core::exec::spawn("swaybg", &["-i", path, "-m", "fill"]);
    }

    fn set_color(&self, hex: &str) {
        let _ = Command::new("pkill").args(["-x", "swaybg"]).status();
        let color = format!("#{hex}");
        info!(color = %color, "Setting solid Bihar Night background via swaybg");
        night_core::exec::spawn("swaybg", &["-c", &color]);
    }
}

/// Resolves the highest-priority available wallpaper backend on `PATH`.
pub fn resolve_wallpaper_backend() -> Option<Box<dyn WallpaperBackend>> {
    if which::which("swww").is_ok() {
        Some(Box::new(SwwwBackend))
    } else if which::which("swaybg").is_ok() {
        Some(Box::new(SwaybgBackend))
    } else {
        None
    }
}

/// Synchronizes desktop wallpaper with active Bihar Night theme variant
pub fn sync_wallpaper_theme(variant: ThemeVariant) {
    let Some(backend) = resolve_wallpaper_backend() else {
        return;
    };
    let p = variant.palette();
    let hex = p.bg_darkest.trim_start_matches('#');

    if let Some(wallpaper_path) = resolve_variant_wallpaper(variant) {
        backend.set_image(wallpaper_path.to_string_lossy().as_ref());
    } else {
        backend.set_color(hex);
    }
}

/// Applies a specific image wallpaper using the active backend
pub fn set_wallpaper(path_str: &str) {
    if !Path::new(path_str).exists() {
        return;
    }
    if let Some(backend) = resolve_wallpaper_backend() {
        backend.set_image(path_str);
    }
}

/// Queries currently applied wallpaper path from the active backend
pub fn current_wallpaper() -> Option<String> {
    resolve_wallpaper_backend().and_then(|backend| backend.current_image())
}

