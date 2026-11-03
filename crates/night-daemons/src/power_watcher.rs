//! 🌙 Battery Power & Low-Energy Alert Supervisor
//! Periodically monitors battery capacity via `/sys/class/power_supply` and emits desktop notifications.

use night_core::exec::spawn_shell;
use night_core::power::read_battery;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

static WARNED_LOW: AtomicBool = AtomicBool::new(false);
static WARNED_CRITICAL: AtomicBool = AtomicBool::new(false);

/// Background supervision loop for battery warnings
pub async fn supervise_power_alerts() {
    info!("Starting background battery power supervisor");

    loop {
        sleep(Duration::from_secs(30)).await;

        if let Some(battery) = read_battery() {
            if battery.is_charging {
                // Reset alert thresholds once plugged in
                WARNED_LOW.store(false, Ordering::Relaxed);
                WARNED_CRITICAL.store(false, Ordering::Relaxed);
            } else {
                // Battery is discharging
                if battery.percentage <= 5 && !WARNED_CRITICAL.load(Ordering::Relaxed) {
                    WARNED_CRITICAL.store(true, Ordering::Relaxed);
                    let msg = format!(
                        "notify-send -u critical -a \"Night Power\" \"🚨 Battery Critical\" \"Battery level is at {}%! Connect charger immediately.\" 2>/dev/null",
                        battery.percentage
                    );
                    spawn_shell(&msg);
                } else if battery.percentage <= 15 && !WARNED_LOW.load(Ordering::Relaxed) {
                    WARNED_LOW.store(true, Ordering::Relaxed);
                    let msg = format!(
                        "notify-send -u normal -a \"Night Power\" \"⚠️ Battery Low\" \"Battery level is at {}%. Connect charger soon.\" 2>/dev/null",
                        battery.percentage
                    );
                    spawn_shell(&msg);
                }
            }
        }
    }
}

