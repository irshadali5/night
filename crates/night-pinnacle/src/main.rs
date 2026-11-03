// 🌙 Night Desktop Environment - Main Pinnacle Entrypoint
// Orchestrates modular workspace crates: core, daemons, tags, rules, and binds.
// Utilizes TOML for user preferences, RON for window rules, and postcard for session caching.

use night_core::ipc::{IpcCommand, IpcResponse};
use night_core::{SessionState, UserConfig};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn config() {
    // -------------------------------------------------------------------------
    // 1. Initialize Logging & Tracing
    // -------------------------------------------------------------------------
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "night_pinnacle=info,night_core=info,pinnacle_api=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🌙 Initializing Project Night (Rust Wayland Desktop Environment)...");

    // -------------------------------------------------------------------------
    // 2. Load User Configuration (TOML)
    // -------------------------------------------------------------------------
    let user_config = UserConfig::load();
    info!(terminal = %user_config.apps.terminal, "User configuration active.");

    // -------------------------------------------------------------------------
    // 3. Load or Restore Session State (Postcard Binary Deserialization)
    // -------------------------------------------------------------------------
    if let Some(saved_session) = SessionState::load() {
        info!(tag = %saved_session.active_tag, "Resuming previous session state from postcard cache.");
    }

    // -------------------------------------------------------------------------
    // 4. Setup Dynamic Tags & Layout Engine
    // -------------------------------------------------------------------------
    night_tags::setup_tags();
    night_layout::setup_layout();

    // -------------------------------------------------------------------------
    // 5. Supervise Background Daemons & Services
    // -------------------------------------------------------------------------
    if user_config.general.autostart_daemons {
        night_daemons::start_all().await;
    }

    // -------------------------------------------------------------------------
    // 6. Initialize Window Rules (RON)
    // -------------------------------------------------------------------------
    night_rules::setup_rules();

    // -------------------------------------------------------------------------
    // 7. Register Keyboard & Multimedia Bindings
    // -------------------------------------------------------------------------
    night_binds::register_all();

    // -------------------------------------------------------------------------
    // 8. Start Desktop IPC Server (Unix Socket via `postcard` binary serialization)
    // -------------------------------------------------------------------------
    tokio::spawn(async move {
        night_core::ipc::run_ipc_server(|cmd| match cmd {
            IpcCommand::ReloadConfig => {
                info!("IPC: Received ReloadConfig command");
                pinnacle_api::pinnacle::reload_config();
                IpcResponse::Ok
            }
            IpcCommand::SwitchTag(name) => {
                info!(tag = %name, "IPC: Received SwitchTag command");
                night_tags::switch_to_tag(&name);
                IpcResponse::Ok
            }
            IpcCommand::QueryState => {
                let state = SessionState::load().unwrap_or(SessionState {
                    active_tag: "1".into(),
                    last_reload_timestamp: 0,
                });
                IpcResponse::State(state)
            }
            IpcCommand::Spawn(command) => {
                info!(command = %command, "IPC: Received Spawn command");
                night_core::exec::spawn_shell(&command);
                IpcResponse::Ok
            }
            IpcCommand::CloseFocused => {
                info!("IPC: Received CloseFocused command");
                if let Some(focused) = pinnacle_api::window::get_focused() {
                    focused.close();
                }
                IpcResponse::Ok
            }
            IpcCommand::CycleLayout => {
                let next = night_layout::cycle_layout();
                IpcResponse::Layout(next.name().into())
            }
            IpcCommand::AdjustGaps(delta) => {
                night_layout::adjust_gaps(delta);
                IpcResponse::Ok
            }
            IpcCommand::GetLayout => {
                IpcResponse::Layout(night_layout::current_layout().name().into())
            }
        })
        .await;
    });

    // Save initial session state using postcard
    let initial_state = SessionState {
        active_tag: user_config.general.default_tag,
        last_reload_timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };
    initial_state.save();

    info!("🌙 Project Night fully initialized and active.");
}

// Expands into Tokio runtime setup, connection handling, and event loop.
pinnacle_api::main!(config);
