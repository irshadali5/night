//! 🌙 Night Keybindings Management
//! Centralized registration for application shortcuts, window actions, tag switching, and media keys.

use night_core::exec::{spawn, spawn_shell};
use night_core::Defaults;
use night_tags::DEFAULT_TAG_NAMES;
use pinnacle_api::input::{self, Mod};
use pinnacle_api::{pinnacle, tag, window};
use tracing::info;

/// Registers all user keybindings with the Pinnacle compositor
pub fn register_all() {
    info!("Registering desktop keybindings...");

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

    // Super + e -> Yazi File Manager in Terminal
    input::keybind(Mod::SUPER, 'e').on_press(|| {
        spawn(Defaults::TERMINAL, &["-e", Defaults::FILE_MANAGER]);
    });

    // -------------------------------------------------------------------------
    // 2. Window Management & Session
    // -------------------------------------------------------------------------

    // Super + Shift + q -> Close focused window
    input::keybind(Mod::SUPER | Mod::SHIFT, 'q').on_press(|| {
        if let Some(focused) = window::get_focused() {
            focused.close();
        }
    });

    // Super + Shift + e -> Exit Pinnacle compositor
    input::keybind(Mod::SUPER | Mod::SHIFT, 'e').on_press(|| {
        pinnacle::quit();
    });

    // Super + Shift + r -> Reload config
    input::keybind(Mod::SUPER | Mod::SHIFT, 'r').on_press(|| {
        pinnacle::reload_config();
    });

    // -------------------------------------------------------------------------
    // 3. Screenshots & Screen Recording
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
    // 4. Hardware Multimedia Keys (SwayOSD)
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
    // 5. Dynamic Tag Navigation (AwesomeWM Style: Super + [1..9])
    // -------------------------------------------------------------------------
    for (i, name) in DEFAULT_TAG_NAMES.iter().enumerate() {
        let key_char = char::from_digit((i + 1) as u32, 10).unwrap();
        let tag_name = (*name).to_string();

        // Super + <digit>: Switch to tag
        let target_view = tag_name.clone();
        input::keybind(Mod::SUPER, key_char).on_press(move || {
            if let Some(t) = tag::get(&target_view) {
                t.switch_to();
            }
        });

        // Super + Shift + <digit>: Move focused window to tag
        let target_move = tag_name.clone();
        input::keybind(Mod::SUPER | Mod::SHIFT, key_char).on_press(move || {
            if let (Some(win), Some(t)) = (window::get_focused(), tag::get(&target_move)) {
                win.move_to_tag(&t);
            }
        });
    }

    info!("All keybindings registered successfully.");
}
