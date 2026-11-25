//! 🌙 Night Interprocess Communication (IPC)
//! Powered by `interprocess::local_socket` (Tokio), `tokio_util::codec::LengthDelimitedCodec`,
//! and `postcard` for bounded, framed binary message serialization.
//! Provides single-command request/response as well as real-time event streaming for status bars and widgets.

use crate::state::SessionState;
use futures::{SinkExt, StreamExt};
use interprocess::local_socket::{
    tokio::{prelude::*, Stream as LocalStream},
    GenericFilePath, ListenerOptions, ToFsName,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::sync::broadcast;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, info, warn};

/// Comprehensive real-time desktop status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
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
    #[serde(default)]
    pub media_info: Option<String>,
    #[serde(default)]
    pub active_tag_glyph: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default)]
    pub battery: Option<crate::power::BatteryInfo>,
    #[serde(default)]
    pub volume: Option<crate::audio::VolumeInfo>,
    #[serde(default)]
    pub network: Option<crate::network::NetworkInfo>,
    #[serde(default)]
    pub system: Option<crate::sysinfo::SystemStats>,
    #[serde(default)]
    pub bluetooth: Option<crate::bluetooth::BluetoothInfo>,
    #[serde(default)]
    pub backlight: Option<crate::backlight::BacklightInfo>,
    #[serde(default)]
    pub dnd: bool,
    #[serde(default)]
    pub auto_theme: bool,
    #[serde(default)]
    pub power_saver: bool,
}

