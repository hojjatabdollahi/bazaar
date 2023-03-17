use std::path::PathBuf;

use iced::{
    executor, theme,
    widget::{
        button, column, container, horizontal_rule, image, row, scrollable, svg, text, Container,
    },
    Application, Length, Settings, Theme,
};
use iced_aw::wrap;

use crate::backend::flatpak_backend::{self, Package};

use super::appearance::{self, StyleSheet};

pub fn run() -> iced::Result {
    BazaarApp::run(Settings {
        default_font: Some(appearance::NOTO_SANS),
        ..Settings::default()
    })
}

struct Config {
    dark_mode: bool,
}

struct BazaarApp {
    // pub view: String,
    pub config: Config,
}

#[derive(Debug, Clone)]
enum Message {
    // Refresh,
}

impl BazaarApp {
    fn app_icon<'a>(&self, width: u16, path: &PathBuf) -> Container<'a, Message> {
        container(
            image(path)
                .content_fit(iced::ContentFit::Fill)
                .height(width)
                .width(width),
        )
        .padding(10)
        .center_x()
    }
    fn style_sheet(&self) -> StyleSheet {
        appearance::StyleSheet::from_theme(&self.theme())
    }
    fn app_card(&self, package: &Package) -> iced::Element<Message> {
        container(row(vec![
            self.app_icon(64, &package.icon_path).into(),
            column(vec![
                text(&package.name)
                    .width(Length::Fixed(250.))
                    .style(theme::Text::Color(self.style_sheet().app_card_text_color))
                    .size(28)
                    .into(),
                text(&package.description)
                    .width(Length::Fixed(250.))
                    .style(theme::Text::Color(self.style_sheet().app_card_text_color))
                    .size(18)
                    .into(),
            ])
            .width(Length::Shrink)
            .into(),
            button(row(vec![
                appearance::icon('\u{f01da}').into(),
                text("install").into(),
            ]))
            .style(theme::Button::Text)
            .into(),
        ]))
        .width(Length::Fixed(300.0))
        .style(theme::Container::Custom(Box::new(
            appearance::AppCardStyle {},
        )))
        .padding(10.0)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into()
    }
    fn installed_apps_view(&self) -> iced::Element<Message> {
        let mut apps = vec![];
        for package in flatpak_backend::get_installed_apps() {
            apps.push(self.app_card(&package));
        }
        container(
            column(vec![
                text("Installed Apps").size(30).into(),
                horizontal_rule(1.).into(),
                container(
                    wrap::Wrap::with_elements(apps)
                        .spacing(10.0)
                        .line_spacing(10.0),
                )
                .width(Length::Fill)
                .center_x()
                .into(),
            ])
            .spacing(10.0),
        )
        .padding(10.0)
        .style(theme::Container::Custom(Box::new(
            appearance::SectionsStyle {},
        )))
        .into()
    }

    fn banner(&self) -> iced::Element<Message> {
        let handle = svg::Handle::from_path(format!(
            "{}/resources/banner_small.svg",
            env!("CARGO_MANIFEST_DIR")
        ));
        svg(handle)
            .width(Length::Fill)
            .height(Length::Fixed(400.))
            .into()
    }
}

impl Application for BazaarApp {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = ();

    fn theme(&self) -> Self::Theme {
        if self.config.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            BazaarApp {
                // view: String::new(),
                config: Config { dark_mode: true },
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Bazaar")
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        // match _message {
        //     Message::Refresh => {
        //         println!("Refreshing");
        //     }
        // }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        scrollable(
            column(vec![self.installed_apps_view()])
                .padding(30.0)
                .width(Length::Fill)
                .height(Length::Shrink),
        )
        .into()
    }
}
