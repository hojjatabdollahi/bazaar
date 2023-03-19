// use crate::ui::action::refresh;
use iced::{
    executor,
    futures::channel::mpsc,
    theme,
    widget::{button, column, container, horizontal_rule, image, row, scrollable, text, Container},
    Application, Length, Settings, Theme,
};
use iced_aw::wrap;
use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::backend::flatpak_backend::{uninstall, Package, PackageId};

use super::{
    action,
    appearance::{self, StyleSheet},
};

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
    pub installed_apps: Arc<Mutex<RefCell<Vec<Package>>>>,
    pub config: Config,
    status: String,
    action: mpsc::Sender<action::action::Action>,
}

#[derive(Debug, Clone)]
enum Message {
    RequestRefreshInstalledApps,
    Uninstall(PackageId),
    ActionMessage(action::action::Message),
}

impl BazaarApp {
    fn app_icon<'a>(&self, width: u16, path: &Option<PathBuf>) -> Container<'a, Message> {
        let path = path
            .clone()
            .unwrap_or(format!("{}/resources/DefaultApp.png", env!("CARGO_MANIFEST_DIR")).into());
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
                text(&package.pretty_name.clone().unwrap_or("".to_string()))
                    .width(Length::Fixed(250.))
                    .style(theme::Text::Color(self.style_sheet().app_card_text_color))
                    .size(28)
                    .into(),
                text(&package.description.clone().unwrap_or(String::from("")))
                    .width(Length::Fixed(250.))
                    .style(theme::Text::Color(self.style_sheet().app_card_text_color))
                    .size(18)
                    .into(),
            ])
            .width(Length::Shrink)
            .into(),
            button(appearance::icon('\u{f1767}'))
                .on_press(Message::Uninstall(package.name.clone()))
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
        if let Ok(installed_apps) = self.installed_apps.try_lock() {
            for package in installed_apps.borrow().iter() {
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
        } else {
            container(
                column(vec![
                    text("Loading").size(30).into(),
                    horizontal_rule(1.).into(),
                ])
                .spacing(10.0),
            )
            .padding(10.0)
            .style(theme::Container::Custom(Box::new(
                appearance::SectionsStyle {},
            )))
            .into()
        }
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
        let (tx, _) = mpsc::channel(1);
        (
            BazaarApp {
                config: Config { dark_mode: true },
                installed_apps: Default::default(),
                status: "nothing".into(),
                action: tx,
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Bazaar")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        action::action::subscribe().map(Message::ActionMessage)
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::RequestRefreshInstalledApps => {
                let _ = self
                    .action
                    .start_send(action::action::Action::RefreshInstalled);
            }
            Message::Uninstall(id) => {
                println!("Uninstalling {}", id);
                uninstall(&id);
            }
            Message::ActionMessage(msg) => match msg {
                action::action::Message::Ready(tx) => {
                    self.action = tx;
                }
                action::action::Message::Refreshed(apps) => {
                    println!("Refreshed installed apps");
                    *self.installed_apps.lock().unwrap().borrow_mut() =
                        Arc::try_unwrap(apps).unwrap();
                }
            },
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        scrollable(
            column(vec![
                button("refresh")
                    .on_press(Message::RequestRefreshInstalledApps)
                    .into(),
                text(self.status.clone()).into(),
                self.installed_apps_view(),
            ])
            .padding(30.0)
            .width(Length::Fill)
            .height(Length::Shrink),
        )
        .into()
    }
}
