//! 🌙 Tokyo Night Desktop Command Palette
//! Fast keyboard-driven command palette overlay providing quick access to all desktop operations:
//! Launcher, Window Switcher, Profiles, Themes, Snipping, Audio, DND, Gamma, Settings, and Session Power.

use pinnacle_api::experimental::snowcap_api::layer::{
    new_widget, ExclusiveZone, KeyboardInteractivity, ZLayer,
};
use pinnacle_api::experimental::snowcap_api::widget::{
    column::Column,
    container::Container,
    font::{Family, Font, Weight},
    text::{self, Text},
    Alignment, Background, Length, Padding, WidgetDef,
};
use pinnacle_api::Keysym;
use tracing::info;

/// Desktop action item displayed within the palette
#[derive(Clone, Debug)]
pub struct PaletteAction {
    pub key: &'static str,
    pub icon: &'static str,
    pub title: &'static str,
    pub shortcut: &'static str,
}

#[derive(Clone, Debug)]
pub struct CommandPaletteModal {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: Font,
    pub width: u32,
    pub height: u32,
    pub actions: Vec<PaletteAction>,
}

impl Default for CommandPaletteModal {
    fn default() -> Self {
        Self {
            border_radius: 14.0,
            border_thickness: 2.0,
            background_color: [0.10, 0.11, 0.15, 0.96], // Tokyo Night Dark #1a1b26
            border_color: [0.48, 0.64, 0.97, 1.0],     // Tokyo Night Blue #7aa2f7
            font: Font::new_with_family(Family::Name("Ubuntu".into())),
            width: 700,
            height: 480,
            actions: vec![
                PaletteAction { key: "1", icon: "󰀻", title: "Application Launcher", shortcut: "Super + Space" },
                PaletteAction { key: "2", icon: "󱂬", title: "Window Switcher & Overview", shortcut: "Super + W" },
                PaletteAction { key: "3", icon: "󰖲", title: "Tiling Layout Profile Presets", shortcut: "Super + Alt + 1..4" },
                PaletteAction { key: "4", icon: "󰏘", title: "Tokyo Night Theme Variant", shortcut: "Super + T" },
                PaletteAction { key: "5", icon: "󰄄", title: "Snipping & Screenshot Studio", shortcut: "Super + Shift + S" },
                PaletteAction { key: "6", icon: "󰕾", title: "Audio Output Sink Selector", shortcut: "Super + Shift + A" },
                PaletteAction { key: "7", icon: "󰂛", title: "Do Not Disturb (DND) Mode", shortcut: "Super + Shift + N" },
                PaletteAction { key: "8", icon: "󰃠", title: "Night Light Gamma Temperature", shortcut: "Super + Alt + G" },
                PaletteAction { key: "9", icon: "", title: "Desktop Control Center GUI", shortcut: "Super + Alt + ," },
                PaletteAction { key: "0", icon: "󰌾", title: "Lock Desktop Session", shortcut: "Super + Alt + L" },
                PaletteAction { key: "G", icon: "󰕰", title: "Workspace Grid (Exposé)", shortcut: "Super + G" },
                PaletteAction { key: "S", icon: "󰒲", title: "Sleep / Suspend System", shortcut: "Super + Alt + S" },
                PaletteAction { key: "X", icon: "󰗽", title: "Logout Desktop Session", shortcut: "Super + Shift + Esc" },
            ],
        }
    }
}

impl pinnacle_api::experimental::snowcap_api::widget::Program for CommandPaletteModal {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(&self) -> Option<WidgetDef<Self::Message>> {
        let title = Text::new("⚡ Tokyo Night Command Palette").style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(18.0),
        );

        let subtitle = Text::new(
            "Press [Key] for instant execution  •  [Esc] Dismiss",
        )
        .style(text::Style::new().font(self.font.clone()).pixels(12.0));

        let mut children: Vec<WidgetDef<Self::Message>> = Vec::new();
        children.push(title.into());
        children.push(Text::new("").style(text::Style::new().pixels(4.0)).into());
        children.push(subtitle.into());
        children.push(Text::new("").style(text::Style::new().pixels(12.0)).into());

        for action in &self.actions {
            let row = format!(
                "  [{}]  {}  {:<32} {:>24}",
                action.key, action.icon, action.title, action.shortcut
            );

            let row_text = Text::new(row).style(
                text::Style::new()
                    .font(self.font.clone().weight(Weight::Normal))
                    .pixels(13.0),
            );

            children.push(row_text.into());
            children.push(Text::new("").style(text::Style::new().pixels(4.0)).into());
        }

