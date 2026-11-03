//! 🌙 Night Interprocess Communication (IPC)
//! Powered by `interprocess` for Unix domain sockets and `postcard` for binary message serialization.

use crate::state::SessionState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Desktop IPC Request Commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IpcCommand {
    /// Reloads the compositor configuration
    ReloadConfig,
    /// Switches to the specified tag name
    SwitchTag(String),
    /// Queries the current session state
    QueryState,
    /// Spawns a command string
    Spawn(String),
    /// Closes the focused window
    CloseFocused,
}

/// Desktop IPC Response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IpcResponse {
    Ok,
    State(SessionState),
    Error(String),
}

/// Resolves standard XDG socket path via `dirs`
pub fn socket_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("night.sock")
}
