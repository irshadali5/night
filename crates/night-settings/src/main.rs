//! 🌙 Night Settings & Control Center
//! Pure-Rust GUI Control Center written in Slint for Project Night.
//! Provides interactive live previews via IPC, editing `night.toml` and `monitors.ron`.

slint::include_modules!();

use clap::Parser;
use night_core::config::UserConfig;
use night_core::ipc::{send_command, DpmsAction, IpcCommand, IpcResponse};
use night_core::monitors::{MonitorMode, MonitorProfile, MonitorTransform, MonitorVrr};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(
    name = "night-settings",
    author = "Night Desktop Team",
    version,
    about = "🌙 Tokyo Night Control Center & System Settings (Slint GUI)"
)]
struct SettingsCli {
    /// Initial settings tab to display: appearance, apps, monitors, power, about
    #[arg(short, long)]
    tab: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = SettingsCli::parse();
    info!("Starting Night Control Center (Slint GUI)");

    let ui = SettingsWindow::new()?;

    // Load configurations from disk
    let user_cfg = Arc::new(Mutex::new(UserConfig::load()));
    let monitor_profiles = Arc::new(Mutex::new(MonitorProfile::load_all()));

    // Populate initial state from user config
    {
        let cfg = user_cfg.lock().unwrap();
        ui.set_inner_gaps(cfg.general.inner_gaps);
        ui.set_outer_gaps(cfg.general.outer_gaps);
        ui.set_border_width(cfg.general.border_width as i32);
        ui.set_autostart_daemons(cfg.general.autostart_daemons);

        ui.set_app_terminal(cfg.apps.terminal.clone().into());
        ui.set_app_alt_terminal(cfg.apps.alt_terminal.clone().into());
        ui.set_app_browser(cfg.apps.browser.clone().into());
        ui.set_app_alt_browser(cfg.apps.alt_browser.clone().into());
        ui.set_app_file_manager(cfg.apps.file_manager.clone().into());
        ui.set_app_term_file_manager(cfg.apps.term_file_manager.clone().into());
        ui.set_app_launcher(cfg.apps.launcher.clone().into());
        ui.set_app_minimal_launcher(cfg.apps.minimal_launcher.clone().into());
        ui.set_app_lock_cmd(cfg.apps.lock_cmd.clone().into());
    }

    // Populate initial monitor profile
    {
        let profiles = monitor_profiles.lock().unwrap();
        if let Some(first) = profiles.first() {
            ui.set_monitor_name(first.name.clone().into());
            ui.set_monitor_scale(first.scale.unwrap_or(1.0));
            if let Some(ref mode) = first.mode {
                ui.set_monitor_width(mode.width as i32);
                ui.set_monitor_height(mode.height as i32);
                ui.set_monitor_refresh(mode.refresh_rate_hz.unwrap_or(60) as i32);
            }
            if let Some(vrr) = first.vrr {
                ui.set_monitor_vrr(match vrr {
                    MonitorVrr::Off => "Off".into(),
                    MonitorVrr::AlwaysOn => "AlwaysOn".into(),
                    MonitorVrr::OnDemand => "OnDemand".into(),
                });
            }
        }
    }

    // Initial tab selection
    if let Some(ref tab_name) = cli.tab {
        let tab_idx = match tab_name.to_lowercase().as_str() {
            "appearance" | "gaps" | "theme" => 0,
            "apps" | "applications" => 1,
            "monitors" | "displays" | "screen" => 2,
            "power" | "session" | "system" => 3,
            "about" => 4,
            _ => 0,
        };
        ui.set_active_tab(tab_idx);
    }

    // Connect to live compositor via IPC (status query)
    let ui_weak = ui.as_weak();
    tokio::spawn(async move {
        match send_command(&IpcCommand::GetStatus).await {
            Ok(IpcResponse::Status(status)) => {
                let _ = ui_weak.upgrade_in_event_loop(move |u| {
                    u.set_ipc_status("Connected (<2ms)".into());
                    u.set_ipc_connected(true);
                    u.set_current_layout(status.layout.into());
                    u.set_active_tag(status.active_tag.into());
                    u.set_output_count(status.output_count as i32);
                    u.set_inner_gaps(status.inner_gaps);
                    u.set_outer_gaps(status.outer_gaps);
                    u.set_is_inhibited(status.inhibited);
                    if let Some(temp) = status.gamma_temp {
                        u.set_gamma_temp(temp as f32);
                    }
                });
            }
            Ok(_) => {
                let _ = ui_weak.upgrade_in_event_loop(|u| {
                    u.set_ipc_status("Connected".into());
                    u.set_ipc_connected(true);
                });
            }
            Err(e) => {
                info!("IPC server offline: {e}");
                let _ = ui_weak.upgrade_in_event_loop(|u| {
                    u.set_ipc_status("Compositor Offline".into());
                    u.set_ipc_connected(false);
                });
            }
        }
    });

