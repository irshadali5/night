//! 🌙 Bihar Night Diurnal Theme Scheduler
//! Clock- and solar-driven automatic transition between Bihar Night theme expressions:
//! - Dawn / Day (06:00 - 18:00): Nalanda Dawn (Sandstone Light, 6500K)
//! - Twilight (18:00 - 22:00): Rajgir Mist (Slate Basalt, 4500K)
//! - Midnight (22:00 - 06:00): Ganga Midnight (Deep River Obsidian, 3400K)

use chrono::{Local, Timelike};
use night_core::config::UserConfig;
use night_core::ipc::{send_command, IpcCommand};
use night_core::notifications::NotificationHistory;
use night_core::theme::ThemeVariant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

static DIURNAL_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Returns whether automatic diurnal theme scheduling is currently active
pub fn is_diurnal_active() -> bool {
    DIURNAL_ACTIVE.load(Ordering::Relaxed)
}

/// Sets whether automatic diurnal theme scheduling is active
pub fn set_diurnal_active(active: bool) {
    DIURNAL_ACTIVE.store(active, Ordering::Relaxed);
    let mut cfg = UserConfig::load();
    cfg.general.auto_theme_schedule = active;
    let _ = cfg.save();
    info!(active = active, "Diurnal theme scheduler state updated");
}

/// Resolves the optimal theme variant and color temperature for a given hour
pub fn resolve_diurnal_phase(hour: u32) -> (ThemeVariant, u32, &'static str) {
    match hour {
        6..=17 => (
            ThemeVariant::Day,
            6500,
            "Nalanda Dawn (Sun-warmed terracotta & daylight)",
        ),
        18..=21 => (
            ThemeVariant::Storm,
            4500,
            "Rajgir Mist (Misty basalt hills & twilight warm)",
        ),
        _ => (
            ThemeVariant::Night,
            3400,
            "Ganga Midnight (Deep river obsidian & late-night serenity)",
        ),
    }
}

/// Background supervision loop for diurnal theme transitions
pub async fn supervise_diurnal_schedule() {
    let initial_cfg = UserConfig::load();
    DIURNAL_ACTIVE.store(initial_cfg.general.auto_theme_schedule, Ordering::Relaxed);

    info!(
        enabled = initial_cfg.general.auto_theme_schedule,
        "Starting Bihar Night diurnal theme scheduler"
    );

    let mut last_phase: Option<ThemeVariant> = None;

    loop {
        if is_diurnal_active() {
            let now = Local::now();
            let hour = now.hour();
            let (target_variant, target_temp, phase_desc) = resolve_diurnal_phase(hour);

            if last_phase != Some(target_variant) {
                last_phase = Some(target_variant);
                info!(
                    hour = hour,
                    variant = %target_variant.name(),
                    bihar = %target_variant.bihar_name(),
                    desc = %phase_desc,
                    "Diurnal schedule: transitioning to new Bihar Night phase"
                );

                // 1. Update theme via compositor IPC
                let _ = send_command(&IpcCommand::SetTheme(target_variant)).await;

                // 2. Adjust color temperature
                let _ = send_command(&IpcCommand::SetGamma(target_temp)).await;

                // 3. Emit notification banner
                NotificationHistory::send(
                    "Night Solar",
                    "🌙 Bihar Night Solar Transition",
                    &format!(
                        "Switched to {} ({})\n{}",
                        target_variant.name(),
                        target_variant.bihar_name(),
                        phase_desc
                    ),
                );
            }
        }

        // Check time every 60 seconds
        sleep(Duration::from_secs(60)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diurnal_phase_resolution() {
        // Daylight hours
        let (v, temp, _) = resolve_diurnal_phase(10);
        assert_eq!(v, ThemeVariant::Day);
        assert_eq!(temp, 6500);

        // Evening twilight
        let (v, temp, _) = resolve_diurnal_phase(19);
        assert_eq!(v, ThemeVariant::Storm);
        assert_eq!(temp, 4500);

        // Midnight
        let (v, temp, _) = resolve_diurnal_phase(23);
        assert_eq!(v, ThemeVariant::Night);
        assert_eq!(temp, 3400);

        // Early morning (3 AM)
        let (v, temp, _) = resolve_diurnal_phase(3);
        assert_eq!(v, ThemeVariant::Night);
        assert_eq!(temp, 3400);
    }
}