impl DesktopStatus {
    /// Formats this status as a JSON string without external dependencies
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"active_tag":"{}","layout":"{}","inner_gaps":{:.1},"outer_gaps":{:.1},"active_window":{},"output_count":{},"last_reload":{},"inhibited":{},"gamma_temp":{},"media_info":{},"active_tag_glyph":{},"theme":{},"battery":{},"volume":{},"network":{},"system":{},"bluetooth":{},"backlight":{},"dnd":{},"auto_theme":{},"power_saver":{}}}"#,
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
            },
            match &self.media_info {
                Some(m) => format!("\"{}\"", m.replace('\\', "\\\\").replace('"', "\\\"")),
                None => "null".to_string(),
            },
            match &self.active_tag_glyph {
                Some(g) => format!("\"{}\"", g.replace('\\', "\\\\").replace('"', "\\\"")),
                None => "null".to_string(),
            },
            match &self.theme {
                Some(t) => format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\"")),
                None => "null".to_string(),
            },
            match &self.battery {
                Some(b) => format!(r#"{{"percentage":{},"is_charging":{},"status":"{}"}}"#, b.percentage, b.is_charging, b.status),
                None => "null".to_string(),
            },
            match &self.volume {
                Some(v) => format!(r#"{{"percentage":{},"is_muted":{}}}"#, v.percentage, v.is_muted),
                None => "null".to_string(),
            },
            match &self.network {
                Some(n) => format!(
                    r#"{{"interface":"{}","ssid":{},"signal_pct":{},"ip_addr":{},"is_online":{}}}"#,
                    n.interface,
                    match &n.ssid { Some(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")), None => "null".to_string() },
                    match n.signal_pct { Some(s) => s.to_string(), None => "null".to_string() },
                    match &n.ip_addr { Some(ip) => format!("\"{}\"", ip), None => "null".to_string() },
                    n.is_online
                ),
                None => "null".to_string(),
            },
            match &self.system {
                Some(s) => format!(
                    r#"{{"cpu_pct":{},"mem_used_mb":{},"mem_total_mb":{},"mem_pct":{},"cpu_temp_c":{},"disk_used_gb":{},"disk_total_gb":{},"disk_pct":{}}}"#,
                    s.cpu_pct, s.mem_used_mb, s.mem_total_mb, s.mem_pct,
                    match s.cpu_temp_c { Some(t) => t.to_string(), None => "null".to_string() },
                    s.disk_used_gb, s.disk_total_gb, s.disk_pct
                ),
                None => "null".to_string(),
            },
            match &self.bluetooth {
                Some(bt) => format!(
                    r#"{{"is_powered":{},"is_connected":{},"device_count":{}}}"#,
                    bt.is_powered, bt.is_connected, bt.devices.len()
                ),
                None => "null".to_string(),
            },
            match &self.backlight {
                Some(bl) => format!(r#"{{"percentage":{},"device":"{}"}}"#, bl.percentage, bl.device_name),
                None => "null".to_string(),
            },
            self.dnd,
            self.auto_theme,
            self.power_saver
        )
    }

    /// Formats this status using a customizable template string
    pub fn format_template(&self, template: &str) -> String {
        let active_win = self.active_window.as_deref().unwrap_or("None");
        let tag_glyph = self.active_tag_glyph.as_deref().unwrap_or("󰣆");
        let theme_str = self.theme.as_deref().unwrap_or("Night");
        let bat_str = self.battery.as_ref().map(|b| b.display_string()).unwrap_or_else(|| "AC".into());
        let bat_pct = self.battery.as_ref().map(|b| b.percentage.to_string()).unwrap_or_else(|| "100".into());
        let bat_icon = self.battery.as_ref().map(|b| b.glyph()).unwrap_or("󰂄");
        let vol_str = self.volume.as_ref().map(|v| v.display_string()).unwrap_or_else(|| "󰕾 50%".into());
        let vol_pct = self.volume.as_ref().map(|v| v.percentage.to_string()).unwrap_or_else(|| "50".into());
        let vol_icon = self.volume.as_ref().map(|v| v.glyph()).unwrap_or("󰕾");
        let net_str = self.network.as_ref().map(|n| n.display_string()).unwrap_or_else(|| "󰤮 Offline".into());
        let net_glyph = self.network.as_ref().map(|n| n.glyph()).unwrap_or("󰤮");
        let net_ssid = self.network.as_ref().and_then(|n| n.ssid.as_deref()).unwrap_or("Offline");
        let net_ip = self.network.as_ref().and_then(|n| n.ip_addr.as_deref()).unwrap_or("");
        let cpu_str = self.system.as_ref().map(|s| s.display_cpu()).unwrap_or_else(|| " 0%".into());
        let cpu_pct = self.system.as_ref().map(|s| s.cpu_pct.to_string()).unwrap_or_else(|| "0".into());
        let mem_str = self.system.as_ref().map(|s| s.display_mem()).unwrap_or_else(|| "󰍛 0 GiB".into());
        let mem_pct = self.system.as_ref().map(|s| s.mem_pct.to_string()).unwrap_or_else(|| "0".into());
        let temp_str = self.system.as_ref().map(|s| s.display_temp()).unwrap_or_else(|| "󰔏 N/A".into());
        let disk_str = self.system.as_ref().map(|s| s.display_disk()).unwrap_or_else(|| "󰋊 N/A".into());
        let bt_str = self.bluetooth.as_ref().map(|b| b.display_string()).unwrap_or_else(|| "󰂲 Off".into());
        let bt_icon = self.bluetooth.as_ref().map(|b| b.glyph()).unwrap_or("󰂲");
        let bl_str = self.backlight.as_ref().map(|b| b.display_string()).unwrap_or_else(|| "󰃟 50%".into());
        let bl_pct = self.backlight.as_ref().map(|b| b.percentage.to_string()).unwrap_or_else(|| "50".into());
        let bl_icon = self.backlight.as_ref().map(|b| b.glyph()).unwrap_or("󰃟");
        let dnd_str = if self.dnd { "DND Active" } else { "Normal" };
        let dnd_icon = if self.dnd { "󰂛" } else { "󰂚" };

        template
            .replace("{active_tag}", &self.active_tag)
            .replace("{tag}", &self.active_tag)
            .replace("{tag_glyph}", tag_glyph)
            .replace("{glyph}", tag_glyph)
            .replace("{layout}", &self.layout)
            .replace("{active_window}", active_win)
            .replace("{window}", active_win)
            .replace("{inner_gaps}", &format!("{:.0}", self.inner_gaps))
            .replace("{outer_gaps}", &format!("{:.0}", self.outer_gaps))
            .replace("{output_count}", &self.output_count.to_string())
            .replace("{inhibited}", if self.inhibited { "󰈈 Inhibited" } else { "󰈉 Normal" })
            .replace("{gamma}", &self.gamma_temp.map(|t| format!("{t}K")).unwrap_or_else(|| "6500K".into()))
            .replace("{media}", self.media_info.as_deref().unwrap_or("No Media"))
            .replace("{theme}", theme_str)
            .replace("{battery}", &bat_str)
            .replace("{battery_pct}", &bat_pct)
            .replace("{battery_icon}", bat_icon)
            .replace("{volume}", &vol_str)
            .replace("{volume_pct}", &vol_pct)
            .replace("{volume_icon}", vol_icon)
            .replace("{network}", &net_str)
            .replace("{net_icon}", net_glyph)
            .replace("{ssid}", net_ssid)
            .replace("{ip}", net_ip)
            .replace("{cpu}", &cpu_str)
            .replace("{cpu_pct}", &cpu_pct)
            .replace("{mem}", &mem_str)
            .replace("{mem_pct}", &mem_pct)
            .replace("{temp}", &temp_str)
            .replace("{disk}", &disk_str)
            .replace("{bt}", &bt_str)
            .replace("{bt_icon}", bt_icon)
            .replace("{brightness}", &bl_str)
            .replace("{brightness_pct}", &bl_pct)
            .replace("{brightness_icon}", bl_icon)
            .replace("{dnd}", dnd_str)
            .replace("{dnd_icon}", dnd_icon)
    }
}

/// Actions for MPRIS Media Players
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaAction {
    PlayPause,
    Next,
    Previous,
    Stop,
    Query,
}

/// Actions for Wayland Clipboard History
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClipboardAction {
    List,
    Show,
    Clear,
    Copy(usize),
    Record(String),
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
    Swap,
    Mirror,
}

/// Snapping target for floating client windows
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SnapPosition {
    Center,
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Pip,
    Maximize,
}

impl SnapPosition {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Center => "Center",
            Self::LeftHalf => "Left-Half",
            Self::RightHalf => "Right-Half",
            Self::TopHalf => "Top-Half",
            Self::BottomHalf => "Bottom-Half",
            Self::TopLeft => "Top-Left",
            Self::TopRight => "Top-Right",
            Self::BottomLeft => "Bottom-Left",
            Self::BottomRight => "Bottom-Right",
            Self::Pip => "Picture-in-Picture",
            Self::Maximize => "Maximize",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "center" | "mid" | "c" => Some(Self::Center),
            "left" | "left-half" | "l" => Some(Self::LeftHalf),
            "right" | "right-half" | "r" => Some(Self::RightHalf),
            "top" | "top-half" | "t" => Some(Self::TopHalf),
            "bottom" | "bottom-half" | "b" => Some(Self::BottomHalf),
            "top-left" | "topleft" | "tl" => Some(Self::TopLeft),
            "top-right" | "topright" | "tr" => Some(Self::TopRight),
            "bottom-left" | "bottomleft" | "bl" => Some(Self::BottomLeft),
            "bottom-right" | "bottomright" | "br" => Some(Self::BottomRight),
            "pip" | "picture-in-picture" | "p" => Some(Self::Pip),
            "max" | "maximize" | "fullscreen-float" => Some(Self::Maximize),
            _ => None,
        }
    }
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
    Snap(SnapPosition),
    Swallow,
    Unswallow,
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
    /// Triggers the Snowcap Bihar Night session flyout menu
    ShowSessionMenu,
    /// Triggers the Snowcap Bihar Night visual window switcher modal
    ShowSwitcher,
    /// Triggers the Snowcap Bihar Night clipboard history modal
    ShowClipboard,
    /// Triggers the Snowcap Bihar Night interactive snipping modal
    ShowSnipping,
    /// Sets active Bihar Night theme variant (Night, Storm, Moon, Day)
    SetTheme(crate::theme::ThemeVariant),
    /// Queries active Bihar Night theme variant
    GetTheme,
    /// Performs Wayland clipboard history operation
    Clipboard(ClipboardAction),
    /// Performs MPRIS media operation
    Media(MediaAction),
    /// Toggles a named scratchpad drawer (term, music, notes, fm)
    ToggleNamedScratchpad(String),
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
    /// Performs audio volume operation (raise, lower, set, mute, query)
    Volume(crate::audio::VolumeAction),
    /// Queries battery and power supply status
    GetBattery,
    /// Queries current volume status
    GetVolume,
    /// Triggers the Snowcap Bihar Night fuzzy app launcher modal
    ShowLauncher,
    /// Queries network connectivity status
    GetNetwork,
    /// Queries real-time hardware performance metrics (CPU, RAM, Temp, Disk)
    GetSystemStats,
    /// Queries Bluetooth controller and connected peripherals
    GetBluetooth,
    /// Discovers all available audio output sinks
    GetAudioSinks,
    /// Sets default audio output sink by ID
    SetAudioSink(u32),
    /// Cycles to the next available audio output sink
    CycleAudioSink,
    /// Triggers the Snowcap Bihar Night interactive audio sink switcher modal
    ShowAudioSinkMenu,
    /// Performs display backlight brightness adjustment
    Backlight(crate::backlight::BacklightAction),
    /// Queries display backlight status
    GetBacklight,
    /// Toggles Do Not Disturb (DND) mode
    ToggleDnd,
    /// Queries Do Not Disturb (DND) state
    GetDnd,
    /// Triggers the Snowcap Bihar Night notification center modal
    ShowNotifications,
    /// Saves current running desktop session layout
    SaveSession,
    /// Restores saved desktop session layout
    RestoreSession,
    /// Sets an ergonomic layout profile (Coding, Reading, Multitasking, Ultrawide)
    SetLayoutProfile(String),
    /// Triggers an on-screen display overlay (volume, brightness, dnd, gamma, layout)
    ShowOsd(String),
    /// Snaps focused window according to geometric position
    SnapWindow(SnapPosition),
    /// Triggers the Snowcap Bihar Night command palette modal
    ShowPalette,
    /// Triggers the Snowcap Bihar Night window overview switcher modal
    ShowOverview,
    /// Triggers the Snowcap Bihar Night 3x3 workspace grid (Exposé) matrix modal
    ShowGrid,
    /// Toggles window focus border decorations
    ToggleDecorations,
    /// Toggles window titlebars
    ToggleTitlebars,
    /// Sets window border thickness in pixels
    SetBorderWidth(u32),
    /// Sets layout algorithm directly for active workspace tag
    SetLayout(String),
    /// Toggles diurnal automatic day/night theme scheduling
    ToggleAutoTheme,
    /// Sets diurnal automatic day/night theme scheduling state
    SetAutoTheme(bool),
    /// Queries diurnal automatic day/night theme scheduling state
    GetAutoTheme,
    /// Toggles battery power saver mode
    TogglePowerSaver,
    /// Sets battery power saver mode state
    SetPowerSaver(bool),
    /// Queries battery power saver mode state
    GetPowerSaver,
    /// Queries live health status of core background daemons
    GetDaemons,
}