    // -----------------------------------------------------------------
    // Callbacks: Appearance & Gaps
    // -----------------------------------------------------------------
    ui.on_apply_gaps_preview(move |inner, outer| {
        tokio::spawn(async move {
            let cmd = IpcCommand::SetGaps {
                inner: Some(inner),
                outer: Some(outer),
            };
            let _ = send_command(&cmd).await;
        });
    });

    let ui_weak = ui.as_weak();
    let cfg_clone = Arc::clone(&user_cfg);
    ui.on_save_appearance_config(move |inner, outer, border| {
        let mut cfg = cfg_clone.lock().unwrap();
        cfg.general.inner_gaps = inner;
        cfg.general.outer_gaps = outer;
        cfg.general.border_width = border.max(0) as u32;

        match cfg.save() {
            Ok(()) => {
                info!("Appearance configuration saved to night.toml");
                let _ = ui_weak.upgrade_in_event_loop(|u| {
                    u.set_notification_msg("Saved appearance settings to ~/.config/night/night.toml".into());
                });
            }
            Err(e) => {
                error!("Failed to save night.toml: {e}");
                let _ = ui_weak.upgrade_in_event_loop(move |u| {
                    u.set_notification_msg(format!("Save error: {e}").into());
                });
            }
        }

        tokio::spawn(async move {
            let _ = send_command(&IpcCommand::SetGaps {
                inner: Some(inner),
                outer: Some(outer),
            })
            .await;
        });
    });

    // -----------------------------------------------------------------
    // Callbacks: Layout Cycle
    // -----------------------------------------------------------------
    let ui_weak = ui.as_weak();
    ui.on_cycle_layout(move || {
        let ui_weak = ui_weak.clone();
        tokio::spawn(async move {
            if let Ok(IpcResponse::Layout(name)) = send_command(&IpcCommand::CycleLayout).await {
                let _ = ui_weak.upgrade_in_event_loop(move |u| {
                    u.set_current_layout(name.into());
                    u.set_notification_msg("Switched tiling layout".into());
                });
            }
        });
    });

    // -----------------------------------------------------------------
    // Callbacks: Default Applications
    // -----------------------------------------------------------------
    let ui_weak = ui.as_weak();
    let cfg_clone = Arc::clone(&user_cfg);
    ui.on_save_apps_config(
        move |term, alt_term, browser, alt_browser, fm, term_fm, launcher, min_launcher, lock| {
            let mut cfg = cfg_clone.lock().unwrap();
            cfg.apps.terminal = term.to_string();
            cfg.apps.alt_terminal = alt_term.to_string();
            cfg.apps.browser = browser.to_string();
            cfg.apps.alt_browser = alt_browser.to_string();
            cfg.apps.file_manager = fm.to_string();
            cfg.apps.term_file_manager = term_fm.to_string();
            cfg.apps.launcher = launcher.to_string();
            cfg.apps.minimal_launcher = min_launcher.to_string();
            cfg.apps.lock_cmd = lock.to_string();

            match cfg.save() {
                Ok(()) => {
                    info!("Default applications saved to night.toml");
                    let _ = ui_weak.upgrade_in_event_loop(|u| {
                        u.set_notification_msg("Saved default applications to ~/.config/night/night.toml".into());
                    });
                }
                Err(e) => {
                    error!("Failed to save night.toml: {e}");
                    let _ = ui_weak.upgrade_in_event_loop(move |u| {
                        u.set_notification_msg(format!("Save error: {e}").into());
                    });
                }
            }
        },
    );

