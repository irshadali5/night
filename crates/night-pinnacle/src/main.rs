// 🌙 Night Desktop Environment - Main Pinnacle Entrypoint
// Orchestrates modular workspace crates: core, daemons, tags, rules, and binds.
// Utilizes TOML for user preferences, RON for window rules, and postcard for session caching.

use night_core::ipc::{DesktopStatus, IpcCommand, IpcResponse};
use night_core::{SessionState, UserConfig};
use tracing::info;

/// Gathers comprehensive real-time desktop status across active outputs, windows, and tags
fn current_desktop_status() -> DesktopStatus {
    let state = SessionState::load().unwrap_or(SessionState {
        active_tag: "1".into(),
        last_reload_timestamp: 0,
    });
    let (inner_gaps, outer_gaps) = night_layout::get_gaps();
    let active_win = pinnacle_api::window::get_focused()
        .map(|w| format!("{} ({})", w.title(), w.app_id()));
    let output_count = pinnacle_api::output::get_all().count();

    let media_info = {
        let bus_out = std::process::Command::new("sh")
            .args(["-c", "target=$(busctl --user list 2>/dev/null | grep org.mpris.MediaPlayer2 | head -n1 | awk '{print $1}'); if [ -n \"$target\" ]; then busctl --user get-property \"$target\" /org/mpris/MediaPlayer2 org.mpris.MediaPlayer2.Player Metadata 2>/dev/null; fi"])
            .output()
            .ok();
        bus_out.and_then(|out| {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !s.is_empty() && s != " - " {
                    // If output contains busctl metadata signature, extract title
                    if s.contains("xesam:title") {
                        let title = s.lines()
                            .find(|l| l.contains("xesam:title"))
                            .and_then(|l| l.split('"').nth(1))
                            .unwrap_or("");
                        let artist = s.lines()
                            .find(|l| l.contains("xesam:artist"))
                            .and_then(|l| l.split('"').nth(1))
                            .unwrap_or("");
                        if !title.is_empty() {
                            if !artist.is_empty() {
                                return Some(format!("{artist} - {title}"));
                            }
                            return Some(title.to_string());
                        }
                    } else if !s.contains("Metadata") {
                        return Some(s);
                    }
                }
            }
            None
        })
    };

    let glyph = night_tags::tag_glyph(&state.active_tag);
    let user_cfg = UserConfig::load();
    let theme_str = user_cfg.general.theme.name().to_string();

    DesktopStatus {
        active_tag: state.active_tag,
        layout: night_layout::current_layout().name().into(),
        inner_gaps,
        outer_gaps,
        active_window: active_win,
        output_count,
        last_reload: state.last_reload_timestamp,
        inhibited: night_core::gamma::is_inhibited(),
        gamma_temp: Some(night_core::gamma::current_temperature()),
        media_info,
        active_tag_glyph: Some(glyph.to_string()),
        theme: Some(theme_str),
        battery: night_core::power::read_battery(),
        volume: night_core::audio::get_volume(),
        network: night_core::network::read_network(),
        system: Some(night_core::sysinfo::read_system_stats()),
        bluetooth: night_core::bluetooth::read_bluetooth(),
        backlight: night_core::backlight::get_backlight(),
        dnd: night_core::notifications::NotificationHistory::is_dnd(),
    }
}

