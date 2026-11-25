//! 🌙 Night Core Library
//! Shared types, defaults, TOML user config, dirs path resolution,
//! interprocess IPC, and postcard session caching.

pub mod audio;
pub mod backlight;
pub mod bluetooth;
pub mod clipboard;
pub mod config;
pub mod defaults;
pub mod error;
pub mod exec;
pub mod gamma;
pub mod ipc;
pub mod logging;
pub mod monitors;
pub mod network;
pub mod notifications;
pub mod power;
pub mod session_restore;
pub mod state;
pub mod sysinfo;
pub mod theme;

pub use audio::{AudioSink, VolumeAction, VolumeInfo};
pub use backlight::{BacklightAction, BacklightInfo};
pub use bluetooth::{BluetoothDevice, BluetoothInfo};
pub use config::UserConfig;
pub use defaults::Defaults;
pub use error::{ConfigError, IpcError, NightError, Result};
pub use exec::{ensure_desktop_env, resolve_executable, spawn, spawn_shell};
pub use ipc::{DpmsAction, IpcCommand, IpcResponse, OutputNav, WindowAction};
pub use logging::{init_logger, log_dir, log_file_path, read_recent_logs};
pub use monitors::{MonitorMode, MonitorPosition, MonitorProfile, MonitorTransform, MonitorVrr};
pub use network::{NetworkInfo, NetworkKind};
pub use notifications::{NotificationHistory, NotificationRecord};
pub use power::BatteryInfo;
pub use session_restore::{SavedSession, SavedWindow};
pub use state::SessionState;
pub use sysinfo::SystemStats;
pub use theme::{Palette, ThemeVariant};

/// Acquires a [`std::sync::Mutex`] guard, recovering cleanly if the mutex was poisoned.
pub fn lock_unpoison<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Atomically writes `data` to `path` via a sibling temporary file and `rename`,
/// preventing partial/corrupted config or state files on crash or concurrent writes.
pub fn write_atomic(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "night".to_string());
    let tmp_path = path.with_file_name(format!(".{file_name}.tmp.{}", std::process::id()));
    std::fs::write(&tmp_path, data)?;
    if let Err(err) = std::fs::rename(&tmp_path, path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(err);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn test_write_atomic_creates_and_overwrites() {
        let dir = std::env::temp_dir().join(format!("night-atomic-test-{}", std::process::id()));
        let file = dir.join("nested/state.bin");

        write_atomic(&file, b"first").expect("initial atomic write");
        assert_eq!(std::fs::read(&file).unwrap(), b"first");

        write_atomic(&file, b"second").expect("overwrite atomic write");
        assert_eq!(std::fs::read(&file).unwrap(), b"second");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_lock_unpoison_recovers_from_panic() {
        let m = Mutex::new(42);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut guard = m.lock().unwrap();
            *guard = 99;
            panic!("intentional poison");
        }));
        let guard = lock_unpoison(&m);
        assert_eq!(*guard, 99);
    }
}
