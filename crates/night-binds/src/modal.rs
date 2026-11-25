//! 🌙 Shared Snowcap Modal Frame & Styling Helpers
//! Eliminates duplicated header, spacing, and container border/background boilerplate
//! across Bihar Night layer-shell modals.

use pinnacle_api::experimental::snowcap_api::widget::{
    column::Column,
    container::{self, Container},
    font::{Font, Weight},
    text::{self, Text},
    Alignment, Background, Border, Length, Padding, WidgetDef,
};

/// Visual styling parameters for a Bihar Night Snowcap modal container.
pub struct ModalStyle {
    pub width: u32,
    pub height: u32,
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_thickness: f32,
    pub border_radius: f32,
}

/// Builds the standard Bihar Night modal header (`title`, 4px spacer, `subtitle`, 12px spacer).
pub fn modal_header<M: Clone + Send + 'static>(
    font: &Font,
    title: &str,
    subtitle: &str,
) -> Vec<WidgetDef<M>> {
    let title_widget = Text::new(title).style(
        text::Style::new()
            .font(font.clone().weight(Weight::Bold))
            .pixels(18.0),
    );
    let subtitle_widget =
        Text::new(subtitle).style(text::Style::new().font(font.clone()).pixels(12.0));

    vec![
        title_widget.into(),
        Text::new("").style(text::Style::new().pixels(4.0)).into(),
        subtitle_widget.into(),
        Text::new("").style(text::Style::new().pixels(12.0)).into(),
    ]
}

/// Wraps modal `children` in the standard Bihar Night `Container` with uniform padding and border.
pub fn modal_container<M: Clone + Send + 'static>(
    children: Vec<WidgetDef<M>>,
    style: ModalStyle,
) -> WidgetDef<M> {
    Container::new(Column::new_with_children(children))
        .width(Length::Fixed(style.width as f32))
        .height(Length::Fixed(style.height as f32))
        .padding(Padding {
            top: 20.0,
            right: 24.0,
            bottom: 20.0,
            left: 24.0,
        })
        .vertical_alignment(Alignment::Start)
        .horizontal_alignment(Alignment::Start)
        .style(container::Style {
            text_color: None,
            background: Some(Background::Color(style.background_color.into())),
            border: Some(Border {
                color: Some(style.border_color.into()),
                width: Some(style.border_thickness),
                radius: Some(style.border_radius.into()),
            }),
        })
        .into()
}
