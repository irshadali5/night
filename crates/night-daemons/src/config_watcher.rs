//! 🌙 Configuration Auto-Reloading Supervisor
//! Polls modification timestamps of `night.toml`, `rules.ron`, and `monitors.ron`,
//! automatically reloading window rules and broadcasting updated status without restarting Pinnacle.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use tracing::info;

fn get_mtime(path: &PathBuf) -> Option<SystemTime> {
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Watches configuration files for modifications and triggers hot-reloads
pub async fn supervise_config_watcher() {
    let Some(config_dir) = dirs::config_dir().map(|d| d.join("night")) else {
        return;
    };

    let rules_path = config_dir.join("rules.ron");
    let monitors_path = config_dir.join("monitors.ron");
    let config_path = config_dir.join("night.toml");

    let mut last_rules_mtime = get_mtime(&rules_path);
    let mut last_monitors_mtime = get_mtime(&monitors_path);
    let mut last_config_mtime = get_mtime(&config_path);

    info!("Starting background configuration file auto-reloading supervisor");

    loop {
        sleep(Duration::from_millis(1500)).await;

        // 1. Check rules.ron
        if let Some(mtime) = get_mtime(&rules_path) {
            if last_rules_mtime.map(|prev| mtime > prev).unwrap_or(false) {
                info!("Detected modifications to rules.ron; hot-reloading window rules...");
                night_rules::setup_rules();
            }
            last_rules_mtime = Some(mtime);
        }

        // 2. Check monitors.ron
        if let Some(mtime) = get_mtime(&monitors_path) {
            if last_monitors_mtime.map(|prev| mtime > prev).unwrap_or(false) {
                info!("Detected modifications to monitors.ron; hot-reloading display profiles...");
                let profiles = night_core::monitors::MonitorProfile::load_all();
                pinnacle_api::output::for_each_output(move |op| {
                    let name = op.name();
                    if let Some(profile) = profiles.iter().find(|p| p.name == name) {
                        profile.apply_to(op);
                    } else if let Some(fallback) = profiles.iter().find(|p| p.name == "*") {
                        fallback.apply_to(op);
                    }
                });
            }
            last_monitors_mtime = Some(mtime);
        }

        // 3. Check night.toml
        if let Some(mtime) = get_mtime(&config_path) {
            if last_config_mtime.map(|prev| mtime > prev).unwrap_or(false) {
                info!("Detected modifications to night.toml; reloading user preferences...");
                let cfg = night_core::UserConfig::load();
                crate::wallpaper::sync_wallpaper_theme(cfg.general.theme);
                cfg.general.theme.sync_walker_theme();
                if cfg.gamma.enabled {
                    night_core::gamma::set_temperature(cfg.gamma.night_temp);
                }
            }
            last_config_mtime = Some(mtime);
        }
    }
}