/// Live health status of a managed Night background daemon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonHealth {
    pub name: String,
    pub command: String,
    pub running: bool,
    pub installed: bool,
    pub restarts: u32,
}

/// Desktop IPC Response
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IpcResponse {
    Ok,
    State(SessionState),
    Layout(String),
    Status(DesktopStatus),
    Gamma(u32),
    Inhibited(bool),
    Clipboard(Vec<String>),
    Media(Option<String>),
    Theme(crate::theme::ThemeVariant),
    Battery(Option<crate::power::BatteryInfo>),
    Volume(Option<crate::audio::VolumeInfo>),
    Network(Option<crate::network::NetworkInfo>),
    SystemStats(crate::sysinfo::SystemStats),
    Bluetooth(Option<crate::bluetooth::BluetoothInfo>),
    AudioSinks(Vec<crate::audio::AudioSink>),
    Backlight(Option<crate::backlight::BacklightInfo>),
    Dnd(bool),
    Notifications(Vec<crate::notifications::NotificationRecord>),
    SavedSession(Option<crate::session_restore::SavedSession>),
    Profile(String),
    AutoTheme(bool),
    PowerSaver(bool),
    Daemons(Vec<DaemonHealth>),
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

const MAX_IPC_FRAME_BYTES: usize = 4 * 1024 * 1024;

fn framed_socket(stream: LocalStream) -> Framed<LocalStream, LengthDelimitedCodec> {
    LengthDelimitedCodec::builder()
        .max_frame_length(MAX_IPC_FRAME_BYTES)
        .new_framed(stream)
}

async fn send_command_to(
    path: &std::path::Path,
    cmd: &IpcCommand,
) -> Result<IpcResponse, crate::error::IpcError> {
    let path_buf = path.to_path_buf();
    let name = path_buf
        .clone()
        .to_fs_name::<GenericFilePath>()
        .map_err(|source| crate::error::IpcError::ConnectionFailed {
            path: path_buf.clone(),
            source,
        })?;
    let stream = LocalStream::connect(name)
        .await
        .map_err(|source| crate::error::IpcError::ConnectionFailed {
            path: path_buf.clone(),
            source,
        })?;
    let mut framed = framed_socket(stream);

    let payload = postcard::to_stdvec(cmd).map_err(crate::error::IpcError::Encode)?;
    framed
        .send(payload.as_slice())
        .await
        .map_err(|source| crate::error::IpcError::ConnectionFailed {
            path: path_buf.clone(),
            source,
        })?;

    let frame = framed
        .next()
        .await
        .ok_or_else(|| crate::error::IpcError::ConnectionFailed {
            path: path_buf.clone(),
            source: std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "IPC stream closed"),
        })?
        .map_err(|source| crate::error::IpcError::ConnectionFailed {
            path: path_buf,
            source,
        })?;

    postcard::from_bytes(&frame).map_err(crate::error::IpcError::Decode)
}

