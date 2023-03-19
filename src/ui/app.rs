// use crate::ui::action::refresh;
use iced::{
    executor,
    futures::channel::mpsc,
    theme,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, image, row, scrollable, text,
        text_input, Container, Row,
    },
    window, Application, Length, Settings, Theme,
};
use iced_aw::wrap;
use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::{
    backend::flatpak_backend::{Package, PackageId},
    db::Storage,
};

use super::{
    action,
    appearance::{self, StyleSheet},
};

pub fn run() -> iced::Result {
    BazaarApp::run(Settings {
        default_font: Some(appearance::NOTO_SANS),
        window: window::Settings {
            decorations: true,
            transparent: true,
            ..window::Settings::default()
        },
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
    action: mpsc::Sender<action::Action>,
    db: Arc<Mutex<Storage>>,
    pub found_apps: Arc<Mutex<RefCell<Vec<Package>>>>,
    search_term: String,
}

#[derive(Debug, Clone)]
enum Message {
    RequestRefreshInstalledApps,
    Uninstall(PackageId),
    ActionMessage(action::Message),
    Search(String),
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
                text(&package.summary.clone().unwrap_or(String::from("")))
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
                    row(vec![
                        text("Installed Apps").size(30).into(),
                        horizontal_space(Length::Fill).into(),
                        button(appearance::icon('\u{eb37}'))
                            .on_press(Message::RequestRefreshInstalledApps)
                            .padding(10.)
                            .style(theme::Button::Text)
                            .into(),
                    ])
                    .into(),
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

    fn search_view(&self) -> iced::Element<Message> {
        let mut apps = vec![];
        if let Ok(found_apps) = self.found_apps.try_lock() {
            for package in found_apps.borrow().iter() {
                apps.push(self.app_card(&package));
            }

            container(
                column(vec![
                    row(vec![text_input(
                        "Search Term",
                        &self.search_term,
                        Message::Search,
                    )
                    .into()])
                    .into(),
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
        let mut db = Storage::new().unwrap();
        db.create_table().unwrap();
        db.all_packages = Some(db.all_names().unwrap());
        let db = Arc::new(Mutex::new(db));
        (
            BazaarApp {
                config: Config { dark_mode: true },
                installed_apps: Default::default(),
                status: "nothing".into(),
                action: tx,
                db,
                search_term: Default::default(),
                found_apps: Default::default(),
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Bazaar")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        action::subscribe().map(Message::ActionMessage)
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::RequestRefreshInstalledApps => {
                let _ = self.action.start_send(action::Action::RefreshInstalled);
            }
            Message::Uninstall(id) => {
                println!("Uninstalling {}", id);
                let _ = self.action.start_send(action::Action::Uninstall(id));
            }
            Message::Search(st) => {
                self.search_term = st.clone();
                if st.len() >= 3 {
                    println!("searching for {}", st);
                    let _ = self
                        .action
                        .start_send(action::Action::Search((self.db.clone(), st)));
                } else {
                    self.found_apps.lock().unwrap().get_mut().clear();
                }
            }
            Message::ActionMessage(msg) => match msg {
                action::Message::Ready(tx) => {
                    self.action = tx;
                }
                action::Message::Refreshed(apps) => {
                    println!("Refreshed installed apps");
                    *self.installed_apps.lock().unwrap().borrow_mut() =
                        Arc::try_unwrap(apps).unwrap();
                }
                action::Message::Found(apps) => {
                    println!("Found apps: {}", apps.len());
                    *self.found_apps.lock().unwrap().borrow_mut() = Arc::try_unwrap(apps).unwrap();
                }
                action::Message::Uninstalled(id) => {
                    println!("Uninstalled {:?}", id);
                    let _ = self.action.start_send(action::Action::RefreshInstalled);
                }
            },
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        container(column(vec![
            scrollable(
                column(vec![self.search_view()])
                    .padding(30.0)
                    .width(Length::Fill)
                    .height(Length::Shrink),
            )
            .into(),
            scrollable(
                column(vec![self.installed_apps_view()])
                    .padding(30.0)
                    .width(Length::Fill)
                    .height(Length::Shrink),
            )
            .into(),
        ]))
        .into()
    }
}
