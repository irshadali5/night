//! 🌙 Night Keybindings Management
//! Centralized registration for application shortcuts, window actions, tag switching, directional navigation, and mouse drags.

use night_core::exec::{spawn, spawn_shell};
use night_core::{Defaults, UserConfig};
use night_tags::DEFAULT_TAG_NAMES;
use pinnacle_api::input::{self, Mod, MouseButton};
use pinnacle_api::util::Direction;
use pinnacle_api::{output, pinnacle, window};
use tracing::info;

/// Registers all user keybindings and interactive mouse bindings with the Pinnacle compositor
pub fn register_all() {
    info!("Registering desktop keybindings and interactive mouse controls...");
    let user_cfg = UserConfig::load();

    // -------------------------------------------------------------------------
    // 1. Application Launchers & Terminals
    // -------------------------------------------------------------------------

    // Super + Return -> Alacritty
    input::keybind(Mod::SUPER, "Return").on_press(|| {
        spawn(Defaults::TERMINAL, &[]);
    });

    // Super + Shift + Return -> Rio (WebGPU Terminal)
    input::keybind(Mod::SUPER | Mod::SHIFT, "Return").on_press(|| {
        spawn(Defaults::ALT_TERMINAL, &[]);
    });

    // Super + Space -> Anyrun (Modular Runner)
    input::keybind(Mod::SUPER, "space").on_press(|| {
        spawn(Defaults::LAUNCHER, &[]);
    });

    // Super + d -> Yofi (Minimal Pure-Rust Launcher)
    input::keybind(Mod::SUPER, 'd').on_press(|| {
        spawn(Defaults::MINIMAL_LAUNCHER, &[]);
    });

    // Super + e -> Favnyr (Modern Rust+Slint GUI File Manager)
    let fm = user_cfg.apps.file_manager.clone();
    input::keybind(Mod::SUPER, 'e').on_press(move || {
        spawn(&fm, &[]);
    });

    // Super + y -> Yazi (Async Terminal File Manager)
    let term = user_cfg.apps.terminal.clone();
    let term_fm = user_cfg.apps.term_file_manager.clone();
    input::keybind(Mod::SUPER, 'y').on_press(move || {
        spawn(&term, &["-e", &term_fm]);
    });

    // Super + b -> Zen Browser
    let browser = user_cfg.apps.browser.clone();
    input::keybind(Mod::SUPER, 'b').on_press(move || {
        spawn(&browser, &[]);
    });

    // Super + Shift + b -> Brave Browser
    let alt_browser = user_cfg.apps.alt_browser.clone();
    input::keybind(Mod::SUPER | Mod::SHIFT, 'b').on_press(move || {
        spawn(&alt_browser, &[]);
    });

    // Super + Escape -> Lock Session (Veila - Pure Rust Wayland Screen Locker)
    let lock_cmd = user_cfg.apps.lock_cmd.clone();
    input::keybind(Mod::SUPER, "Escape").on_press(move || {
        spawn_shell(&lock_cmd);
    });

    // Super + grave (tilde) -> Toggle Quake-style Dropdown Scratchpad Terminal
    input::keybind(Mod::SUPER, "grave").on_press(|| {
        toggle_scratchpad();
    });

    // -------------------------------------------------------------------------
    // 2. Window Layout, Navigation & Directional Focus
    // -------------------------------------------------------------------------

    // Super + Tab -> Cycle Tiling Layout (MasterStack -> Dwindle -> Spiral -> Fair)
    input::keybind(Mod::SUPER, "Tab").on_press(|| {
        night_layout::cycle_layout();
    });

    // Super + equal -> Increase Gaps
    input::keybind(Mod::SUPER, "equal").on_press(|| {
        night_layout::adjust_gaps(2.0);
    });

    // Super + minus -> Decrease Gaps
    input::keybind(Mod::SUPER, "minus").on_press(|| {
        night_layout::adjust_gaps(-2.0);
    });

    // Super + f -> Toggle Fullscreen
    input::keybind(Mod::SUPER, 'f').on_press(|| {
        if let Some(focused) = window::get_focused() {
            focused.toggle_fullscreen();
        }
    });

    // Super + Shift + Space -> Toggle Floating Mode
    input::keybind(Mod::SUPER | Mod::SHIFT, "space").on_press(|| {
        if let Some(focused) = window::get_focused() {
            focused.toggle_floating();
        }
    });

    // Directional Window Navigation (Vim Keys: h/j/k/l)
    let nav_keys = [
        ('h', Direction::Left),
        ('j', Direction::Down),
        ('k', Direction::Up),
        ('l', Direction::Right),
    ];

    for (key, dir) in nav_keys {
        // Super + <key>: Move focus in direction
        input::keybind(Mod::SUPER, key).on_press(move || {
            if let Some(win) = window::get_focused() {
                if let Some(target) = win.in_direction(dir).next() {
                    target.set_focused(true);
                }
            }
        });

        // Super + Shift + <key>: Swap window with neighbor in direction
        input::keybind(Mod::SUPER | Mod::SHIFT, key).on_press(move || {
            if let Some(win) = window::get_focused() {
                if let Some(target) = win.in_direction(dir).next() {
                    win.swap(&target);
                }
            }
        });
    }

    // Directional Arrow Keys Support
    let arrow_keys = [
        ("Left", Direction::Left),
        ("Down", Direction::Down),
        ("Up", Direction::Up),
        ("Right", Direction::Right),
    ];

    for (key_name, dir) in arrow_keys {
        input::keybind(Mod::SUPER, key_name).on_press(move || {
            if let Some(win) = window::get_focused() {
                if let Some(target) = win.in_direction(dir).next() {
                    target.set_focused(true);
                }
            }
        });
    }

    // -------------------------------------------------------------------------
    // 3. Interactive Mouse Controls (Super + Drag / Super + Right-Drag)
    // -------------------------------------------------------------------------

    // Super + Left Mouse Button -> Interactive Window Move
    input::mousebind(Mod::SUPER, MouseButton::Left).on_press(|| {
        window::begin_move(MouseButton::Left);
    });

    // Super + Right Mouse Button -> Interactive Window Resize
    input::mousebind(Mod::SUPER, MouseButton::Right).on_press(|| {
        window::begin_resize(MouseButton::Right);
    });

    // -------------------------------------------------------------------------
    // 4. Window Management & Session
    // -------------------------------------------------------------------------

    // Super + Shift + q -> Close focused window
    input::keybind(Mod::SUPER | Mod::SHIFT, 'q').on_press(|| {
        if let Some(focused) = window::get_focused() {
            focused.close();
        }
    });

    // Super + Shift + x -> Exit Pinnacle compositor
    input::keybind(Mod::SUPER | Mod::SHIFT, 'x').on_press(|| {
        pinnacle::quit();
    });

    // Super + Shift + e -> Snowcap Quit Confirmation Prompt
    input::keybind(Mod::SUPER | Mod::SHIFT, 'e').on_press(|| {
        show_quit_prompt();
    });

    // Super + slash -> Snowcap Interactive Keybinds Cheatsheet Overlay
    input::keybind(Mod::SUPER, "slash").on_press(|| {
        show_bind_overlay();
    });

    // Super + Shift + r -> Reload config
    input::keybind(Mod::SUPER | Mod::SHIFT, 'r').on_press(|| {
        pinnacle::reload_config();
    });

    // -------------------------------------------------------------------------
    // 5. Screenshots & Screen Recording
    // -------------------------------------------------------------------------

    // PrintScreen -> Interactive Area Screenshot to Satty
    input::keybind(Mod::empty(), "Print").on_press(|| {
        spawn_shell("wayshot -s \"$(slurp)\" --stdout | satty --filename -");
    });

    // Super + PrintScreen -> Full Screen Screenshot to ~/Pictures
    input::keybind(Mod::SUPER, "Print").on_press(|| {
        spawn_shell("wayshot -f ~/Pictures/Screenshot_$(date +%Y-%m-%d_%H-%M-%S).png");
    });

    // Super + Alt + r -> Hardware-Accelerated Recording Toggle (wl-screenrec)
    input::keybind(Mod::SUPER | Mod::ALT, 'r').on_press(|| {
        spawn_shell("mkdir -p ~/Videos/Recordings && wl-screenrec -f ~/Videos/Recordings/rec_$(date +%Y-%m-%d_%H-%M-%S).mp4");
    });

    // -------------------------------------------------------------------------
    // 6. Hardware Multimedia Keys (SwayOSD)
    // -------------------------------------------------------------------------

    // Volume Up
    input::keybind(Mod::empty(), "XF86AudioRaiseVolume").on_press(|| {
        spawn(Defaults::OSD_CLIENT, &["--output-volume", "raise"]);
    });

    // Volume Down
    input::keybind(Mod::empty(), "XF86AudioLowerVolume").on_press(|| {
        spawn(Defaults::OSD_CLIENT, &["--output-volume", "lower"]);
    });

    // Audio Mute Toggle
    input::keybind(Mod::empty(), "XF86AudioMute").on_press(|| {
        spawn(Defaults::OSD_CLIENT, &["--output-volume", "mute-toggle"]);
    });

    // Brightness Up
    input::keybind(Mod::empty(), "XF86MonBrightnessUp").on_press(|| {
        spawn(Defaults::OSD_CLIENT, &["--brightness", "raise"]);
    });

    // Brightness Down
    input::keybind(Mod::empty(), "XF86MonBrightnessDown").on_press(|| {
        spawn(Defaults::OSD_CLIENT, &["--brightness", "lower"]);
    });

    // -------------------------------------------------------------------------
    // 7. Dynamic Tag Navigation (AwesomeWM Style: Super + [1..9])
    // -------------------------------------------------------------------------
    for (i, name) in DEFAULT_TAG_NAMES.iter().enumerate() {
        let key_char = char::from_digit((i + 1) as u32, 10).unwrap();
        let tag_name = (*name).to_string();

        // Super + <digit>: Switch to tag on focused output
        let target_view = tag_name.clone();
        input::keybind(Mod::SUPER, key_char).on_press(move || {
            night_tags::switch_to_tag(&target_view);
        });

        // Super + Shift + <digit>: Move focused window to tag on focused output
        let target_move = tag_name.clone();
        input::keybind(Mod::SUPER | Mod::SHIFT, key_char).on_press(move || {
            night_tags::move_focused_to_tag(&target_move);
        });
    }

    // -------------------------------------------------------------------------
    // 8. Multi-Monitor Focus & Window Movement
    // -------------------------------------------------------------------------

    // Super + o -> Focus Next Display Monitor
    input::keybind(Mod::SUPER, 'o').on_press(|| {
        night_tags::focus_next_output();
    });

    // Super + Shift + o -> Move Focused Window to Next Display Monitor
    input::keybind(Mod::SUPER | Mod::SHIFT, 'o').on_press(|| {
        night_tags::move_focused_to_next_output();
    });

    // Super + period (.) -> Focus Next Display Monitor
    input::keybind(Mod::SUPER, "period").on_press(|| {
        night_tags::focus_next_output();
    });

    // Super + comma (,) -> Focus Previous Display Monitor
    input::keybind(Mod::SUPER, "comma").on_press(|| {
        night_tags::focus_prev_output();
    });

    // Super + Shift + period (>) -> Move Window to Next Display Monitor
    input::keybind(Mod::SUPER | Mod::SHIFT, "period").on_press(|| {
        night_tags::move_focused_to_next_output();
    });

    // Super + Shift + comma (<) -> Move Window to Previous Display Monitor
    input::keybind(Mod::SUPER | Mod::SHIFT, "comma").on_press(|| {
        night_tags::move_focused_to_prev_output();
    });

    // Super + Shift + Escape -> Toggle DPMS Display Power
    input::keybind(Mod::SUPER | Mod::SHIFT, "Escape").on_press(|| {
        night_tags::toggle_dpms();
    });

    info!("All keybindings and interactive mouse controls registered successfully.");
}

