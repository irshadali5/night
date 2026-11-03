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

    // Super + s -> Toggle Sticky Window (pinned across all workspace tags)
    input::keybind(Mod::SUPER, 's').on_press(|| {
        toggle_sticky();
    });

    // Super + p -> Toggle Picture-in-Picture (PiP) Floating Mode
    input::keybind(Mod::SUPER, 'p').on_press(|| {
        toggle_pip();
    });

    // Super + c -> Center Focused Floating Window
    input::keybind(Mod::SUPER, 'c').on_press(|| {
        center_focused_window();
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

    // Super + BackSpace -> Snowcap Tokyo Night Session Flyout Menu
    input::keybind(Mod::SUPER, "BackSpace").on_press(|| {
        show_session_menu();
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

/// Toggles sticky state on the focused window (visible across all workspace tags)
pub fn toggle_sticky() {
    if let (Some(focused_win), Some(focused_op)) = (window::get_focused(), output::get_focused()) {
        let all_count = DEFAULT_TAG_NAMES.len();
        let win_tags_count = focused_win.tags().count();
        if win_tags_count >= all_count {
            if let Some(active_tag) = focused_op.active_tags().next() {
                for name in DEFAULT_TAG_NAMES {
                    if let Some(t) = pinnacle_api::tag::get_on_output(name, &focused_op) {
                        if t != active_tag {
                            focused_win.set_tag(&t, false);
                        }
                    }
                }
            }
            info!("Focused window unpinned from sticky mode");
        } else {
            for name in DEFAULT_TAG_NAMES {
                if let Some(t) = pinnacle_api::tag::get_on_output(name, &focused_op) {
                    focused_win.set_tag(&t, true);
                }
            }
            info!("Focused window pinned sticky across all workspace tags");
        }
    }
}

/// Toggles Picture-in-Picture (PiP) mode (bottom-right floating + sticky)
pub fn toggle_pip() {
    if let (Some(focused_win), Some(focused_op)) = (window::get_focused(), output::get_focused()) {
        if focused_win.floating() && focused_win.tags().count() > 1 {
            focused_win.set_floating(false);
            if let Some(active_tag) = focused_op.active_tags().next() {
                for name in DEFAULT_TAG_NAMES {
                    if let Some(t) = pinnacle_api::tag::get_on_output(name, &focused_op) {
                        if t != active_tag {
                            focused_win.set_tag(&t, false);
                        }
                    }
                }
            }
            info!("Window restored from Picture-in-Picture mode");
        } else {
            focused_win.set_floating(true);
            for name in DEFAULT_TAG_NAMES {
                if let Some(t) = pinnacle_api::tag::get_on_output(name, &focused_op) {
                    focused_win.set_tag(&t, true);
                }
            }
            let op_size = focused_op
                .logical_size()
                .unwrap_or(pinnacle_api::util::Size { w: 1920, h: 1080 });
            let op_loc = focused_op
                .loc()
                .unwrap_or(pinnacle_api::util::Point { x: 0, y: 0 });
            let pip_w = 480;
            let pip_h = 270;
            let pad = 24;
            let x = op_loc.x + (op_size.w as i32) - pip_w - pad;
            let y = op_loc.y + (op_size.h as i32) - pip_h - pad;
            focused_win.set_geometry(x, y, pip_w as u32, pip_h as u32);
            focused_win.raise();
            info!("Window placed in Picture-in-Picture bottom-right overlay");
        }
    }
}

/// Centers the currently focused floating window on the active display
pub fn center_focused_window() {
    if let (Some(focused_win), Some(focused_op)) = (window::get_focused(), output::get_focused()) {
        focused_win.set_floating(true);
        let op_size = focused_op
            .logical_size()
            .unwrap_or(pinnacle_api::util::Size { w: 1920, h: 1080 });
        let op_loc = focused_op
            .loc()
            .unwrap_or(pinnacle_api::util::Point { x: 0, y: 0 });
        let win_size = focused_win
            .size()
            .unwrap_or(pinnacle_api::util::Size { w: 800, h: 500 });
        let x = op_loc.x + ((op_size.w as i32) - (win_size.w as i32)) / 2;
        let y = op_loc.y + ((op_size.h as i32) - (win_size.h as i32)) / 2;
        focused_win.set_geometry(x, y, win_size.w, win_size.h);
        focused_win.raise();
        info!("Centered focused floating window on active display");
    }
}

/// Interactive Tokyo Night Session Manager Flyout Modal
#[derive(Clone, Debug)]
pub struct SessionFlyout {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: pinnacle_api::experimental::snowcap_api::widget::font::Font,
    pub width: u32,
    pub height: u32,
}

impl Default for SessionFlyout {
    fn default() -> Self {
        use pinnacle_api::experimental::snowcap_api::widget::font::{Family, Font};
        Self {
            border_radius: 14.0,
            border_thickness: 2.0,
            background_color: [0.10, 0.11, 0.15, 0.96],
            border_color: [0.48, 0.64, 0.97, 1.0],
            font: Font::new_with_family(Family::Name("Ubuntu".into())),
            width: 560,
            height: 240,
        }
    }
}

impl pinnacle_api::experimental::snowcap_api::widget::Program for SessionFlyout {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(
        &self,
    ) -> Option<pinnacle_api::experimental::snowcap_api::widget::WidgetDef<Self::Message>> {
        use pinnacle_api::experimental::snowcap_api::widget::{
            column::Column,
            container::Container,
            font::Weight,
            row::Row,
            text::{self, Text},
            Alignment, Background, Length, Padding,
        };

        let title = Text::new("🌙 Tokyo Night Session Manager").style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(20.0),
        );

        let subtitle = Text::new("Select an action via keyboard shortcut:")
            .style(text::Style::new().font(self.font.clone()).pixels(13.0));

        let actions_row1 = Row::new_with_children([Text::new(
            "  [L] Lock Screen      [S] Sleep (DPMS)      [R] Reload Config",
        )
        .style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(14.0),
        )
        .into()]);

        let actions_row2 = Row::new_with_children([Text::new(
            "  [Q] Exit Desktop     [P] Power Off         [B] Reboot System",
        )
        .style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(14.0),
        )
        .into()]);

        let hint = Text::new("Press ESCAPE to cancel")
            .style(text::Style::new().font(self.font.clone()).pixels(12.0));

        let widget = Container::new(Column::new_with_children([
            title.into(),
            Text::new("").style(text::Style::new().pixels(6.0)).into(),
            subtitle.into(),
            Text::new("").style(text::Style::new().pixels(12.0)).into(),
            actions_row1.into(),
            Text::new("").style(text::Style::new().pixels(8.0)).into(),
            actions_row2.into(),
            Text::new("").style(text::Style::new().pixels(14.0)).into(),
            hint.into(),
        ]))
        .width(Length::Fixed(self.width as f32))
        .height(Length::Fixed(self.height as f32))
        .padding(Padding {
            top: 20.0,
            right: 20.0,
            bottom: 20.0,
            left: 20.0,
        })
        .vertical_alignment(Alignment::Center)
        .horizontal_alignment(Alignment::Center)
        .style(
            pinnacle_api::experimental::snowcap_api::widget::container::Style {
                text_color: None,
                background: Some(Background::Color(self.background_color.into())),
                border: Some(pinnacle_api::experimental::snowcap_api::widget::Border {
                    color: Some(self.border_color.into()),
                    width: Some(self.border_thickness),
                    radius: Some(self.border_radius.into()),
                }),
            },
        );

        Some(widget.into())
    }
}

/// Displays the interactive Tokyo Night Session Flyout Modal via Snowcap
pub fn show_session_menu() {
    let flyout = SessionFlyout::default();
    let lock_cmd = UserConfig::load().apps.lock_cmd;

    use pinnacle_api::experimental::snowcap_api::layer::{
        new_widget, ExclusiveZone, KeyboardInteractivity, ZLayer,
    };
    use pinnacle_api::Keysym;

    let _ = new_widget(
        flyout,
        None,
        KeyboardInteractivity::Exclusive,
        ExclusiveZone::Respect,
        ZLayer::Overlay,
    )
    .map(|handle| {
        handle.on_key_press(move |h, key, _mods| match key {
            Keysym::l | Keysym::L => {
                spawn_shell(&lock_cmd);
                h.close();
            }
            Keysym::s | Keysym::S => {
                night_tags::set_dpms(false);
                h.close();
            }
            Keysym::r | Keysym::R => {
                pinnacle_api::pinnacle::reload_config();
                h.close();
            }
            Keysym::q | Keysym::Q => {
                pinnacle_api::pinnacle::quit();
            }
            Keysym::p | Keysym::P => {
                spawn_shell("systemctl poweroff");
                h.close();
            }
            Keysym::b | Keysym::B => {
                spawn_shell("systemctl reboot");
                h.close();
            }
            _ => {
                h.close();
            }
        });
    });
}


