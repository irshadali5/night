//! 🌙 Night Core Library
//! Shared types, defaults, TOML user config, dirs path resolution,
//! interprocess IPC, and postcard session caching.

pub mod audio;
pub mod bluetooth;
pub mod clipboard;
pub mod config;
pub mod defaults;
pub mod exec;
pub mod gamma;
pub mod ipc;
pub mod monitors;
pub mod network;
pub mod power;
pub mod state;
pub mod sysinfo;
pub mod theme;

pub use audio::{AudioSink, VolumeAction, VolumeInfo};
pub use bluetooth::{BluetoothDevice, BluetoothInfo};
pub use config::UserConfig;
pub use defaults::Defaults;
pub use exec::{spawn, spawn_shell};
pub use ipc::{DpmsAction, IpcCommand, IpcResponse, OutputNav, WindowAction};
pub use monitors::{MonitorMode, MonitorPosition, MonitorProfile, MonitorTransform, MonitorVrr};
pub use network::{NetworkInfo, NetworkKind};
pub use power::BatteryInfo;
pub use state::SessionState;
pub use sysinfo::SystemStats;
pub use theme::{Palette, ThemeVariant};
