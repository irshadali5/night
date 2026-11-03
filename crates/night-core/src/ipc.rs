//! 🌙 Night Interprocess Communication (IPC)
//! Powered by standard Unix domain sockets and `postcard` for binary message serialization.
//! Provides single-command request/response as well as real-time event streaming for status bars and widgets.

use crate::state::SessionState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

/// Comprehensive real-time desktop status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesktopStatus {
    pub active_tag: String,
    pub layout: String,
    pub inner_gaps: f32,
    pub outer_gaps: f32,
    pub active_window: Option<String>,
    pub output_count: usize,
    pub last_reload: u64,
    #[serde(default)]
    pub inhibited: bool,
    #[serde(default)]
    pub gamma_temp: Option<u32>,
}

impl DesktopStatus {
    /// Formats this status as a JSON string without external dependencies
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"active_tag":"{}","layout":"{}","inner_gaps":{:.1},"outer_gaps":{:.1},"active_window":{},"output_count":{},"last_reload":{},"inhibited":{},"gamma_temp":{}}}"#,
            self.active_tag,
            self.layout,
            self.inner_gaps,
            self.outer_gaps,
            match &self.active_window {
                Some(w) => format!("\"{}\"", w.replace('\\', "\\\\").replace('"', "\\\"")),
                None => "null".to_string(),
            },
            self.output_count,
            self.last_reload,
            self.inhibited,
            match self.gamma_temp {
                Some(t) => t.to_string(),
                None => "null".to_string(),
            }
        )
    }

    /// Formats this status using a customizable template string
    pub fn format_template(&self, template: &str) -> String {
        let active_win = self.active_window.as_deref().unwrap_or("None");
        template
            .replace("{active_tag}", &self.active_tag)
            .replace("{tag}", &self.active_tag)
            .replace("{layout}", &self.layout)
            .replace("{active_window}", active_win)
            .replace("{window}", active_win)
            .replace("{inner_gaps}", &format!("{:.0}", self.inner_gaps))
            .replace("{outer_gaps}", &format!("{:.0}", self.outer_gaps))
            .replace("{output_count}", &self.output_count.to_string())
            .replace("{inhibited}", if self.inhibited { "󰈈 Inhibited" } else { "󰈉 Normal" })
            .replace("{gamma}", &self.gamma_temp.map(|t| format!("{t}K")).unwrap_or_else(|| "6500K".into()))
    }
}

/// DPMS Display Power Action
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DpmsAction {
    On,
    Off,
    Toggle,
}

/// Output Navigation Direction or Target
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputNav {
    Next,
    Prev,
    Name(String),
}

/// Window Operations for focused window
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WindowAction {
    Sticky,
    Pip,
    Center,
    ToggleFloat,
    ToggleFullscreen,
    ToggleMaximize,
}

/// Desktop IPC Request Commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Cycles to the next tiling layout algorithm
    CycleLayout,
    /// Adjusts layout gaps by a delta value
    AdjustGaps(f32),
    /// Sets layout gaps to explicit values
    SetGaps {
        inner: Option<f32>,
        outer: Option<f32>,
    },
    /// Queries current active layout name
    GetLayout,
    /// Queries full desktop status
    GetStatus,
    /// Toggles the interactive Quake-style dropdown scratchpad terminal
    ToggleScratchpad,
    /// Triggers the Snowcap keybindings overlay
    ShowBinds,
    /// Triggers the Snowcap quit confirmation prompt
    ShowQuit,
    /// Triggers the Snowcap Tokyo Night session flyout menu
    ShowSessionMenu,
    /// Triggers the Snowcap Tokyo Night visual window switcher modal
    ShowSwitcher,
    /// Toggles night light / gamma color temperature
    ToggleGamma,
    /// Sets night light color temperature in Kelvin (e.g. 3800, 6500)
    SetGamma(u32),
    /// Toggles idle inhibition (presentation/compile mode)
    ToggleInhibit,
    /// Subscribes to real-time status updates (persistent event stream)
    Subscribe,
    /// Changes focus to an output monitor
    FocusOutput(OutputNav),
    /// Moves currently focused window to an output monitor
    MoveWindowToOutput(OutputNav),
    /// Manages DPMS display power
    Dpms(DpmsAction),
    /// Performs window operation on currently focused window
    WindowOp(WindowAction),
}

/// Desktop IPC Response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IpcResponse {
    Ok,
    State(SessionState),
    Layout(String),
    Status(DesktopStatus),
    Gamma(u32),
    Inhibited(bool),
    Error(String),
}

/// Global status broadcast channel for real-time subscribers
static STATUS_BROADCASTER: OnceLock<broadcast::Sender<DesktopStatus>> = OnceLock::new();