    // -----------------------------------------------------------------
    // Callbacks: Monitors & Displays
    // -----------------------------------------------------------------
    let ui_weak = ui.as_weak();
    let mon_clone = Arc::clone(&monitor_profiles);
    ui.on_save_monitor_config(move |name, scale, width, height, refresh, transform_str, vrr_str| {
        let vrr = match vrr_str.as_str() {
            "AlwaysOn" => MonitorVrr::AlwaysOn,
            "OnDemand" => MonitorVrr::OnDemand,
            _ => MonitorVrr::Off,
        };

        let transform = match transform_str.as_str() {
            "Rotate90" => MonitorTransform::Rotate90,
            "Rotate180" => MonitorTransform::Rotate180,
            "Rotate270" => MonitorTransform::Rotate270,
            "Flipped" => MonitorTransform::Flipped,
            _ => MonitorTransform::Normal,
        };

        let new_profile = MonitorProfile {
            name: name.to_string(),
            scale: Some(scale),
            mode: Some(MonitorMode {
                width: width.max(640) as u32,
                height: height.max(480) as u32,
                refresh_rate_hz: Some(refresh.max(30) as u32),
            }),
            position: None,
            transform: Some(transform),
            vrr: Some(vrr),
        };

        let mut profiles = mon_clone.lock().unwrap();
        if let Some(pos) = profiles.iter().position(|p| p.name == new_profile.name) {
            profiles[pos] = new_profile;
        } else {
            profiles.push(new_profile);
        }

        match MonitorProfile::save_all(&profiles) {
            Ok(()) => {
                info!("Monitor profiles saved to monitors.ron");
                let _ = ui_weak.upgrade_in_event_loop(|u| {
                    u.set_notification_msg("Saved display configuration to ~/.config/night/monitors.ron".into());
                });
                tokio::spawn(async move {
                    let _ = send_command(&IpcCommand::ReloadConfig).await;
                });
            }
            Err(e) => {
                error!("Failed to save monitors.ron: {e}");
                let _ = ui_weak.upgrade_in_event_loop(move |u| {
                    u.set_notification_msg(format!("Save error: {e}").into());
                });
            }
        }
    });

    // -----------------------------------------------------------------
    // Callbacks: System & Power
    // -----------------------------------------------------------------
    ui.on_toggle_dpms(|| {
        tokio::spawn(async move {
            let _ = send_command(&IpcCommand::Dpms(DpmsAction::Toggle)).await;
        });
    });

    let cfg_lock = Arc::clone(&user_cfg);
    ui.on_lock_screen(move || {
        let lock_cmd = {
            let cfg = cfg_lock.lock().unwrap();
            cfg.apps.lock_cmd.clone()
        };
        night_core::exec::spawn_shell(&lock_cmd);
    });

    let ui_weak = ui.as_weak();
    ui.on_reload_compositor(move || {
        let ui_weak = ui_weak.clone();
        tokio::spawn(async move {
            match send_command(&IpcCommand::ReloadConfig).await {
                Ok(_) => {
                    let _ = ui_weak.upgrade_in_event_loop(|u| {
                        u.set_notification_msg("Compositor reloaded configuration".into());
                    });
                }
                Err(e) => {
                    let _ = ui_weak.upgrade_in_event_loop(move |u| {
                        u.set_notification_msg(format!("Compositor offline: {e}").into());
                    });
                }
            }
        });
    });

    ui.on_session_poweroff(|| {
        night_core::exec::spawn_shell("systemctl poweroff");
    });

    ui.on_session_reboot(|| {
        night_core::exec::spawn_shell("systemctl reboot");
    });

    ui.on_set_gamma_temp(move |temp| {
        let t = temp.max(1000.0) as u32;
        tokio::spawn(async move {
            let _ = send_command(&IpcCommand::SetGamma(t)).await;
        });
    });

    let ui_weak = ui.as_weak();
    ui.on_toggle_inhibit(move || {
        let ui_weak = ui_weak.clone();
        tokio::spawn(async move {
            if let Ok(IpcResponse::Inhibited(val)) = send_command(&IpcCommand::ToggleInhibit).await {
                let _ = ui_weak.upgrade_in_event_loop(move |u| {
                    u.set_is_inhibited(val);
                    let msg = if val {
                        "Idle inhibition enabled (Presentation Mode)"
                    } else {
                        "Idle inhibition disabled (Normal power saving)"
                    };
                    u.set_notification_msg(msg.into());
                });
            }
        });
    });

    // Run Slint Event Loop
    ui.run()?;
    Ok(())
}