        let widget = Container::new(Column::new_with_children(children))
            .width(Length::Fixed(self.width as f32))
            .height(Length::Fixed(self.height as f32))
            .padding(Padding {
                top: 20.0,
                right: 24.0,
                bottom: 20.0,
                left: 24.0,
            })
            .vertical_alignment(Alignment::Start)
            .horizontal_alignment(Alignment::Start)
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

/// Displays the interactive Tokyo Night Command Palette via Snowcap
pub fn show_command_palette() {
    let modal = CommandPaletteModal::default();

    let _ = new_widget(
        modal,
        None,
        KeyboardInteractivity::Exclusive,
        ExclusiveZone::Respect,
        ZLayer::Overlay,
    )
    .map(|handle| {
        handle.on_key_press(move |h, key, _mods| {
            h.close();

            match key {
                // [1] App Launcher
                Keysym::_1 => {
                    info!("Palette: opening application launcher");
                    crate::launcher::show_launcher_modal();
                }
                // [2] Window Switcher
                Keysym::_2 => {
                    info!("Palette: opening window switcher");
                    crate::switcher::show_window_switcher();
                }
                // [3] Layout Profile
                Keysym::_3 => {
                    info!("Palette: cycling layout profile");
                    let cur_tag = night_core::SessionState::load()
                        .map(|s| s.active_tag)
                        .unwrap_or_else(|| "1".into());
                    let cur_layout = night_layout::get_layout_for_tag(&cur_tag);
                    let next_profile = match cur_layout {
                        night_layout::LayoutKind::MasterStack => night_layout::LayoutProfile::Reading,
                        night_layout::LayoutKind::Dwindle => night_layout::LayoutProfile::Multitasking,
                        night_layout::LayoutKind::Spiral => night_layout::LayoutProfile::Coding,
                        night_layout::LayoutKind::Fair => night_layout::LayoutProfile::Ultrawide,
                    };
                    next_profile.apply();
                    crate::osd::show_profile_osd(next_profile.name());
                }
                // [4] Theme Variant
                Keysym::_4 => {
                    info!("Palette: cycling Tokyo Night theme variant");
                    let mut cfg = night_core::UserConfig::load();
                    let next_theme = cfg.general.theme.next();
                    cfg.general.theme = next_theme;
                    let _ = cfg.save();
                    night_core::exec::spawn("nightctl", &["theme", &next_theme.name().to_lowercase()]);
                    crate::osd::show_layout_osd(&format!("Theme: {}", next_theme.name()));
                }
                // [5] Snipping
                Keysym::_5 => {
                    info!("Palette: opening snipping modal");
                    crate::snipping::show_snipping_modal();
                }
                // [6] Audio Sinks
                Keysym::_6 => {
                    info!("Palette: opening audio sink switcher");
                    crate::audio_ui::show_audio_sink_modal();
                }
                // [7] DND
                Keysym::_7 => {
                    info!("Palette: toggling DND");
                    let is_dnd = night_core::notifications::NotificationHistory::toggle_dnd();
                    crate::osd::show_dnd_osd(is_dnd);
                }
                // [8] Gamma
                Keysym::_8 => {
                    info!("Palette: toggling night light gamma");
                    let temp = night_core::gamma::toggle_night_light();
                    crate::osd::show_gamma_osd(temp);
                }
                // [9] Settings
                Keysym::_9 => {
                    info!("Palette: launching desktop control center");
                    night_core::exec::spawn("night-settings", &[]);
                }
                // [0] Lock
                Keysym::_0 => {
                    info!("Palette: locking desktop session");
                    night_core::exec::spawn("night-session", &["lock"]);
                }
                // [G] Workspace Grid
                Keysym::g | Keysym::G => {
                    info!("Palette: opening workspace grid matrix");
                    crate::grid::show_workspace_grid();
                }
                // [S] Suspend
                Keysym::s | Keysym::S => {
                    info!("Palette: suspending system to RAM");
                    night_core::exec::spawn_shell("systemctl suspend 2>/dev/null");
                }
                // [X] Logout
                Keysym::x | Keysym::X => {
                    info!("Palette: logging out desktop session");
                    night_core::exec::spawn("night-session", &["logout"]);
                }
                _ => {}
            }
        });
    });
}
