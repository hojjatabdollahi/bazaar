use iced::{
    alignment, application,
    widget::{button, container, horizontal_rule, scrollable, text, text_input, Text},
    Background, Color, Font, Vector,
};
use iced_style::scrollable::Scroller;

const ICONS: Font = Font::External {
    name: "Nerc Icons",
    bytes: include_bytes!("../../fonts/nerd_font.ttf"),
};

pub const NOTO_SANS: &[u8; 556216] = include_bytes!("../../fonts/noto_sans.ttf");

pub fn icon(unicode: char) -> Text<'static, iced::Renderer<Theme>> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
        .size(20)
}

#[derive(Default)]
pub enum Theme {
    Light,
    #[default]
    Dark,
}

pub struct Colors {
    pub background: Color,
    pub text: Color,
    pub accent: Color,
    pub border: Color,
    pub border_low_contrast: Color,
}

impl Colors {
    pub const DARK: Self = Self {
        background: Color::from_rgb(0.2, 0.2, 0.2),
        text: Color::from_rgb(0.9, 0.9, 0.9),
        accent: iced::Color::from_rgb(1.0, 0.72, 0.29),
        border: iced::Color::from_rgb(0.3, 0.3, 0.3),
        border_low_contrast: iced::Color::from_rgb(0.1, 0.1, 0.1),
    };
    pub const LIGHT: Self = Self {
        background: Color::from_rgb(0.9, 0.9, 0.9),
        text: Color::from_rgb(0.2, 0.2, 0.2),
        accent: iced::Color::from_rgb(1.0, 0.72, 0.29),
        border: iced::Color::from_rgb(0.6, 0.6, 0.6),
        border_low_contrast: iced::Color::from_rgb(0.8, 0.8, 0.8),
    };
}

impl Theme {
    pub fn colors(&self) -> Colors {
        match self {
            Self::Light => Colors::LIGHT,
            Self::Dark => Colors::DARK,
        }
    }
}

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: self.colors().background,
            text_color: self.colors().text,
        }
    }
}

#[derive(Default)]
pub enum ContainerStyle {
    #[default]
    Default,
    AppCard,
    Section,
}

impl container::StyleSheet for Theme {
    type Style = ContainerStyle;
    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let border_color = match style {
            ContainerStyle::Default => self.colors().background,
            ContainerStyle::AppCard => self.colors().border,
            ContainerStyle::Section => self.colors().border,
        };
        container::Appearance {
            border_radius: 20.0,
            border_width: 2.0,
            border_color,
            background: Some(Background::Color(self.colors().background)),
            text_color: None,
        }
    }
}

impl text::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        text::Appearance {
            color: Some(self.colors().text),
        }
    }
}

impl iced::widget::rule::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, style: &Self::Style) -> iced_style::rule::Appearance {
        iced_style::rule::Appearance {
            color: self.colors().border,
            width: 1,
            radius: 1.0,
            fill_mode: iced_style::rule::FillMode::Full,
        }
    }
}

#[derive(Default)]
pub enum ButtonStyle {
    #[default]
    Default,
    Icon,
}

impl button::StyleSheet for Theme {
    type Style = ButtonStyle;

    fn active(&self, style: &Self::Style) -> iced_style::button::Appearance {
        let border_color = match style {
            ButtonStyle::Default => self.colors().border,
            ButtonStyle::Icon => Color::TRANSPARENT,
        };
        button::Appearance {
            shadow_offset: Vector::default(),
            background: None,
            border_radius: 1.0,
            border_width: 1.0,
            border_color,
            text_color: self.colors().accent,
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced_style::button::Appearance {
        let active = self.active(style);

        iced_style::button::Appearance {
            background: None,
            ..active
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        self.active(style)
    }
}

impl scrollable::StyleSheet for Theme {
    type Style = ();

    fn active(&self, style: &Self::Style) -> iced_style::scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: Some(Background::Color(self.colors().background)),
            border_radius: 1.0,
            border_width: 1.0,
            border_color: self.colors().border,
            scroller: Scroller {
                color: self.colors().accent,
                border_radius: 1.0,
                border_width: 1.0,
                border_color: self.colors().border,
            },
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced_style::scrollable::Scrollbar {
        self.active(style)
    }
}

impl text_input::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(self.colors().background),
            border_radius: 20.0,
            border_width: 1.0,
            border_color: self.colors().border,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            border_color: self.colors().accent,
            ..self.active(style)
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> iced::Color {
        iced::Color::from_rgb(0.4, 0.4, 0.4)
    }

    fn value_color(&self, _style: &Self::Style) -> iced::Color {
        iced::Color::from_rgb(0.99, 0.99, 0.99)
    }

    fn selection_color(&self, _style: &Self::Style) -> iced::Color {
        iced::Color::from_rgb(0.1, 0.6, 0.6)
    }
}

impl iced_aw::tabs::StyleSheet for Theme {
    type Style = ();

    fn active(&self, _style: Self::Style, _is_active: bool) -> iced_aw::style::tab_bar::Appearance {
        iced_aw::style::tab_bar::Appearance {
            background: Some(Background::Color(self.colors().background)),
            border_color: Some(self.colors().border),
            border_width: 1.0,
            tab_label_background: Background::Color(self.colors().background),
            tab_label_border_color: self.colors().border,
            tab_label_border_width: 1.0,
            icon_color: self.colors().accent,
            text_color: self.colors().accent,
        }
    }

    fn hovered(
        &self,
        _style: Self::Style,
        _is_active: bool,
    ) -> iced_aw::style::tab_bar::Appearance {
        iced_aw::style::tab_bar::Appearance::default()
    }
}