/// Toggles the interactive Quake-style dropdown scratchpad terminal
pub fn toggle_scratchpad() {
    let all_windows = window::get_all().collect::<Vec<_>>();
    let scratch = all_windows
        .into_iter()
        .find(|w| w.app_id().to_lowercase() == "night-scratchpad");

    if let Some(scratch_win) = scratch {
        if let Some(active_tag) = output::get_focused().and_then(|op| op.active_tags().next()) {
            if scratch_win.is_on_active_tag() {
                scratch_win.set_tag(&active_tag, false);
            } else {
                scratch_win.set_tag(&active_tag, true);
                scratch_win.raise();
                scratch_win.set_focused(true);
            }
        }
    } else {
        spawn("alacritty", &["--class", "night-scratchpad"]);
    }
}

/// Displays the interactive Tokyo Night keybinding cheatsheet overlay via Snowcap
pub fn show_bind_overlay() {
    let mut overlay = pinnacle_api::snowcap::BindOverlay::new();
    overlay.border_radius = 12.0;
    overlay.border_thickness = 2.0;
    overlay.background_color = [0.10, 0.11, 0.15, 0.92].into(); // Tokyo Night Dark #1a1b26
    overlay.border_color = [0.48, 0.64, 0.97].into(); // Tokyo Night Blue #7aa2f7
    overlay.width = 760;
    overlay.height = 560;
    overlay.show();
}

/// Displays the interactive Tokyo Night quit confirmation prompt via Snowcap
pub fn show_quit_prompt() {
    let mut prompt = pinnacle_api::snowcap::QuitPrompt::new();
    prompt.border_radius = 12.0;
    prompt.border_thickness = 2.0;
    prompt.background_color = [0.10, 0.11, 0.15, 0.95].into();
    prompt.border_color = [0.97, 0.46, 0.56].into(); // Tokyo Night Red #f7768e
    prompt.width = 320;
    prompt.height = 140;
    prompt.show();
}

