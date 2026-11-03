//! 📸 Tokyo Night Snipping & Screen Capture Modal
//! Snowcap layer-shell modal providing single-key shortcuts for region snips, window cropping,
//! full-screen captures, clipboard copying, and hardware-accelerated video/GIF recording.

use night_core::exec::spawn_shell;
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

#[derive(Clone, Debug)]
pub struct SnippingModal {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: Font,
    pub width: u32,
    pub height: u32,
}

impl Default for SnippingModal {
    fn default() -> Self {
        Self {
            border_radius: 14.0,
            border_thickness: 2.0,
            background_color: [0.10, 0.11, 0.15, 0.96],
            border_color: [0.48, 0.64, 0.97, 1.0], // Tokyo Night Blue #7aa2f7
            font: Font::new_with_family(Family::Name("Ubuntu".into())),
            width: 660,
            height: 330,
        }
    }
}

impl pinnacle_api::experimental::snowcap_api::widget::Program for SnippingModal {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(&self) -> Option<WidgetDef<Self::Message>> {
        let title = Text::new("📸 Tokyo Night Snipping & Screen Capture").style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(18.0),
        );

        let subtitle = Text::new(
            "Select capture mode with a single key  •  [Esc] / [Q] Cancel",
        )
        .style(text::Style::new().font(self.font.clone()).pixels(12.0));

        let options = [
            (" [R]", "󰩭  Snip Region & Annotate", "slurp selection piped to satty editor"),
            (" [W]", "󰖯  Snip Focused Window", "auto-crops active window coordinates"),
            (" [F]", "󰹑  Full Screen to File", "saved to ~/Pictures/Screenshot_..."),
            (" [C]", "󰅍  Full Screen to Clipboard", "direct copy via wl-copy"),
            (" [V]", "󰕧  Record Screen (MP4 60fps)", "hardware accelerated wl-screenrec"),
            (" [G]", "󰈫  Record Region as GIF", "animated screencast via wl-screenrec"),
        ];

        let mut children: Vec<WidgetDef<Self::Message>> = Vec::new();
        children.push(title.into());
        children.push(Text::new("").style(text::Style::new().pixels(4.0)).into());
        children.push(subtitle.into());
        children.push(Text::new("").style(text::Style::new().pixels(12.0)).into());

        for (key, label, desc) in options {
            let row = format!("{:<5}  {:<30}  •  {}", key, label, desc);
            let row_text = Text::new(row).style(
                text::Style::new()
                    .font(self.font.clone())
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

/// Displays the interactive Tokyo Night Snipping Modal via Snowcap
pub fn show_snipping_modal() {
    let modal = SnippingModal::default();

    let _ = new_widget(
        modal,
        None,
        KeyboardInteractivity::Exclusive,
        ExclusiveZone::Respect,
        ZLayer::Overlay,
    )
    .map(|handle| {
        handle.on_key_press(move |h, key, _mods| {
            // Close the overlay modal immediately before capturing
            h.close();

            match key {
                // [R] Snip Region & Annotate
                Keysym::r | Keysym::R => {
                    info!("Triggered interactive region snip to Satty");
                    spawn_shell("wayshot -s \"$(slurp)\" --stdout | satty --filename -");
                }
                // [W] Snip Focused Window
                Keysym::w | Keysym::W => {
                    info!("Triggered focused window snip to Satty");
                    if let Some(win) = pinnacle_api::window::get_focused() {
                        let loc = win.loc().unwrap_or(pinnacle_api::util::Point { x: 0, y: 0 });
                        let size = win.size().unwrap_or(pinnacle_api::util::Size { w: 1280, h: 720 });
                        let cmd = format!(
                            "wayshot -s \"{},{} {}x{}\" --stdout | satty --filename -",
                            loc.x, loc.y, size.w, size.h
                        );
                        spawn_shell(&cmd);
                    } else {
                        // Fallback to interactive region
                        spawn_shell("wayshot -s \"$(slurp)\" --stdout | satty --filename -");
                    }
                }
                // [F] Full Screen to File
                Keysym::f | Keysym::F => {
                    info!("Triggered full screen screenshot to file");
                    spawn_shell("mkdir -p ~/Pictures/Screenshots && wayshot -f ~/Pictures/Screenshots/Screenshot_$(date +%Y-%m-%d_%H-%M-%S).png");
                }
                // [C] Full Screen to Clipboard
                Keysym::c | Keysym::C => {
                    info!("Triggered full screen screenshot to clipboard");
                    spawn_shell("wayshot --stdout | wl-copy");
                }
                // [V] Record Screen (MP4)
                Keysym::v | Keysym::V => {
                    info!("Triggered screen recording (MP4 60fps)");
                    spawn_shell("mkdir -p ~/Videos/Recordings && wl-screenrec -f ~/Videos/Recordings/rec_$(date +%Y-%m-%d_%H-%M-%S).mp4");
                }
                // [G] Record Region as GIF
                Keysym::g | Keysym::G => {
                    info!("Triggered animated region recording (GIF)");
                    spawn_shell("mkdir -p ~/Videos/Recordings && wl-screenrec -g \"$(slurp)\" --output-format gif -f ~/Videos/Recordings/rec_$(date +%Y-%m-%d_%H-%M-%S).gif");
                }
                // [Esc] / [Q] / other keys just dismissed by h.close()
                _ => {}
            }
        });
    });
}

