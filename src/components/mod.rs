use iced::{
    Alignment, Border, Color, Length,
    widget::{
        Container, container, svg,
        svg::{Handle, Svg},
    },
};

use crate::icons::{Icons, IconsFilled};

mod section;

pub fn section<'a, Message>(
    content: impl Into<iced::Element<'a, Message>>,
) -> Container<'a, Message> {
    container(content)
        .style(|_| container::Style {
            // TODO: use theme
            background: Some(iced::Background::Color(Color::parse("#2c2c3f").unwrap())),
            border: Border {
                radius: 6.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .padding([6, 12])
        .align_y(Alignment::Center)
        .height(Length::Fill)
}

pub fn icon(name: &str, color: Option<Color>) -> Svg {
    svg(Handle::from_memory(
        Icons::get(&format!("{name}.svg")).unwrap().data,
    ))
    .style(move |_, _| svg::Style { color })
    .width(Length::Shrink)
}

pub fn icon_filled(name: &str, color: Option<Color>) -> Svg {
    svg(Handle::from_memory(
        IconsFilled::get(&format!("{name}.svg")).unwrap().data,
    ))
    .style(move |_, _| svg::Style { color })
    .width(Length::Shrink)
}
