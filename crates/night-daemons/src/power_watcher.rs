//! 🌙 Bihar Night Battery Power & Energy Conservation Supervisor
//! Periodically monitors battery capacity via `/sys/class/power_supply`.
//! Features:
//! - Multi-stage battery warnings (15% Low, 5% Critical)
//! - Bihar Night Battery Saver Mode: Engages Ganga Midnight deep dark palette and lowers brightness to 30%
//! - Automatic charger plug-in recovery

use night_core::backlight::{handle_backlight_action, BacklightAction};
use night_core::config::UserConfig;
use night_core::ipc::{send_command, IpcCommand};
use night_core::notifications::NotificationHistory;
use night_core::power::read_battery;
use night_core::theme::ThemeVariant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

static WARNED_LOW: AtomicBool = AtomicBool::new(false);
static WARNED_CRITICAL: AtomicBool = AtomicBool::new(false);
static POWER_SAVER_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Returns whether battery power saver mode is active
pub fn is_power_saver() -> bool {
    POWER_SAVER_ACTIVE.load(Ordering::Relaxed)
}

/// Sets battery power saver mode state and applies hardware adjustments
pub async fn set_power_saver(enabled: bool) {
    let was_active = POWER_SAVER_ACTIVE.swap(enabled, Ordering::Relaxed);
    let mut cfg = UserConfig::load();
    cfg.general.power_saver_mode = enabled;
    let _ = cfg.save();

    if enabled && !was_active {
        info!("Engaging Bihar Night Battery Saver Mode");

        // 1. Switch to energy-conserving Ganga Midnight dark palette
        let _ = send_command(&IpcCommand::SetTheme(ThemeVariant::Night)).await;

        // 2. Reduce backlight to 30% conservation level
        handle_backlight_action(BacklightAction::Set(30));

        // 3. Emit notification banner
        NotificationHistory::send(
            "Night Power",
            "⚡ Bihar Night Power Saver Active",
            "Switched to Ganga Midnight & lowered display brightness to conserve energy.",
        );
    } else if !enabled && was_active {
        info!("Disengaging Bihar Night Battery Saver Mode");

        // Restore normal brightness (e.g. 65%)
        handle_backlight_action(BacklightAction::Set(65));

        NotificationHistory::send(
            "Night Power",
            "⚡ Power Saver Deactivated",
            "Restored normal display brightness and desktop performance.",
        );
    }
}

/// Background supervision loop for battery warnings and energy conservation
pub async fn supervise_power_alerts() {
    let initial_cfg = UserConfig::load();
    POWER_SAVER_ACTIVE.store(initial_cfg.general.power_saver_mode, Ordering::Relaxed);

    info!(
        power_saver = initial_cfg.general.power_saver_mode,
        "Starting Bihar Night battery power supervisor"
    );

    loop {
        sleep(Duration::from_secs(30)).await;

        if let Some(battery) = read_battery() {
            if battery.is_charging {
                // Reset alert thresholds once plugged in
                WARNED_LOW.store(false, Ordering::Relaxed);
                WARNED_CRITICAL.store(false, Ordering::Relaxed);

                // Auto-exit power saver if triggered by low battery
                if is_power_saver() && battery.percentage >= 40 {
                    set_power_saver(false).await;
                }
            } else {
                // Battery is discharging
                if battery.percentage <= 5 && !WARNED_CRITICAL.load(Ordering::Relaxed) {
                    WARNED_CRITICAL.store(true, Ordering::Relaxed);
                    NotificationHistory::send(
                        "Night Power",
                        "🚨 Battery Critical",
                        &format!("Battery level is at {}%! Connect charger immediately.", battery.percentage),
                    );
                } else if battery.percentage <= 15 && !WARNED_LOW.load(Ordering::Relaxed) {
                    WARNED_LOW.store(true, Ordering::Relaxed);
                    NotificationHistory::send(
                        "Night Power",
                        "⚠️ Battery Low",
                        &format!("Battery level is at {}%. Connect charger soon.", battery.percentage),
                    );

                    // Automatically engage power saver mode at 15%
                    if !is_power_saver() {
                        set_power_saver(true).await;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_saver_toggle_state() {
        assert!(!is_power_saver());
        POWER_SAVER_ACTIVE.store(true, Ordering::Relaxed);
        assert!(is_power_saver());
        POWER_SAVER_ACTIVE.store(false, Ordering::Relaxed);
        assert!(!is_power_saver());
    }
}