async fn config() {
    // -------------------------------------------------------------------------
    // 1. Initialize Logging & Desktop Environment
    // -------------------------------------------------------------------------
    let _log_guard = night_core::logging::init_logger("night");
    if let Some(guard) = _log_guard {
        Box::leak(Box::new(guard));
    }
    night_core::exec::ensure_desktop_env();

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
    // 7. Register Keyboard & Multimedia Bindings (including Snowcap widgets)
    // -------------------------------------------------------------------------
    night_binds::register_all();

    // -------------------------------------------------------------------------
    // 8. Connect Compositor Lifecycle Signals (for real-time streaming to status bars)
    // -------------------------------------------------------------------------
    let _win_focused_signal = Box::leak(Box::new(pinnacle_api::window::connect_signal(
        pinnacle_api::signal::WindowSignal::Focused(Box::new(|_win| {
            night_core::ipc::broadcast_status(current_desktop_status());
        })),
    )));

    let _win_title_signal = Box::leak(Box::new(pinnacle_api::window::connect_signal(
        pinnacle_api::signal::WindowSignal::TitleChanged(Box::new(|_win, _title| {
            night_core::ipc::broadcast_status(current_desktop_status());
        })),
    )));

    let _tag_active_signal = Box::leak(Box::new(pinnacle_api::tag::connect_signal(
        pinnacle_api::signal::TagSignal::Active(Box::new(|_handle, active| {
            if active {
                night_core::ipc::broadcast_status(current_desktop_status());
            }
        })),
    )));

    let _op_connect_signal = Box::leak(Box::new(pinnacle_api::output::connect_signal(
        pinnacle_api::signal::OutputSignal::Connect(Box::new(|_op| {
            night_core::ipc::broadcast_status(current_desktop_status());
        })),
    )));

    let _op_disconnect_signal = Box::leak(Box::new(pinnacle_api::output::connect_signal(
        pinnacle_api::signal::OutputSignal::Disconnect(Box::new(|_op| {
            night_core::ipc::broadcast_status(current_desktop_status());
        })),
    )));

    let _op_focused_signal = Box::leak(Box::new(pinnacle_api::output::connect_signal(
        pinnacle_api::signal::OutputSignal::Focused(Box::new(|_op| {
            night_core::ipc::broadcast_status(current_desktop_status());
        })),
    )));

    // -------------------------------------------------------------------------
    // 9. Start Desktop IPC Server (Unix Socket via `postcard` binary serialization)
    // -------------------------------------------------------------------------
    tokio::spawn(async move {
        night_core::ipc::run_ipc_server(
            |cmd| match cmd {
                IpcCommand::ReloadConfig => {
                    info!("IPC: Received ReloadConfig command");
                    pinnacle_api::pinnacle::reload_config();
                    IpcResponse::Ok
                }
                IpcCommand::SwitchTag(name) => {
                    info!(tag = %name, "IPC: Received SwitchTag command");
                    night_tags::switch_to_tag(&name);
                    night_core::ipc::broadcast_status(current_desktop_status());
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
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Layout(next.name().into())
                }
                IpcCommand::AdjustGaps(delta) => {
                    night_layout::adjust_gaps(delta);
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Ok
                }
                IpcCommand::SetGaps { inner, outer } => {
                    night_layout::set_gaps(inner, outer);
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Ok
                }
                IpcCommand::GetLayout => {
                    IpcResponse::Layout(night_layout::current_layout().name().into())
                }
                IpcCommand::GetStatus => {
                    IpcResponse::Status(current_desktop_status())
                }
                IpcCommand::ToggleScratchpad => {
                    night_binds::toggle_scratchpad();
                    IpcResponse::Ok
                }
                IpcCommand::ShowBinds => {
                    night_binds::show_bind_overlay();
                    IpcResponse::Ok
                }
                IpcCommand::ShowQuit => {
                    night_binds::show_quit_prompt();
                    IpcResponse::Ok
                }
                IpcCommand::ShowSessionMenu => {
                    night_binds::show_session_menu();
                    IpcResponse::Ok
                }
                IpcCommand::ShowSwitcher | IpcCommand::ShowOverview => {
                    night_binds::show_window_switcher();
                    IpcResponse::Ok
                }
                IpcCommand::ShowPalette => {
                    night_binds::show_command_palette();
                    IpcResponse::Ok
                }
                IpcCommand::ShowGrid => {
                    night_binds::show_workspace_grid();
                    IpcResponse::Ok
                }
                IpcCommand::ToggleDecorations => {
                    let state = night_binds::decorations::toggle_decorations();
                    info!(enabled = state, "IPC: ToggleDecorations");
                    IpcResponse::Ok
                }
                IpcCommand::ToggleTitlebars => {
                    let state = night_binds::decorations::toggle_titlebars();
                    info!(titlebars = state, "IPC: ToggleTitlebars");
                    IpcResponse::Ok
                }
                IpcCommand::SetBorderWidth(width) => {
                    night_binds::decorations::set_border_width(width);
                    info!(width, "IPC: SetBorderWidth");
                    IpcResponse::Ok
                }
                IpcCommand::SnapWindow(pos) => {
                    night_binds::snap_focused_window(pos);
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Ok
                }
                IpcCommand::ShowClipboard => {
                    night_binds::show_clipboard_modal();
                    IpcResponse::Ok
                }
                IpcCommand::ShowSnipping => {
                    night_binds::show_snipping_modal();
                    IpcResponse::Ok
                }
                IpcCommand::SetTheme(variant) => {
                    info!(theme = %variant.name(), "IPC: Setting Tokyo Night theme variant");
                    let mut cfg = UserConfig::load();
                    cfg.general.theme = variant;
                    let _ = cfg.save();
                    night_daemons::wallpaper::sync_wallpaper_theme(variant);
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Theme(variant)
                }
                IpcCommand::GetTheme => {
                    let cfg = UserConfig::load();
                    IpcResponse::Theme(cfg.general.theme)
                }
                IpcCommand::Clipboard(action) => {
                    use night_core::ipc::ClipboardAction;
                    match action {
                        ClipboardAction::List => {
                            let hist = night_core::clipboard::ClipboardHistory::load();
                            let items = hist.items.into_iter().map(|i| i.content).collect();
                            IpcResponse::Clipboard(items)
                        }
                        ClipboardAction::Clear => {
                            night_core::clipboard::ClipboardHistory::clear_all();
                            IpcResponse::Ok
                        }
                        ClipboardAction::Copy(idx) => {
                            let _ = night_core::clipboard::ClipboardHistory::copy_entry_at(idx);
                            IpcResponse::Ok
                        }
                        ClipboardAction::Record(text) => {
                            night_core::clipboard::ClipboardHistory::record_text(&text);
                            IpcResponse::Ok
                        }
                        ClipboardAction::Show => {
                            night_binds::show_clipboard_modal();
                            IpcResponse::Ok
                        }
                    }
                }
                IpcCommand::Media(action) => {
                    night_binds::handle_media_action(action);
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Ok
                }
                IpcCommand::ToggleNamedScratchpad(name) => {
                    night_binds::toggle_named_scratchpad(&name);
                    IpcResponse::Ok
                }
                IpcCommand::ToggleGamma => {
                    let temp = night_core::gamma::toggle_night_light();
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Gamma(temp)
                }
                IpcCommand::SetGamma(temp) => {
                    night_core::gamma::set_temperature(temp);
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Gamma(temp)
                }
                IpcCommand::ToggleInhibit => {
                    let inh = night_core::gamma::toggle_inhibit();
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Inhibited(inh)
                }
                IpcCommand::WindowOp(op) => {
                    match op {
                        night_core::ipc::WindowAction::Sticky => night_binds::toggle_sticky(),
                        night_core::ipc::WindowAction::Pip => night_binds::toggle_pip(),
                        night_core::ipc::WindowAction::Center => night_binds::center_focused_window(),
                        night_core::ipc::WindowAction::ToggleFloat => {
                            if let Some(focused) = pinnacle_api::window::get_focused() {
                                focused.toggle_floating();
                            }
                        }
                        night_core::ipc::WindowAction::ToggleFullscreen => {
                            if let Some(focused) = pinnacle_api::window::get_focused() {
                                focused.toggle_fullscreen();
                            }
                        }
                        night_core::ipc::WindowAction::ToggleMaximize => {
                            if let Some(focused) = pinnacle_api::window::get_focused() {
                                focused.toggle_maximized();
                            }
                        }
                        night_core::ipc::WindowAction::Snap(pos) => {
                            night_binds::snap_focused_window(pos);
                        }
                        night_core::ipc::WindowAction::Swallow => {
                            if let Some(focused) = pinnacle_api::window::get_focused() {
                                for w in pinnacle_api::window::get_all() {
                                    if w.id() != focused.id() && night_rules::swallow::is_terminal_app(&w.app_id()) {
                                        night_rules::swallow::swallow(&focused, &w);
                                        break;
                                    }
                                }
                            }
                        }
                        night_core::ipc::WindowAction::Unswallow => {
                            night_rules::swallow::unswallow_all();
                        }
                    }
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Ok
                }
                IpcCommand::FocusOutput(nav) => {
                    match nav {
                        night_core::ipc::OutputNav::Next => night_tags::focus_next_output(),
                        night_core::ipc::OutputNav::Prev => night_tags::focus_prev_output(),
                        night_core::ipc::OutputNav::Name(n) => night_tags::focus_output_by_name(&n),
                        night_core::ipc::OutputNav::Swap => night_tags::swap_active_outputs(),
                        night_core::ipc::OutputNav::Mirror => night_tags::mirror_active_output_to_next(),
                    }
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Ok
                }
                IpcCommand::MoveWindowToOutput(nav) => {
                    match nav {
                        night_core::ipc::OutputNav::Next => night_tags::move_focused_to_next_output(),
                        night_core::ipc::OutputNav::Prev => night_tags::move_focused_to_prev_output(),
                        night_core::ipc::OutputNav::Name(n) => night_tags::move_focused_to_output_by_name(&n),
                        night_core::ipc::OutputNav::Swap => night_tags::swap_active_outputs(),
                        night_core::ipc::OutputNav::Mirror => night_tags::mirror_active_output_to_next(),
                    }
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Ok
                }
                IpcCommand::Dpms(action) => {
                    match action {
                        night_core::ipc::DpmsAction::On => night_tags::set_dpms(true),
                        night_core::ipc::DpmsAction::Off => night_tags::set_dpms(false),
                        night_core::ipc::DpmsAction::Toggle => night_tags::toggle_dpms(),
                    }
                    IpcResponse::Ok
                }
                IpcCommand::Volume(action) => {
                    let vol = night_core::audio::handle_volume_action(action);
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Volume(vol)
                }
                IpcCommand::GetBattery => {
                    IpcResponse::Battery(night_core::power::read_battery())
                }
                IpcCommand::GetVolume => {
                    IpcResponse::Volume(night_core::audio::get_volume())
                }
                IpcCommand::ShowLauncher => {
                    night_binds::show_launcher_modal();
                    IpcResponse::Ok
                }
                IpcCommand::GetNetwork => {
                    IpcResponse::Network(night_core::network::read_network())
                }
                IpcCommand::GetSystemStats => {
                    IpcResponse::SystemStats(night_core::sysinfo::read_system_stats())
                }
                IpcCommand::GetBluetooth => {
                    IpcResponse::Bluetooth(night_core::bluetooth::read_bluetooth())
                }
                IpcCommand::GetAudioSinks => {
                    IpcResponse::AudioSinks(night_core::audio::list_sinks())
                }
                IpcCommand::SetAudioSink(id) => {
                    night_core::audio::set_default_sink(id);
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Ok
                }
                IpcCommand::CycleAudioSink => {
                    night_core::audio::cycle_sink();
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Ok
                }
                IpcCommand::ShowAudioSinkMenu => {
                    night_binds::show_audio_sink_modal();
                    IpcResponse::Ok
                }
                IpcCommand::Backlight(action) => {
                    let bl = night_core::backlight::handle_backlight_action(action);
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Backlight(bl)
                }
                IpcCommand::GetBacklight => {
                    IpcResponse::Backlight(night_core::backlight::get_backlight())
                }
                IpcCommand::ToggleDnd => {
                    let dnd = night_core::notifications::NotificationHistory::toggle_dnd();
                    night_core::ipc::broadcast_status(current_desktop_status());
                    IpcResponse::Dnd(dnd)
                }
                IpcCommand::GetDnd => {
                    IpcResponse::Dnd(night_core::notifications::NotificationHistory::is_dnd())
                }
                IpcCommand::ShowNotifications => {
                    night_binds::show_notifications_modal();
                    IpcResponse::Ok
                }
                IpcCommand::SaveSession => {
                    let state = SessionState::load().unwrap_or(SessionState {
                        active_tag: "1".into(),
                        last_reload_timestamp: 0,
                    });
                    let user_cfg = UserConfig::load();
                    let mut windows = Vec::new();
                    for win in pinnacle_api::window::get_all() {
                        windows.push(night_core::session_restore::SavedWindow {
                            app_id: win.app_id().to_string(),
                            title: win.title().to_string(),
                            tag: state.active_tag.clone(),
                            is_floating: win.floating(),
                        });
                    }
                    night_core::session_restore::SavedSession::save(
                        &state.active_tag,
                        user_cfg.general.theme.name(),
                        windows,
                    );
                    IpcResponse::Ok
                }
                IpcCommand::RestoreSession => {
                    let saved = night_core::session_restore::SavedSession::load();
                    if let Some(ref s) = saved {
                        info!(windows = s.windows.len(), "Restoring saved desktop session");
                        for w in &s.windows {
                            if which::which(&w.app_id).is_ok() {
                                night_core::exec::spawn_shell(&w.app_id);
                            }
                        }
                    }
                    IpcResponse::SavedSession(saved)
                }
                IpcCommand::SetLayoutProfile(name) => {
                    if let Some(profile) = night_layout::LayoutProfile::parse(&name) {
                        profile.apply();
                        night_binds::show_profile_osd(profile.name());
                        night_core::ipc::broadcast_status(current_desktop_status());
                        IpcResponse::Profile(profile.name().to_string())
                    } else {
                        IpcResponse::Error(format!("Unknown layout profile: {name}"))
                    }
                }
                IpcCommand::ShowOsd(target) => {
                    match target.to_lowercase().as_str() {
                        "volume" => {
                            if let Some(vol) = night_core::audio::get_volume() {
                                night_binds::show_volume_osd(vol.percentage, vol.is_muted);
                            }
                        }
                        "brightness" => {
                            if let Some(bl) = night_core::backlight::get_backlight() {
                                night_binds::show_brightness_osd(bl.percentage);
                            }
                        }
                        "dnd" => {
                            let dnd = night_core::notifications::NotificationHistory::is_dnd();
                            night_binds::show_dnd_osd(dnd);
                        }
                        "gamma" => {
                            let temp = night_core::gamma::current_temperature();
                            night_binds::show_gamma_osd(temp);
                        }
                        "layout" => {
                            let layout = night_layout::current_layout();
                            night_binds::show_layout_osd(layout.name());
                        }
                        _ => {
                            night_binds::show_profile_osd(&target);
                        }
                    }
                    IpcResponse::Ok
                }
                IpcCommand::Subscribe => {
                    IpcResponse::Status(current_desktop_status())
                }
            },
            current_desktop_status,
        )
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
