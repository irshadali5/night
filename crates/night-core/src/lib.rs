//! 🌙 Night Core Library
//! Shared types, defaults, TOML user config, dirs path resolution,
//! interprocess IPC, and postcard session caching.

pub mod config;
pub mod defaults;
pub mod exec;
pub mod gamma;
pub mod ipc;
pub mod monitors;
pub mod state;
pub mod theme;

pub use config::UserConfig;
pub use defaults::Defaults;
pub use exec::{spawn, spawn_shell};
pub use ipc::{DpmsAction, IpcCommand, IpcResponse, OutputNav, WindowAction};
pub use monitors::{MonitorMode, MonitorPosition, MonitorProfile, MonitorTransform, MonitorVrr};
pub use state::SessionState;
pub use theme::Palette;
