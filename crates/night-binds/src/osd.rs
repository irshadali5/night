//! 🌙 Bihar Night Pure-Rust Visual On-Screen Display (OSD) Engine
//! Ultra-responsive layer-shell HUD overlay displaying instant visual feedback for:
//! Audio Volume, Backlight Brightness, Do Not Disturb (DND), Night Light (Gamma), and Tiling Layout Profiles.
//! Automatically closes smoothly after a non-blocking timeout.

use pinnacle_api::experimental::snowcap_api::layer::{
    new_widget, ExclusiveZone, KeyboardInteractivity, ZLayer,
};
use pinnacle_api::experimental::snowcap_api::widget::{
    column::Column,
    container::Container,
    font::{Family, Font, Weight},
    row::Row,
    text::{self, Text},
    Alignment, Background, Length, Padding, WidgetDef,
};
use std::time::Duration;
use tracing::info;

/// Visual OSD Mode
#[derive(Clone, Debug)]
pub enum OsdKind {
    Volume { percentage: u8, muted: bool },
    Brightness { percentage: u8 },
    Dnd { active: bool },
    Gamma { kelvin: u32 },
    Layout { name: String },
    Profile { name: String },
}

/// OSD Modal definition
#[derive(Clone, Debug)]
pub struct OsdModal {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: Font,
    pub width: u32,
    pub height: u32,
    pub kind: OsdKind,
}

impl Default for OsdModal {
    fn default() -> Self {
        Self {
            border_radius: 12.0,
            border_thickness: 2.0,
            background_color: [0.10, 0.11, 0.15, 0.94], // Bihar Night Dark #1a1b26
            border_color: [0.48, 0.64, 0.97, 1.0],     // Bihar Night Blue #7aa2f7
            font: Font::new_with_family(Family::Name("Ubuntu".into())),
            width: 380,
            height: 90,
            kind: OsdKind::Volume { percentage: 50, muted: false },
        }
    }
}

impl pinnacle_api::experimental::snowcap_api::widget::Program for OsdModal {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(&self) -> Option<WidgetDef<Self::Message>> {
        let (icon, label, pct_bar) = match &self.kind {
            OsdKind::Volume { percentage, muted } => {
                let icon = if *muted {
                    "󰝟"
                } else if *percentage >= 70 {
                    "󰕾"
                } else if *percentage >= 30 {
                    "󰖀"
                } else {
                    "󰕿"
                };
                let status_text = if *muted {
                    "Volume: Muted".to_string()
                } else {
                    format!("Volume: {}%", percentage)
                };
                let bar = if *muted {
                    "[ MUTED ]".to_string()
                } else {
                    make_ascii_bar(*percentage, 24)
                };
                (icon, status_text, bar)
            }
            OsdKind::Brightness { percentage } => {
                let icon = if *percentage >= 75 {
                    "󰃠"
                } else if *percentage >= 50 {
                    "󰃟"
                } else if *percentage >= 25 {
                    "󰃞"
                } else {
                    "󰃝"
                };
                let label = format!("Brightness: {}%", percentage);
                let bar = make_ascii_bar(*percentage, 24);
                (icon, label, bar)
            }
            OsdKind::Dnd { active } => {
                let icon = if *active { "󰂛" } else { "󰂚" };
                let label = if *active {
                    "Do Not Disturb: Active".to_string()
                } else {
                    "Do Not Disturb: Disabled".to_string()
                };
                let bar = if *active {
                    "[ NOTIFICATIONS MUTED ]".to_string()
                } else {
                    "[ NORMAL NOTIFICATIONS ]".to_string()
                };
                (icon, label, bar)
            }
            OsdKind::Gamma { kelvin } => {
                let icon = if *kelvin <= 4500 { "󰖔" } else { "󰖙" };
                let label = format!("Night Light: {}K", kelvin);
                let bar = if *kelvin <= 4500 {
                    "[ WARM NIGHT FILTER ]".to_string()
                } else {
                    "[ NEUTRAL DAYLIGHT ]".to_string()
                };
                (icon, label, bar)
            }
            OsdKind::Layout { name } => {
                let icon = "󰕰";
                let label = format!("Window Layout: {}", name);
                let bar = format!("[ TILING: {} ]", name.to_uppercase());
                (icon, label, bar)
            }
            OsdKind::Profile { name } => {
                let icon = "󰅩";
                let label = format!("Layout Profile: {}", name);
                let bar = format!("[ PRESET: {} ]", name.to_uppercase());
                (icon, label, bar)
            }
        };

        let header_row = Row::new_with_children(vec![
            Text::new(format!("{icon}  ")).style(
                text::Style::new()
                    .font(self.font.clone().weight(Weight::Bold))
                    .pixels(18.0),
            ).into(),
            Text::new(label).style(
                text::Style::new()
                    .font(self.font.clone().weight(Weight::Bold))
                    .pixels(14.0),
            ).into(),
        ])
        .item_alignment(Alignment::Center);

        let bar_text = Text::new(pct_bar).style(
            text::Style::new()
                .font(self.font.clone())
                .pixels(13.0),
        );

        let col = Column::new_with_children(vec![
            header_row.into(),
            Text::new("").style(text::Style::new().pixels(4.0)).into(),
            bar_text.into(),
        ])
        .item_alignment(Alignment::Center);

        let widget = Container::new(col)
            .width(Length::Fixed(self.width as f32))
            .height(Length::Fixed(self.height as f32))
            .padding(Padding {
                top: 14.0,
                right: 20.0,
                bottom: 14.0,
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

fn make_ascii_bar(pct: u8, total_cells: usize) -> String {
    let filled = ((pct as usize * total_cells) / 100).min(total_cells);
    let empty = total_cells.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

/// Spawns the transient OSD overlay and auto-closes after 1200ms
pub fn show_osd(kind: OsdKind) {
    info!(kind = ?kind, "Triggering Bihar Night Visual OSD");
    let modal = OsdModal {
        kind,
        ..Default::default()
    };

    if let Ok(handle) = new_widget(
        modal,
        None,
        KeyboardInteractivity::None,
        ExclusiveZone::Ignore,
        ZLayer::Overlay,
    ) {
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(1200)).await;
            handle.close();
        });
    }
}

/// Helper: Shows Audio Volume OSD
pub fn show_volume_osd(percentage: u8, muted: bool) {
    show_osd(OsdKind::Volume { percentage, muted });
}

/// Helper: Shows Brightness OSD
pub fn show_brightness_osd(percentage: u8) {
    show_osd(OsdKind::Brightness { percentage });
}

/// Helper: Shows DND OSD
pub fn show_dnd_osd(active: bool) {
    show_osd(OsdKind::Dnd { active });
}

/// Helper: Shows Night Light Gamma OSD
pub fn show_gamma_osd(kelvin: u32) {
    show_osd(OsdKind::Gamma { kelvin });
}

/// Helper: Shows Layout OSD
pub fn show_layout_osd(name: &str) {
    show_osd(OsdKind::Layout { name: name.to_string() });
}

/// Helper: Shows Layout Profile OSD
pub fn show_profile_osd(name: &str) {
    show_osd(OsdKind::Profile { name: name.to_string() });
}
