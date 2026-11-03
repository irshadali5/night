//! 🌙 Tokyo Night Clipboard History Modal
//! Layer-shell overlay program displaying recently copied snippets with single-key selection.

use night_core::clipboard::{copy_to_system_clipboard, ClipboardHistory, ClipboardItem};
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
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[derive(Clone, Debug)]
pub struct ClipboardModal {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: Font,
    pub width: u32,
    pub height: u32,
    pub items: Vec<ClipboardItem>,
}

impl Default for ClipboardModal {
    fn default() -> Self {
        Self {
            border_radius: 14.0,
            border_thickness: 2.0,
            background_color: [0.10, 0.11, 0.15, 0.96],
            border_color: [0.48, 0.64, 0.97, 1.0], // Tokyo Night Blue #7aa2f7
            font: Font::new_with_family(Family::Name("Ubuntu".into())),
            width: 740,
            height: 380,
            items: Vec::new(),
        }
    }
}

impl pinnacle_api::experimental::snowcap_api::widget::Program for ClipboardModal {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(&self) -> Option<WidgetDef<Self::Message>> {
        let title = Text::new("📋 Tokyo Night Clipboard History").style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(18.0),
        );

        let subtitle = Text::new(
            "Press [1-9] to copy & select  •  [C] Clear History  •  [Esc] Cancel",
        )
        .style(text::Style::new().font(self.font.clone()).pixels(12.0));

        let mut children: Vec<WidgetDef<Self::Message>> = Vec::new();
        children.push(title.into());
        children.push(Text::new("").style(text::Style::new().pixels(4.0)).into());
        children.push(subtitle.into());
        children.push(Text::new("").style(text::Style::new().pixels(12.0)).into());

        if self.items.is_empty() {
            let empty_text = Text::new("Clipboard history is empty")
                .style(text::Style::new().font(self.font.clone()).pixels(14.0));
            children.push(empty_text.into());
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            for (idx, item) in self.items.iter().take(9).enumerate() {
                let num = idx + 1;
                let age_secs = now.saturating_sub(item.timestamp);
                let age_str = if age_secs < 60 {
                    format!("{age_secs}s ago")
                } else if age_secs < 3600 {
                    format!("{}m ago", age_secs / 60)
                } else {
                    format!("{}h ago", age_secs / 3600)
                };

                let clean = item.content.replace('\n', " ").replace('\t', " ");
                let preview = if clean.chars().count() > 45 {
                    let s: String = clean.chars().take(42).collect();
                    format!("{s}...")
                } else {
                    clean
                };

                let line = format!(
                    "  [{num}] ({:>3} chars)  {:<46}  • {:>7}",
                    item.content.len(),
                    preview,
                    age_str
                );

                let row_text = Text::new(line).style(
                    text::Style::new()
                        .font(self.font.clone())
                        .pixels(13.0),
                );

                children.push(row_text.into());
                children.push(Text::new("").style(text::Style::new().pixels(4.0)).into());
            }
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

/// Displays the interactive Tokyo Night Clipboard History Modal via Snowcap
pub fn show_clipboard_modal() {
    let history = ClipboardHistory::load();
    let items_clone = history.items.clone();

    let modal = ClipboardModal {
        items: history.items,
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
                Keysym::_6 => Some(5),
                Keysym::_7 => Some(6),
                Keysym::_8 => Some(7),
                Keysym::_9 => Some(8),
                Keysym::c | Keysym::C => {
                    ClipboardHistory::clear_all();
                    info!("Clipboard history cleared via modal");
                    h.close();
                    return;
                }
                _ => {
                    h.close();
                    return;
                }
            };

            if let Some(idx) = select_idx {
                if let Some(item) = items_clone.get(idx) {
                    copy_to_system_clipboard(&item.content);
                    info!(idx, "Restored clipboard item from modal selection");
                }
            }
            h.close();
        });
    });
}

