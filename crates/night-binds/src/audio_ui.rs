//! 🌙 Bihar Night Audio Output & Sink Switcher Modal
//! Layer-shell overlay program displaying available audio output devices (Headphones, Speakers, HDMI, USB DAC)
//! with instant single-key selection (`[1-5]`), quick cycle (`[C]`), and notification feedback.

use night_core::audio::{cycle_sink, list_sinks, set_default_sink, AudioSink};
use pinnacle_api::experimental::snowcap_api::layer::{
    new_widget, ExclusiveZone, KeyboardInteractivity, ZLayer,
};
use pinnacle_api::experimental::snowcap_api::widget::{
    font::{Family, Font, Weight},
    text::{self, Text},
    WidgetDef,
};
use pinnacle_api::Keysym;
use tracing::info;

#[derive(Clone, Debug)]
pub struct AudioSinkModal {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: Font,
    pub width: u32,
    pub height: u32,
    pub sinks: Vec<AudioSink>,
}

impl Default for AudioSinkModal {
    fn default() -> Self {
        Self {
            border_radius: 14.0,
            border_thickness: 2.0,
            background_color: [0.10, 0.11, 0.15, 0.96], // Bihar Night Dark #1a1b26
            border_color: [0.48, 0.64, 0.97, 1.0],     // Bihar Night Blue #7aa2f7
            font: Font::new_with_family(Family::Name("Ubuntu".into())),
            width: 660,
            height: 330,
            sinks: Vec::new(),
        }
    }
}

impl pinnacle_api::experimental::snowcap_api::widget::Program for AudioSinkModal {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(&self) -> Option<WidgetDef<Self::Message>> {
        let title = Text::new("🔊 Bihar Night Audio Output Selector").style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(18.0),
        );

        let subtitle = Text::new(
            "Press [1-5] to select output  •  [C] Cycle Next  •  [Esc] Cancel",
        )
        .style(text::Style::new().font(self.font.clone()).pixels(12.0));

        let mut children: Vec<WidgetDef<Self::Message>> = vec![
            title.into(),
            Text::new("").style(text::Style::new().pixels(4.0)).into(),
            subtitle.into(),
            Text::new("").style(text::Style::new().pixels(12.0)).into(),
        ];

        if self.sinks.is_empty() {
            let empty = Text::new("No active audio output sinks detected via PipeWire / PulseAudio")
                .style(text::Style::new().font(self.font.clone()).pixels(13.0));
            children.push(empty.into());
        } else {
            for (idx, sink) in self.sinks.iter().take(5).enumerate() {
                let num = idx + 1;
                let active_marker = if sink.is_default { "▶ " } else { "  " };
                let default_badge = if sink.is_default { "  [ACTIVE]" } else { "" };
                let icon = sink.icon();

                let line = format!(
                    "{active_marker}[{num}] {}  {:<38}{}",
                    icon, sink.name, default_badge
                );

                let weight = if sink.is_default {
                    Weight::Bold
                } else {
                    Weight::Normal
                };

                let row_text = Text::new(line).style(
                    text::Style::new()
                        .font(self.font.clone().weight(weight))
                        .pixels(13.0),
                );

                children.push(row_text.into());
                children.push(Text::new("").style(text::Style::new().pixels(6.0)).into());
            }
        }

        Some(crate::modal::modal_container(
            children,
            crate::modal::ModalStyle {
                width: self.width,
                height: self.height,
                background_color: self.background_color,
                border_color: self.border_color,
                border_thickness: self.border_thickness,
                border_radius: self.border_radius,
            },
        ))
    }
}

/// Displays the interactive Bihar Night Audio Output Selector Modal via Snowcap
pub fn show_audio_sink_modal() {
    let sinks = list_sinks();
    let sinks_clone = sinks.clone();

    let modal = AudioSinkModal {
        sinks,
        ..Default::default()
    };

    let _ = new_widget(
        modal,
        None,
        KeyboardInteractivity::Exclusive,
        ExclusiveZone::Respect,
        ZLayer::Overlay,
    )
    .map(|handle| {
        handle.on_key_press(move |h, key, _mods| {
            let select_idx = match key {
                Keysym::_1 => Some(0),
                Keysym::_2 => Some(1),
                Keysym::_3 => Some(2),
                Keysym::_4 => Some(3),
                Keysym::_5 => Some(4),
                Keysym::c | Keysym::C => {
                    if let Some(new_sink) = cycle_sink() {
                        info!(name = %new_sink.name, id = new_sink.id, "Audio sink cycled via modal");
                    }
                    h.close();
                    return;
                }
                _ => {
                    h.close();
                    return;
                }
            };

            if let Some(idx) = select_idx {
                if let Some(sink) = sinks_clone.get(idx) {
                    set_default_sink(sink.id);
                    info!(name = %sink.name, id = sink.id, "Audio sink switched via modal");
                }
            }
            h.close();
        });
    });
}

