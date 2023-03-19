// use crate::ui::action::refresh;
use iced::{
    executor,
    futures::{channel::mpsc, SinkExt},
    theme,
    widget::{
        button, column, container, horizontal_rule, image, row, scrollable, svg, text, Container,
    },
    Application, Length, Settings, Theme,
};
use iced_aw::wrap;
use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::backend::flatpak_backend::{self, uninstall, Package, PackageId};

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

#[derive(Debug)]
enum AppStatus {
    Working,
    Still,
}

struct BazaarApp {
    // pub view: String,
    pub installed_apps: Arc<Mutex<RefCell<Vec<Package>>>>,
    pub config: Config,
    status: String,
    // actions: Vec<Action>,
    action_counter: usize,
    action: mpsc::Sender<action::action::Action>,
}

#[derive(Debug, Clone)]
enum Message {
    Start(()),
    RequestRefreshInstalledApps,
    // Refresh(refresh::Event),
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
            // button(row(vec![
            //     appearance::icon('\u{f01da}').into(),
            //     text("install").into(),
            // ]))
            // .style(theme::Button::Text)
            // .into(),
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

    async fn dummy() {}
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
        let (tx, rx) = mpsc::channel(1);
        (
            BazaarApp {
                // view: String::new(),
                config: Config { dark_mode: true },
                installed_apps: Default::default(),
                status: "nothing".into(),
                // actions: Default::default(),
                action_counter: Default::default(),
                action: tx,
            },
            iced::Command::perform(Self::dummy(), Message::Start),
        )
    }

    fn title(&self) -> String {
        String::from("Bazaar")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        // iced::Subscription::batch(self.actions.iter().map(|action| match action.kind {
        //     ActionKind::RefreshInstalled => refresh::refresh(action).map(Message::Refresh),
        // }))
        action::action::subscribe().map(Message::ActionMessage)
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Start(_) => {
                self.action_counter += 1;
                // self.actions.push(Action {
                //     id: self.action_counter,
                //     kind: ActionKind::RefreshInstalled,
                //     installed_apps: self.installed_apps.clone(),
                // });
                let res = self.action.start_send(action::action::Action::StartA);
            }
            Message::RequestRefreshInstalledApps => {
                let res = self
                    .action
                    .start_send(action::action::Action::RefreshInstalled);
            }
            Message::Uninstall(id) => {
                println!("Uninstalling {}", id);
                uninstall(&id);
            }
            // Message::Refresh(event) => match event {
            //     refresh::Event::Started(mut sender) => {
            //         self.status = "Started".into();
            //         let res = sender.try_send(refresh::Message::Load(self.installed_apps.clone()));
            //     }
            //     refresh::Event::Done => {
            //         self.status = "Done".into();
            //     }
            // },
            // Message::Refresh(event) => match event {
            //     refresh::Event::Started => {
            //         self.status = "Started".into();
            //     }
            //     refresh::Event::Done(id) => {
            //         // println!("Deleting action {:?}", id);
            //         //
            //         // let mut idx_wr = 0usize;
            //         // for idx_rd in 0..self.actions.len() {
            //         //     if !(self.actions[idx_rd].id == id) {
            //         //         self.actions.swap(idx_wr, idx_rd);
            //         //         idx_wr += 1;
            //         //     }
            //         // }
            //         // self.actions.truncate(idx_wr);
            //         self.status = "Done".into();
            //     }
            // },
            Message::ActionMessage(msg) => match msg {
                action::action::Message::Ready(tx) => {
                    self.action = tx;
                }
                action::action::Message::Refreshed(apps) => {
                    println!("Refreshed installed apps");
                    *self.installed_apps.lock().unwrap().borrow_mut() =
                        Arc::try_unwrap(apps).unwrap();
                }
                action::action::Message::WorkAStarted => {
                    println!("work a started");
                }
                action::action::Message::WorkBStarted => {
                    println!("work b started");
                }
                action::action::Message::CleanUpStarted => {
                    println!("cleanup started");
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
