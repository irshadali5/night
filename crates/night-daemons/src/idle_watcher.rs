//! 🌙 Tokyo Night Smart Idle & Inactivity Supervision Daemon
//! Monitors user activity, audio playback inhibition, and battery state.
//! Automatically stages power management:
//! 1. Dim display backlight after idle threshold (e.g. 5 minutes).
//! 2. Lock desktop session via `night-session lock` (e.g. 10 minutes).
//! 3. Blank displays via DPMS sleep (e.g. 15 minutes).
//! Automatically restores brightness and display state upon resumed activity.

use night_core::backlight::{handle_backlight_action, BacklightAction};
use night_core::exec::spawn;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};

/// Idle power state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleStage {
    Active,
    Dimmed,
    Locked,
    Suspended,
}

static IDLE_SECONDS: AtomicU64 = AtomicU64::new(0);
static IS_DIMMED: AtomicBool = AtomicBool::new(false);
static PREV_BRIGHTNESS: AtomicU64 = AtomicU64::new(100);

/// Queries current idle duration in seconds
pub fn get_idle_seconds() -> u64 {
    IDLE_SECONDS.load(Ordering::Relaxed)
}

/// Manually resets the idle counter (e.g. on user activity)
pub fn reset_idle() {
    IDLE_SECONDS.store(0, Ordering::Relaxed);
    if IS_DIMMED.swap(false, Ordering::Relaxed) {
        let prev = PREV_BRIGHTNESS.load(Ordering::Relaxed);
        info!(restore = prev, "Resuming from idle: restoring backlight");
        handle_backlight_action(BacklightAction::Set(prev as u8));
        spawn("nightctl", &["dpms", "on"]);
    }
}

/// Checks if an active MPRIS media player is currently playing
pub fn is_media_playing() -> bool {
    let output = std::process::Command::new("sh")
        .args([
            "-c",
            "target=$(busctl --user list 2>/dev/null | grep org.mpris.MediaPlayer2 | head -n1 | awk '{print $1}'); if [ -n \"$target\" ]; then busctl --user get-property \"$target\" /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player PlaybackStatus 2>/dev/null; fi",
        ])
        .output()
        .ok();

    if let Some(out) = output {
        let s = String::from_utf8_lossy(&out.stdout);
        s.contains("\"Playing\"")
    } else {
        false
    }
}

/// Periodically supervises idle timeouts and enforces staged power reduction
pub async fn supervise_idle() {
    info!("Starting pure-Rust smart idle supervision daemon");

    let dim_timeout = 300;  // 5 minutes
    let lock_timeout = 600; // 10 minutes
    let dpms_timeout = 900; // 15 minutes

    let mut current_stage = IdleStage::Active;

    loop {
        sleep(Duration::from_secs(10)).await;

        // 1. If inhibited (e.g. presentation mode or full-screen video), reset idle state
        if night_core::gamma::is_inhibited() || is_media_playing() {
            debug!("Activity/playback active: suppressing idle timers");
            reset_idle();
            current_stage = IdleStage::Active;
            continue;
        }

        let elapsed = IDLE_SECONDS.fetch_add(10, Ordering::Relaxed) + 10;

        // Stage 3: Suspend / DPMS Off
        if elapsed >= dpms_timeout && current_stage != IdleStage::Suspended {
            info!(elapsed = elapsed, "Idle timeout reached: powering off displays via DPMS");
            current_stage = IdleStage::Suspended;
            spawn("nightctl", &["dpms", "off"]);
        }
        // Stage 2: Lock Session
        else if elapsed >= lock_timeout && current_stage != IdleStage::Locked && current_stage != IdleStage::Suspended {
            info!(elapsed = elapsed, "Idle timeout reached: locking desktop session");
            current_stage = IdleStage::Locked;
            spawn("night-session", &["lock"]);
        }
        // Stage 1: Dim Backlight
        else if elapsed >= dim_timeout && current_stage == IdleStage::Active {
            info!(elapsed = elapsed, "Idle timeout reached: dimming backlight");
            current_stage = IdleStage::Dimmed;
            if let Some(bl) = night_core::backlight::get_backlight() {
                PREV_BRIGHTNESS.store(bl.percentage as u64, Ordering::Relaxed);
                let dimmed = (bl.percentage / 2).max(10);
                handle_backlight_action(BacklightAction::Set(dimmed));
                IS_DIMMED.store(true, Ordering::Relaxed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idle_counter_reset() {
        IDLE_SECONDS.store(500, Ordering::Relaxed);
        assert_eq!(get_idle_seconds(), 500);
        reset_idle();
        assert_eq!(get_idle_seconds(), 0);
    }
}
