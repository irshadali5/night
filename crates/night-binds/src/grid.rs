//! 🌙 Bihar Night Workspace Grid & Exposé Matrix
//! Layer-shell overlay program displaying a 3x3 visual overview of all 9 workspace tags.
//! Shows tag status, active window badges, window counts, and provides 1-key jump / move operations.

use night_tags::DEFAULT_TAG_NAMES;
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
use pinnacle_api::{output, window, Keysym};
use tracing::info;

#[derive(Clone, Debug)]
pub struct TagGridItem {
    pub name: String,
    pub glyph: &'static str,
    pub label: &'static str,
    pub is_active: bool,
    pub window_count: usize,
    pub top_window_titles: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct WorkspaceGridModal {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: Font,
    pub width: u32,
    pub height: u32,
    pub tags: Vec<TagGridItem>,
}

impl Default for WorkspaceGridModal {
    fn default() -> Self {
        Self {
            border_radius: 14.0,
            border_thickness: 2.0,
            background_color: [0.10, 0.11, 0.15, 0.96],
            border_color: [0.48, 0.64, 0.97, 1.0], // Bihar Night Blue #7aa2f7
            font: Font::new_with_family(Family::Name("Ubuntu".into())),
            width: 780,
            height: 480,
            tags: Vec::new(),
        }
    }
}

impl pinnacle_api::experimental::snowcap_api::widget::Program for WorkspaceGridModal {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(&self) -> Option<WidgetDef<Self::Message>> {
        let title = Text::new("🌙 Bihar Night Workspace Matrix (Exposé)").style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(18.0),
        );

        let subtitle = Text::new(
            "Press [1-9] to switch workspace tag  •  [Shift+1-9] Move Window  •  [Esc/Q] Dismiss",
        )
        .style(text::Style::new().font(self.font.clone()).pixels(12.0));

        let mut children: Vec<WidgetDef<Self::Message>> = Vec::new();
        children.push(title.into());
        children.push(Text::new("").style(text::Style::new().pixels(4.0)).into());
        children.push(subtitle.into());
        children.push(Text::new("").style(text::Style::new().pixels(14.0)).into());

        // Render 3 rows of 3 tags each
        for row_idx in 0..3 {
            let start = row_idx * 3;
            let end = (start + 3).min(self.tags.len());
            let row_slice = &self.tags[start..end];

            let mut row_lines: Vec<String> = Vec::new();
            for item in row_slice {
                let status_badge = if item.is_active {
                    "★ ACTIVE"
                } else if item.window_count > 0 {
                    "● OCCUPIED"
                } else {
                    "○ EMPTY"
                };

                let count_str = if item.window_count > 0 {
                    format!("({} win{})", item.window_count, if item.window_count > 1 { "s" } else { "" })
                } else {
                    String::new()
                };

                let col_str = format!(
                    "[{}] {} {:<5} {:<10} {:<8}",
                    item.name, item.glyph, item.label, status_badge, count_str
                );
                row_lines.push(col_str);
            }

            let combined_row = row_lines.join("   │   ");
            let row_text = Text::new(combined_row).style(
                text::Style::new()
                    .font(self.font.clone().weight(Weight::Bold))
                    .pixels(13.0),
            );
            children.push(row_text.into());

            // Window previews underneath each cell
            let mut preview_lines: Vec<String> = Vec::new();
            for item in row_slice {
                let preview = if let Some(first_win) = item.top_window_titles.first() {
                    let short = if first_win.chars().count() > 20 {
                        let s: String = first_win.chars().take(18).collect();
                        format!("{s}..")
                    } else {
                        first_win.clone()
                    };
                    format!("   └─ {short}")
                } else {
                    "   └─ (no windows)".to_string()
                };
                preview_lines.push(format!("{:<28}", preview));
            }
            let preview_row = preview_lines.join("   │ ");
            let preview_text = Text::new(preview_row).style(
                text::Style::new()
                    .font(self.font.clone())
                    .pixels(11.0),
            );
            children.push(preview_text.into());
            children.push(Text::new("").style(text::Style::new().pixels(12.0)).into());
        }

        let widget = Container::new(Column::new_with_children(children))
            .width(Length::Fixed(self.width as f32))
            .height(Length::Fixed(self.height as f32))
            .padding(Padding {
                top: 22.0,
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

/// Displays the interactive Bihar Night Workspace Grid Exposé Modal via Snowcap
pub fn show_workspace_grid() {
    let focused_op = output::get_focused();
    let active_tag_name = focused_op
        .as_ref()
        .and_then(|op| op.active_tags().next())
        .map(|t| t.name())
        .unwrap_or_else(|| "1".into());

    let all_windows = window::get_all().collect::<Vec<_>>();

    let mut tags = Vec::new();
    for name in DEFAULT_TAG_NAMES {
        let glyph = night_tags::tag_glyph(name);
        let label = night_tags::tag_label(name);
        let is_active = *name == active_tag_name;

        let mut win_titles = Vec::new();
        let mut count = 0;

        for win in &all_windows {
            if win.tags().any(|t| t.name() == *name) {
                count += 1;
                let title = win.title();
                if !title.is_empty() {
                    win_titles.push(title);
                } else {
                    win_titles.push(win.app_id());
                }
            }
        }

        tags.push(TagGridItem {
            name: (*name).to_string(),
            glyph,
            label,
            is_active,
            window_count: count,
            top_window_titles: win_titles,
        });
    }

    let modal = WorkspaceGridModal {
        tags,
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
        handle.on_key_press(move |h, key, mods| {
            let target_digit = match key {
                Keysym::_1 => Some("1"),
                Keysym::_2 => Some("2"),
                Keysym::_3 => Some("3"),
                Keysym::_4 => Some("4"),
                Keysym::_5 => Some("5"),
                Keysym::_6 => Some("6"),
                Keysym::_7 => Some("7"),
                Keysym::_8 => Some("8"),
                Keysym::_9 => Some("9"),
                _ => None,
            };

            if let Some(digit) = target_digit {
                if mods.shift {
                    info!(tag = digit, "Workspace Grid: Moving focused window to tag");
                    night_tags::move_focused_to_tag(digit);
                } else {
                    info!(tag = digit, "Workspace Grid: Switching to tag");
                    night_tags::switch_to_tag(digit);
                }
                h.close();
                return;
            }

            match key {
                Keysym::Escape | Keysym::q | Keysym::Q => {
                    h.close();
                }
                _ => (),
            }
        });
    });
}