/// Returns reference to the global status broadcaster sender
pub fn broadcaster() -> &'static broadcast::Sender<DesktopStatus> {
    STATUS_BROADCASTER.get_or_init(|| {
        let (tx, _rx) = broadcast::channel(64);
        tx
    })
}

/// Broadcasts a newly computed `DesktopStatus` to all connected streaming clients
pub fn broadcast_status(status: DesktopStatus) {
    let _ = broadcaster().send(status);
}

/// Resolves standard XDG socket path via `dirs`
pub fn socket_path() -> PathBuf {
    dirs::runtime_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("night.sock")
}

/// Sends a single IPC command to the running Night compositor instance and awaits response
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

/// Connects to Night IPC and subscribes to real-time `DesktopStatus` updates.
/// Invokes the provided callback on each update until disconnected.
pub async fn subscribe_status<F>(mut on_status: F) -> Result<(), String>
where
    F: FnMut(DesktopStatus),
{
    let path = socket_path();
    let mut stream = UnixStream::connect(&path)
        .await
        .map_err(|e| format!("Failed to connect to Night IPC socket ({}): {e}", path.display()))?;

    let payload = postcard::to_stdvec(&IpcCommand::Subscribe)
        .map_err(|e| format!("Serialization error: {e}"))?;

    let len = payload.len() as u32;
    stream.write_all(&len.to_be_bytes()).await.map_err(|e| e.to_string())?;
    stream.write_all(&payload).await.map_err(|e| e.to_string())?;

    loop {
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).await.is_err() {
            break;
        }
        let resp_len = u32::from_be_bytes(len_buf) as usize;

        let mut resp_buf = vec![0u8; resp_len];
        if stream.read_exact(&mut resp_buf).await.is_err() {
            break;
        }

        if let Ok(IpcResponse::Status(status)) = postcard::from_bytes::<IpcResponse>(&resp_buf) {
            on_status(status);
        }
    }

    Ok(())
}

/// Runs the multi-client IPC listener server, supporting both single commands and persistent event subscriptions.
pub async fn run_ipc_server<F, S>(handler: F, status_provider: S)
where
    F: Fn(IpcCommand) -> IpcResponse + Send + Sync + 'static,
    S: Fn() -> DesktopStatus + Send + Sync + 'static,
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

    let handler_arc = std::sync::Arc::new(handler);
    let status_arc = std::sync::Arc::new(status_provider);

    loop {
        match listener.accept().await {
            Ok((mut stream, _addr)) => {
                let handler = handler_arc.clone();
                let status_fn = status_arc.clone();

                tokio::spawn(async move {
                    let mut len_buf = [0u8; 4];
                    if stream.read_exact(&mut len_buf).await.is_err() {
                        return;
                    }
                    let len = u32::from_be_bytes(len_buf) as usize;
                    let mut buf = vec![0u8; len];
                    if stream.read_exact(&mut buf).await.is_err() {
                        return;
                    }

                    match postcard::from_bytes::<IpcCommand>(&buf) {
                        Ok(IpcCommand::Subscribe) => {
                            // 1. Send immediate initial state
                            let initial = status_fn();
                            if let Ok(initial_payload) = postcard::to_stdvec(&IpcResponse::Status(initial)) {
                                let len = initial_payload.len() as u32;
                                if stream.write_all(&len.to_be_bytes()).await.is_err()
                                    || stream.write_all(&initial_payload).await.is_err()
                                {
                                    return;
                                }
                            }

                            // 2. Stream broadcasted events
                            let mut rx = broadcaster().subscribe();
                            while let Ok(status) = rx.recv().await {
                                if let Ok(resp_payload) = postcard::to_stdvec(&IpcResponse::Status(status)) {
                                    let len = resp_payload.len() as u32;
                                    if stream.write_all(&len.to_be_bytes()).await.is_err()
                                        || stream.write_all(&resp_payload).await.is_err()
                                    {
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(cmd) => {
                            let response = handler(cmd);
                            if let Ok(resp_payload) = postcard::to_stdvec(&response) {
                                let resp_len = resp_payload.len() as u32;
                                let _ = stream.write_all(&resp_len.to_be_bytes()).await;
                                let _ = stream.write_all(&resp_payload).await;
                            }
                        }
                        Err(e) => {
                            let response = IpcResponse::Error(format!("Malformed command: {e}"));
                            if let Ok(resp_payload) = postcard::to_stdvec(&response) {
                                let resp_len = resp_payload.len() as u32;
                                let _ = stream.write_all(&resp_len.to_be_bytes()).await;
                                let _ = stream.write_all(&resp_payload).await;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                warn!(error = %e, "IPC accept error");
            }
        }
    }
}
