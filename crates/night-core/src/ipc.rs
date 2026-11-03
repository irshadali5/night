//! 🌙 Night Interprocess Communication (IPC)
//! Powered by `interprocess` for Unix domain sockets and `postcard` for binary message serialization.

use crate::state::SessionState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info, warn};

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

/// Sends an IPC command to the running Night compositor instance
pub async fn send_command(cmd: &IpcCommand) -> Result<IpcResponse, String> {
    let path = socket_path();
    let mut stream = UnixStream::connect(&path)
        .await
        .map_err(|e| format!("Failed to connect to Night IPC socket ({}): {e}", path.display()))?;

    let payload = postcard::to_stdvec(cmd)
        .map_err(|e| format!("Serialization error: {e}"))?;

    let len = payload.len() as u32;
    stream.write_all(&len.to_be_bytes()).await.map_err(|e| e.to_string())?;
    stream.write_all(&payload).await.map_err(|e| e.to_string())?;

    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await.map_err(|e| e.to_string())?;
    let resp_len = u32::from_be_bytes(len_buf) as usize;

    let mut resp_buf = vec![0u8; resp_len];
    stream.read_exact(&mut resp_buf).await.map_err(|e| e.to_string())?;

    postcard::from_bytes(&resp_buf)
        .map_err(|e| format!("Deserialization error: {e}"))
}

/// Runs the IPC listener server, processing commands asynchronously
pub async fn run_ipc_server<F>(mut handler: F)
where
    F: FnMut(IpcCommand) -> IpcResponse + Send + 'static,
{
    let path = socket_path();
    let _ = std::fs::remove_file(&path);

    let listener = match UnixListener::bind(&path) {
        Ok(l) => {
            info!(socket = %path.display(), "Night IPC Server active");
            l
        }
        Err(e) => {
            error!(error = %e, socket = %path.display(), "Failed to bind Night IPC socket");
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                let mut len_buf = [0u8; 4];
                if stream.read_exact(&mut len_buf).await.is_err() {
                    continue;
                }
                let len = u32::from_be_bytes(len_buf) as usize;
                let mut buf = vec![0u8; len];
                if stream.read_exact(&mut buf).await.is_err() {
                    continue;
                }

                let response = match postcard::from_bytes::<IpcCommand>(&buf) {
                    Ok(cmd) => handler(cmd),
                    Err(e) => IpcResponse::Error(format!("Malformed command: {e}")),
                };

                if let Ok(resp_payload) = postcard::to_stdvec(&response) {
                    let resp_len = resp_payload.len() as u32;
                    let _ = stream.write_all(&resp_len.to_be_bytes()).await;
                    let _ = stream.write_all(&resp_payload).await;
                }
            }
            Err(e) => {
                warn!(error = %e, "IPC accept error");
            }
        }
    }
}
