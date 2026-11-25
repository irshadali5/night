//! 🌙 Bihar Night Notification Center & DND Modal
//! Layer-shell overlay program displaying recent desktop notification history
//! with single-key Do Not Disturb toggling (`[D]`), history clearing (`[C]`), and dismissal.

use night_core::notifications::{NotificationHistory, NotificationRecord};
use pinnacle_api::experimental::snowcap_api::layer::{
    new_widget, ExclusiveZone, KeyboardInteractivity, ZLayer,
};
use pinnacle_api::experimental::snowcap_api::widget::{
    font::{Family, Font, Weight},
    text::{self, Text},
    WidgetDef,
};
use pinnacle_api::Keysym;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

#[derive(Clone, Debug)]
pub struct NotificationCenterModal {
    pub border_radius: f32,
    pub border_thickness: f32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub font: Font,
    pub width: u32,
    pub height: u32,
    pub items: Vec<NotificationRecord>,
    pub dnd_enabled: bool,
}

impl Default for NotificationCenterModal {
    fn default() -> Self {
        Self {
            border_radius: 14.0,
            border_thickness: 2.0,
            background_color: [0.10, 0.11, 0.15, 0.96], // Bihar Night Dark #1a1b26
            border_color: [0.48, 0.64, 0.97, 1.0],     // Bihar Night Blue #7aa2f7
            font: Font::new_with_family(Family::Name("Ubuntu".into())),
            width: 700,
            height: 420,
            items: Vec::new(),
            dnd_enabled: false,
        }
    }
}

impl pinnacle_api::experimental::snowcap_api::widget::Program for NotificationCenterModal {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) {}

    fn view(&self) -> Option<WidgetDef<Self::Message>> {
        let dnd_badge = if self.dnd_enabled {
            "󰂛 [DND ACTIVE]"
        } else {
            "󰂚 [DND OFF]"
        };

        let title = Text::new(format!("󰂚 Bihar Night Notification Center    {}", dnd_badge)).style(
            text::Style::new()
                .font(self.font.clone().weight(Weight::Bold))
                .pixels(17.0),
        );

        let subtitle = Text::new(
            "Press [D] Toggle Do Not Disturb  •  [C] Clear All  •  [Esc] Cancel",
        )
        .style(text::Style::new().font(self.font.clone()).pixels(12.0));

        let mut children: Vec<WidgetDef<Self::Message>> = vec![
            title.into(),
            Text::new("").style(text::Style::new().pixels(4.0)).into(),
            subtitle.into(),
            Text::new("").style(text::Style::new().pixels(12.0)).into(),
        ];

        if self.items.is_empty() {
            let empty = Text::new("No recent notifications recorded")
                .style(text::Style::new().font(self.font.clone()).pixels(13.0));
            children.push(empty.into());
        } else {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            for (idx, item) in self.items.iter().take(6).enumerate() {
                let num = idx + 1;
                let age_secs = now.saturating_sub(item.timestamp);
                let age_str = if age_secs < 60 {
                    format!("{age_secs}s ago")
                } else if age_secs < 3600 {
                    format!("{}m ago", age_secs / 60)
                } else {
                    format!("{}h ago", age_secs / 3600)
                };

                let app_tag = format!("[{}]", item.app_name);
                let summary_preview = if item.summary.chars().count() > 28 {
                    let s: String = item.summary.chars().take(25).collect();
                    format!("{s}...")
                } else {
                    item.summary.clone()
                };

                let body_preview = if item.body.chars().count() > 32 {
                    let s: String = item.body.replace('\n', " ").chars().take(29).collect();
                    format!("{s}...")
                } else {
                    item.body.replace('\n', " ")
                };

                let line = format!(
                    "  [{num}] {:<14} {:<28} • {:<28} {:>7}",
                    app_tag, summary_preview, body_preview, age_str
                );

                let row_text = Text::new(line).style(
                    text::Style::new()
                        .font(self.font.clone())
                        .pixels(12.5),
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

/// Displays the interactive Bihar Night Notification Center Modal via Snowcap
pub fn show_notifications_modal() {
    let history = NotificationHistory::load();

    let modal = NotificationCenterModal {
        items: history.items,
        dnd_enabled: history.dnd_enabled,
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
            match key {
                Keysym::d | Keysym::D => {
                    let new_dnd = NotificationHistory::toggle_dnd();
                    info!(dnd = new_dnd, "Toggled DND via Notification Modal");
                    h.close();
                }
                Keysym::c | Keysym::C => {
                    NotificationHistory::clear_all();
                    info!("Cleared notification history via Modal");
                    h.close();
                }
                _ => {
                    h.close();
                }
            }
        });
    });
}

