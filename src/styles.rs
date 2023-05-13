use iced::{
    alignment::{self, Horizontal, Vertical},
    theme::{self, Theme},
    widget::{self, button, text, Text},
    Color, Font, Renderer, Vector,
};

use crate::Message;

pub struct BackButtonStyle;
impl iced::widget::button::StyleSheet for BackButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(style.palette().danger)),
            border_radius: 10.0,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
            text_color: style.palette().text,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let Color { r, g, b, a } = style.palette().danger;
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(Color::new(
                r * 0.9,
                g * 0.9,
                b * 0.9,
                a,
            ))),
            border_radius: 10.0,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
            text_color: style.palette().text,
        }
    }
}

pub struct HeaderButtonStyle;
impl iced::widget::button::StyleSheet for HeaderButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let Color { r, g, b, a } = style.palette().primary;
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(Color::new(
                r * 1.2,
                g * 1.2,
                b * 1.2,
                a,
            ))),
            border_radius: 10.0,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
            text_color: style.palette().text,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(style.palette().primary)),
            border_radius: 10.0,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
            text_color: style.palette().text,
        }
    }
}

pub fn header_button<'state>(
    content: &str,
    style: impl iced::widget::button::StyleSheet<Style = Theme> + 'static,
) -> widget::Button<'state, Message, Renderer<Theme>> {
    button(text(content).size(20)).style(theme::Button::Custom(Box::new(style)))
}
pub fn switch_button() -> widget::Button<'static, Message, Renderer<Theme>> {
    button(
        text("⟷")
            .size(30)
            .width(20)
            .height(20)
            .font(JBM)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center),
    )
}

pub const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../fonts/icons.ttf"),
};

pub const JBM: Font = Font::External {
    name: "JetBrainsMono-Regular",
    bytes: include_bytes!("../fonts/fonts/ttf/JetBrainsMono-Regular.ttf"),
};

pub fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
        .size(20)
}

pub fn edit_icon() -> Text<'static> {
    icon('\u{F303}')
}

pub fn delete_icon() -> Text<'static> {
    icon('\u{F1F8}')
}

pub fn delete_button<'state>() -> widget::Button<'state, Message, Renderer<Theme>> {
    button(delete_icon()).style(theme::Button::Custom(Box::new(DeleteButtonStyle)))
}

struct DeleteButtonStyle;
impl iced::widget::button::StyleSheet for DeleteButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(style.palette().danger)),
            border_radius: 1.0,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
            text_color: style.palette().text,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let Color { r, g, b, a } = style.palette().danger;
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(Color {
                r: r * 0.8,
                g: g * 0.8,
                b: b * 0.8,
                a,
            })),
            border_radius: 1.0,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
            text_color: style.palette().text,
        }
    }
}
