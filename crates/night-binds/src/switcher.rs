//! 🌙 Tokyo Night Visual Window Switcher
//! Layer-shell overlay program displaying all active windows across tags and outputs.
//! Provides keyboard-driven navigation (`[1..9]` direct jump, `[Enter]` focus, `[D]` close).

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
pub struct SwitcherItem {
    pub id: u32,
    pub title: String,
    pub app_id: String,
    pub tag: String,
    pub output: String,
    pub is_focused: bool,
}

#[derive(Clone, Debug)]
pub struct WindowSwitcherModal {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: Font,
    pub width: u32,
    pub height: u32,
    pub items: Vec<SwitcherItem>,
}

impl Default for WindowSwitcherModal {
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

impl pinnacle_api::experimental::snowcap_api::widget::Program for WindowSwitcherModal {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(&self) -> Option<WidgetDef<Self::Message>> {
        let title = Text::new("🌙 Tokyo Night Window Switcher").style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(18.0),
        );

        let subtitle = Text::new(
            "Press [1-9] to jump directly  •  [Enter] Focus  •  [D] Close  •  [Esc] Cancel",
        )
        .style(text::Style::new().font(self.font.clone()).pixels(12.0));

        let mut children: Vec<WidgetDef<Self::Message>> = Vec::new();
        children.push(title.into());
        children.push(Text::new("").style(text::Style::new().pixels(4.0)).into());
        children.push(subtitle.into());
        children.push(Text::new("").style(text::Style::new().pixels(12.0)).into());

        if self.items.is_empty() {
            let empty_text = Text::new("No active windows open on any workspace tag")
                .style(text::Style::new().font(self.font.clone()).pixels(14.0));
            children.push(empty_text.into());
        } else {
            for (idx, item) in self.items.iter().take(9).enumerate() {
                let num = idx + 1;
                let focus_indicator = if item.is_focused { "▶ " } else { "  " };
                let app_badge = format!("[{}]", item.app_id);

                let line = format!(
                    "{focus_indicator}[{num}] {:<16} {:<34} (Tag {} • {})",
                    app_badge, item.title, item.tag, item.output
                );

                let weight = if item.is_focused {
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

/// Displays the interactive Tokyo Night Window Switcher Modal via Snowcap
pub fn show_window_switcher() {
    let mut items = Vec::new();
    let focused = pinnacle_api::window::get_focused();

    for win in pinnacle_api::window::get_all() {
        let title = win.title();
        let app_id = win.app_id();
        let is_focused = focused.as_ref().map(|f| f.id() == win.id()).unwrap_or(false);
        let tag_name = win.tags().next().map(|t| t.name()).unwrap_or_else(|| "1".into());
        let output_name = win.output().map(|o| o.name()).unwrap_or_else(|| "default".into());

        let short_title = if title.chars().count() > 36 {
            let s: String = title.chars().take(33).collect();
            format!("{s}...")
        } else if title.is_empty() {
            app_id.clone()
        } else {
            title
        };

        items.push(SwitcherItem {
            id: win.id(),
            title: short_title,
            app_id: if app_id.is_empty() { "App".into() } else { app_id },
            tag: tag_name,
            output: output_name,
            is_focused,
        });
    }

    let items_clone = items.clone();
    let modal = WindowSwitcherModal {
        items,
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
                Keysym::Return => Some(0),
                Keysym::d | Keysym::D => {
                    if let Some(first) = items_clone.first() {
                        for win in pinnacle_api::window::get_all() {
                            if win.id() == first.id {
                                win.close();
                                break;
                            }
                        }
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
                if let Some(target) = items_clone.get(idx) {
                    for win in pinnacle_api::window::get_all() {
                        if win.id() == target.id {
                            if let Some(op) = win.output() {
                                op.focus();
                            }
                            if let Some(tag) = win.tags().next() {
                                tag.set_active(true);
                            }
                            win.raise();
                            win.set_focused(true);
                            info!(id = win.id(), title = %target.title, "Switched window focus");
                            break;
                        }
                    }
                }
            }
            h.close();
        });
    });
}