/// Sends a single IPC command to the running Night compositor instance and awaits response
pub async fn send_command(cmd: &IpcCommand) -> Result<IpcResponse, crate::error::IpcError> {
    send_command_to(&socket_path(), cmd).await
}

/// Connects to Night IPC and subscribes to real-time `DesktopStatus` updates.
/// Invokes the provided callback on each update until disconnected.
pub async fn subscribe_status<F>(mut on_status: F) -> Result<(), crate::error::IpcError>
where
    F: FnMut(DesktopStatus),
{
    let path = socket_path();
    let name = path
        .clone()
        .to_fs_name::<GenericFilePath>()
        .map_err(|source| crate::error::IpcError::ConnectionFailed {
            path: path.clone(),
            source,
        })?;
    let stream = LocalStream::connect(name)
        .await
        .map_err(|source| crate::error::IpcError::ConnectionFailed {
            path: path.clone(),
            source,
        })?;
    let mut framed = framed_socket(stream);

    let payload =
        postcard::to_stdvec(&IpcCommand::Subscribe).map_err(crate::error::IpcError::Encode)?;
    framed
        .send(payload.as_slice())
        .await
        .map_err(|source| crate::error::IpcError::ConnectionFailed {
            path: path.clone(),
            source,
        })?;

    while let Some(Ok(frame)) = framed.next().await {
        if let Ok(IpcResponse::Status(status)) = postcard::from_bytes::<IpcResponse>(&frame) {
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
    run_ipc_server_at(&socket_path(), handler, status_provider).await;
}

async fn run_ipc_server_at<F, S>(path: &std::path::Path, handler: F, status_provider: S)
where
    F: Fn(IpcCommand) -> IpcResponse + Send + Sync + 'static,
    S: Fn() -> DesktopStatus + Send + Sync + 'static,
{
    let _ = std::fs::remove_file(path);

    let name = match path.to_path_buf().to_fs_name::<GenericFilePath>() {
        Ok(n) => n,
        Err(e) => {
            error!(error = %e, socket = %path.display(), "Failed to build Night IPC socket name");
            return;
        }
    };

    let listener = match ListenerOptions::new().name(name).create_tokio() {
        Ok(l) => {
            info!(socket = %path.display(), "Night IPC Server active (interprocess)");
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
            Ok(stream) => {
                let handler = handler_arc.clone();
                let status_fn = status_arc.clone();

                tokio::spawn(async move {
                    let mut framed = framed_socket(stream);
                    let Some(Ok(buf)) = framed.next().await else {
                        return;
                    };

                    match postcard::from_bytes::<IpcCommand>(&buf) {
                        Ok(IpcCommand::Subscribe) => {
                            let initial = status_fn();
                            if let Ok(initial_payload) =
                                postcard::to_stdvec(&IpcResponse::Status(initial))
                            {
                                if framed.send(initial_payload.as_slice()).await.is_err() {
                                    return;
                                }
                            }

                            let mut rx = broadcaster().subscribe();
                            while let Ok(status) = rx.recv().await {
                                if let Ok(resp_payload) =
                                    postcard::to_stdvec(&IpcResponse::Status(status))
                                {
                                    if framed.send(resp_payload.as_slice()).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                        Ok(cmd) => {
                            let response = handler(cmd);
                            if let Ok(resp_payload) = postcard::to_stdvec(&response) {
                                let _ = framed.send(resp_payload.as_slice()).await;
                            }
                        }
                        Err(e) => {
                            let response = IpcResponse::Error(format!("Malformed command: {e}"));
                            if let Ok(resp_payload) = postcard::to_stdvec(&response) {
                                let _ = framed.send(resp_payload.as_slice()).await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_interprocess_ipc_roundtrip() {
        let sock = std::env::temp_dir().join(format!("night-ipc-test-{}.sock", std::process::id()));
        let sock_srv = sock.clone();
        let handle = tokio::spawn(async move {
            run_ipc_server_at(
                &sock_srv,
                |cmd| match cmd {
                    IpcCommand::GetPowerSaver => IpcResponse::PowerSaver(true),
                    _ => IpcResponse::Ok,
                },
                DesktopStatus::default,
            )
            .await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let resp = send_command_to(&sock, &IpcCommand::GetPowerSaver)
            .await
            .expect("IPC roundtrip failed");
        assert_eq!(resp, IpcResponse::PowerSaver(true));

        handle.abort();
        let _ = std::fs::remove_file(&sock);
    }
}
