use iced::{
    event, executor,
    futures::channel::mpsc,
    keyboard::{self, Modifiers},
    subscription,
    widget::{container, text},
    window, Application, Event, Settings,
};

use iced_aw::{tabs::TabBarStyles, Tabs};
use std::sync::{Arc, Mutex};

use crate::{backend::flatpak_backend::PackageId, db::Storage};

use super::{action, appearance};
use super::{
    appearance::Theme,
    tabs::{
        installed_page::{InstalledPage, InstalledPageMessage},
        landing_page::{LandingPage, LandingPageMessage},
        Tab,
    },
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

#[derive(Clone)]
pub struct Config {
    pub dark_mode: bool,
}

struct BazaarApp {
    pub config: Config,
    action: mpsc::Sender<action::Action>,
    db: Arc<Mutex<Storage>>,
    scaling_factor: f64,
    landing_page: LandingPage,
    installed_page: InstalledPage,
    active_tab: usize,
}

#[derive(Debug, Clone)]
pub enum Message {
    RequestRefreshInstalledApps,
    Uninstall(PackageId),
    ActionMessage(action::Message),
    Search(String),
    IncreaseScalingFactor,
    DecreaseScalingFactor,
    TabSelected(usize),
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
        let config = Config { dark_mode: true };
        let db = Arc::new(Mutex::new(db));
        (
            BazaarApp {
                config: config.clone(),
                action: tx,
                db,
                scaling_factor: 1.0,
                landing_page: LandingPage::new(config.clone()),
                installed_page: InstalledPage::new(config.clone()),
                active_tab: Default::default(),
            },
            iced::Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Bazaar")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::batch([
            action::subscribe().map(Message::ActionMessage),
            subscription::events_with(|event, status| match (event, status) {
                (
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key_code: keyboard::KeyCode::Minus,
                        modifiers: Modifiers::CTRL,
                        ..
                    }),
                    event::Status::Ignored,
                ) => Some(Message::DecreaseScalingFactor),
                (
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key_code: keyboard::KeyCode::Equals,
                        modifiers: Modifiers::CTRL,
                        ..
                    }),
                    event::Status::Ignored,
                ) => Some(Message::IncreaseScalingFactor),
                _ => None,
            }),
        ])
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
                if st.len() >= 3 {
                    println!("searching for {}", st);
                    let _ = self
                        .action
                        .start_send(action::Action::Search((self.db.clone(), st.clone())));
                }
                self.landing_page.update(LandingPageMessage::Search(st));
            }
            Message::IncreaseScalingFactor => {
                self.scaling_factor += 0.1;
            }
            Message::DecreaseScalingFactor => {
                self.scaling_factor -= 0.1;
            }
            Message::TabSelected(new_tab) => {
                self.active_tab = new_tab;
            }
            Message::ActionMessage(msg) => match msg {
                action::Message::Ready(tx) => {
                    self.action = tx;
                }
                action::Message::Refreshed(apps) => {
                    self.installed_page
                        .update(InstalledPageMessage::Refreshed(apps));
                }
                action::Message::Found(apps) => {
                    self.landing_page.update(LandingPageMessage::Found(apps));
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
        // TabBar::new(self.active_tab, Message::TabSelected)
        //     .push(self.landing_page.tab_label())
        //     .push(self.installed_page.tab_label())
        //     .style(CustomTabBarStyles::Dark)
        //     .into()
        Tabs::new(self.active_tab, Message::TabSelected)
            .push(self.landing_page.tab_label(), self.landing_page.view())
            .push(self.installed_page.tab_label(), self.installed_page.view())
            // .tab_bar_style(if self.config.dark_mode {
            //     Theme::Dark
            // } else {
            //     Theme::Light
            // })
            .into()
        // container(
        //     Tabs::new(0, Message::TabSelected), //     .push(self.landing_page.tab_label(), self.landing_page.view()),
        //                                         // self.landing_page.view(),
        // )
        // .into()
    }

    fn scale_factor(&self) -> f64 {
        self.scaling_factor
    }
}
