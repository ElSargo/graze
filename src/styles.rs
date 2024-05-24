use std::sync::Arc;

use lazy_static::lazy_static;
lazy_static! {
    pub static ref THEME: Arc<Custom> = Arc::new(Custom::new("Adw".to_string(), PALETTE,));
}

pub const PALETTE: Palette = Palette {
    background: Color::from_rgb(32.0 / 256.0, 32.0 / 256.0, 32.0 / 256.0),
    text: Color::from_rgb(242.0 / 256.0, 242.0 / 256.0, 242.0 / 256.0),
    primary: Color::from_rgb(67.0 / 256.0, 141.0 / 256.0, 230.0 / 256.0),
    success: Color::from_rgb(51.0 / 256.0, 209.0 / 256.0, 122.0 / 256.0),
    danger: Color::from_rgb(237.0 / 256.0, 51.0 / 256.0, 59.0 / 256.0),
};

use iced::{
    alignment::{self, Horizontal, Vertical},
    theme::{self, Custom, Palette, Theme},
    widget::{self, button, text, Text},
    Border, Color, Font, Shadow, Vector,
};

use crate::Message;

pub struct BackButtonStyle;
impl iced::widget::button::StyleSheet for BackButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(style.palette().danger)),
            text_color: style.palette().text,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
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
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
            text_color: style.palette().text,
        }
    }
}

impl Into<iced::theme::TextInput> for InvaldTextStyle {
    fn into(self) -> iced::theme::TextInput {
        iced::theme::TextInput::Custom(Box::new(InvaldTextStyle))
    }
}

pub struct InvaldTextStyle;
impl iced::widget::text_input::StyleSheet for InvaldTextStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> widget::text_input::Appearance {
        widget::text_input::Appearance {
            background: iced::Background::Color(PALETTE.background),
            border: Border {
                color: PALETTE.danger,
                width: 2.0,
                radius: 4.0.into(),
            },
            icon_color: PALETTE.primary,
        }
    }

    fn focused(&self, style: &Self::Style) -> widget::text_input::Appearance {
        widget::text_input::Appearance {
            background: iced::Background::Color(PALETTE.background),
            border: Border {
                color: PALETTE.danger,
                width: 2.0,
                radius: 4.0.into(),
            },
            icon_color: PALETTE.primary,
        }
    }

    fn placeholder_color(&self, style: &Self::Style) -> Color {
        PALETTE.danger
    }

    fn value_color(&self, style: &Self::Style) -> Color {
        PALETTE.danger
    }

    fn disabled_color(&self, style: &Self::Style) -> Color {
        PALETTE.danger
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        PALETTE.danger
    }

    fn disabled(&self, style: &Self::Style) -> widget::text_input::Appearance {
        widget::text_input::Appearance {
            background: iced::Background::Color(PALETTE.background),
            border: Border {
                color: PALETTE.danger,
                width: 2.0,
                radius: 4.0.into(),
            },
            icon_color: PALETTE.primary,
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
                (r * 1.2).clamp(0., 1.),
                (g * 1.2).clamp(0., 1.),
                (b * 1.2).clamp(0., 1.),
                a,
            ))),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
            text_color: style.palette().text,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(style.palette().primary)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
            text_color: style.palette().text,
        }
    }
}

pub fn header_button<'state>(
    content: &str,
    style: impl iced::widget::button::StyleSheet<Style = Theme> + 'static,
) -> widget::Button<'state, Message> {
    button(text(content).size(20)).style(theme::Button::Custom(Box::new(style)))
}
pub fn switch_button() -> widget::Button<'static, Message> {
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

// pub const ICONS: Font = Font::External {
//     name: "Icons",
//     bytes: include_bytes!("../fonts/icons.ttf"),
// };
pub const ICONS: Font = Font {
    family: iced::font::Family::Name("fontello"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

// pub const JBM: Font = Font::External {
//     name: "JetBrainsMono-Regular",
//     bytes: include_bytes!(std::env!("MAIN_FONT_PATH")),
// };

pub const JBM: Font = Font {
    family: iced::font::Family::Name("JetBrainsMono"),
    weight: iced::font::Weight::Normal,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
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
    icon('\u{F1F8}').size(20)
}

pub fn delete_button<'state>() -> widget::Button<'state, Message> {
    button(delete_icon()).style(theme::Button::Custom(Box::new(DeleteButtonStyle)))
}

struct DeleteButtonStyle;
impl iced::widget::button::StyleSheet for DeleteButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(style.palette().danger)),
            border: Border {
                color: Color::TRANSPARENT,
                width: 2.0,
                radius: 10.0.into(),
            },
            shadow: Shadow::default(),
            text_color: style.palette().text,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let Color { r, g, b, a } = style.palette().danger;
        button::Appearance {
            shadow_offset: Vector::ZERO,
            background: Some(iced::Background::Color(Color {
                r: (r * 0.8).clamp(0.0, 1.0),
                g: (g * 0.8).clamp(0.0, 1.0),
                b: (b * 0.8).clamp(0.0, 1.0),
                a,
            })),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 20.0.into(),
            },
            shadow: Shadow::default(),
            text_color: style.palette().text,
        }
    }
}
