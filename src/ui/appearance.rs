use iced::{
    alignment,
    widget::{container, text, text_input, Text},
    Background, Font, Theme,
};
// use lazy_static::lazy_static;

const ICONS: Font = Font::External {
    name: "Nerc Icons",
    bytes: include_bytes!("../../fonts/nerd_font.ttf"),
};

pub const NOTO_SANS: &[u8; 556216] = include_bytes!("../../fonts/noto_sans.ttf");

pub fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
        .size(20)
}

pub struct AppCardStyle {}

impl container::StyleSheet for AppCardStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            border_radius: 20.0,
            border_width: 2.0,
            border_color: StyleSheet::from_theme(style).border_color,
            background: StyleSheet::from_theme(style).app_card_background,
            ..container::Appearance::default()
        }
    }
}

pub struct SectionsStyle {}

impl container::StyleSheet for SectionsStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            border_radius: 20.0,
            border_width: 2.0,
            border_color: StyleSheet::from_theme(style).border_color,
            ..container::Appearance::default()
        }
    }
}

pub struct SearchBoxStyle {}

impl text_input::StyleSheet for SearchBoxStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: StyleSheet::from_theme(style).app_card_background.unwrap(),
            border_radius: 20.0,
            border_width: 1.0,
            border_color: StyleSheet::from_theme(style).accent,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }

    fn placeholder_color(&self, style: &Self::Style) -> iced::Color {
        iced::Color::from_rgb(0.4, 0.4, 0.4)
    }

    fn value_color(&self, style: &Self::Style) -> iced::Color {
        iced::Color::from_rgb(0.99, 0.99, 0.99)
    }

    fn selection_color(&self, style: &Self::Style) -> iced::Color {
        iced::Color::from_rgb(0.1, 0.6, 0.6)
    }
}

pub struct StyleSheet {
    pub app_name_size: f32,
    pub app_card_text_color: iced::Color,
    pub app_desc_size: f32,
    pub app_card_background: Option<Background>,
    pub border_color: iced::Color,
    pub accent: iced::Color,
}

impl StyleSheet {
    pub fn from_theme(theme: &iced::Theme) -> StyleSheet {
        match theme {
            Theme::Dark => StyleSheet {
                app_name_size: 40.0,
                app_card_text_color: iced::Color::from_rgb(0.9, 0.9, 0.9),
                app_desc_size: 20.0,
                app_card_background: Some(Background::Color(iced::Color::from_rgb(0.2, 0.2, 0.2))),
                border_color: iced::Color::from_rgb(0.4, 0.4, 0.4),
                accent: iced::Color::from_rgb(1.0, 0.72, 0.29),
            },
            _ => StyleSheet {
                app_name_size: 40.0,
                app_card_text_color: iced::Color::from_rgb(0.1, 0.1, 0.1),
                app_desc_size: 20.0,
                app_card_background: Some(Background::Color(iced::Color::from_rgb(0.9, 0.9, 0.9))),
                border_color: iced::Color::from_rgb(0.8, 0.8, 0.8),
                accent: iced::Color::from_rgb(1.0, 0.72, 0.29),
            },
        }
    }
}
