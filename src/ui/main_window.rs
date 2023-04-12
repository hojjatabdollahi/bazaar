use cosmic_time::{keyframes, Timeline};
use iced::{
    event, executor,
    futures::channel::mpsc,
    keyboard::{self, Modifiers},
    subscription,
    widget::{button, column, container, text},
    window, Alignment, Application, Color, Element, Event, Length, Settings,
};

use iced_aw::{tabs::TabBarStyles, TabBar, Tabs};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{backend::flatpak_backend::PackageId, db::Storage};

use super::{
    action, appearance,
    tabs::app_view::AppView,
    toast::{self, Status, Toast},
};
use super::{
    appearance::Theme,
    tabs::{
        installed_page::{InstalledPage, InstalledPageMessage},
        landing_page::{LandingPage, LandingPageMessage},
        Tab,
    },
};

use once_cell::sync::Lazy;
static CONTAINER: Lazy<keyframes::container::Id> = Lazy::new(keyframes::container::Id::unique);

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
    app_view_page: AppView,
    active_tab: usize,
    timeline: Timeline,
    current_page: Page,
    toasts: Vec<Toast>,
    timeout_secs: u64,
}

#[derive(Debug, Clone)]
pub enum Page {
    LandingPage,
    Installed,
    Detail,
}

#[derive(Debug, Clone)]
pub enum Message {
    RequestRefreshInstalledApps,
    RequestRefreshStaffPickApps,
    Install(PackageId),
    Uninstall(PackageId),
    Detail(PackageId),
    ActionMessage(action::Message),
    Search(String),
    SearchButton,
    IncreaseScalingFactor,
    DecreaseScalingFactor,
    TabSelected(usize),
    Tick(Instant),
    Close(usize),
    ChangePage(Page),
    StopSearch,
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
        let mut timeline = Timeline::new();
        let animation = cosmic_time::container::Chain::new(CONTAINER.clone())
            .link(keyframes::Container::new(Duration::ZERO).width(Length::Fixed(0.)))
            .link(keyframes::Container::new(Duration::from_millis(700)).width(Length::Fixed(800.)));
        timeline.set_chain(animation).start();

        (
            BazaarApp {
                config: config.clone(),
                action: tx,
                db,
                scaling_factor: 1.0,
                landing_page: LandingPage::new(config.clone()),
                installed_page: InstalledPage::new(config.clone()),
                app_view_page: AppView::new(config.clone()),
                active_tab: Default::default(),
                timeline,
                current_page: Page::LandingPage,
                toasts: vec![Toast {
                    title: "Loading...".into(),
                    body: "Updating the database. Please wait...".into(),
                    status: Status::Primary,
                }],
                timeout_secs: 30, /* toast::DEFAULT_TIMEOUT */
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
            self.landing_page
                .timeline
                .as_subscription::<Event>()
                .map(Message::Tick),
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
                        key_code: keyboard::KeyCode::F,
                        modifiers: Modifiers::CTRL,
                        ..
                    }),
                    event::Status::Ignored,
                ) => Some(Message::SearchButton),
                (
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key_code: keyboard::KeyCode::Equals,
                        modifiers: Modifiers::CTRL,
                        ..
                    }),
                    event::Status::Ignored,
                ) => Some(Message::IncreaseScalingFactor),
                (
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key_code: keyboard::KeyCode::Escape,
                        ..
                    }),
                    event::Status::Ignored,
                ) => Some(Message::StopSearch),
                _ => None,
            }),
        ])
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::RequestRefreshInstalledApps => {
                let _ = self.action.start_send(action::Action::RefreshInstalled);
            }
            Message::RequestRefreshStaffPickApps => {
                let _ = self
                    .action
                    .start_send(action::Action::RefreshStaffPicks(self.db.clone()));
            }
            Message::Install(id) => {
                println!("Installing {}", id);
                // TODO
            }
            Message::Uninstall(id) => {
                println!("Uninstalling {}", id);
                let _ = self
                    .action
                    .start_send(action::Action::Uninstall(id.clone()));
            }
            Message::Search(st) => {
                if st.len() >= 3 {
                    println!("searching for {}", st);
                    let _ = self
                        .action
                        .start_send(action::Action::Search((self.db.clone(), st.clone())));
                }
                let _ = self.landing_page.update(LandingPageMessage::Search(st));
            }
            Message::StopSearch => {
                let _ = self.landing_page.update(LandingPageMessage::StopSearch);
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
            Message::Detail(id) => {
                println!("Show detail for the app: {id:?}");
                self.current_page = Page::Detail;
            }
            Message::ChangePage(page) => {
                self.current_page = page;
            }
            Message::Close(index) => {
                self.toasts.remove(index);
            }
            Message::ActionMessage(msg) => match msg {
                action::Message::Ready(tx) => {
                    self.action = tx;
                    let _ = self
                        .action
                        .start_send(action::Action::RefreshStaffPicks(self.db.clone()));
                    let _ = self.action.start_send(action::Action::RefreshInstalled);
                }
                action::Message::Refreshed(apps) => {
                    self.installed_page
                        .update(InstalledPageMessage::Refreshed(apps));
                }
                action::Message::StaffPicks(apps) => {
                    let _ = self
                        .landing_page
                        .update(LandingPageMessage::StaffPicks(apps));
                }
                action::Message::Found(apps) => {
                    return self.landing_page.update(LandingPageMessage::Found(apps));
                }
                action::Message::Uninstalled(id) => {
                    println!("Uninstalled {:?}", id);
                    let _ = self.action.start_send(action::Action::RefreshInstalled);
                }
            },
            Message::Tick(now) => self.landing_page.timeline.now(now),
            Message::SearchButton => {
                return self.landing_page.update(LandingPageMessage::SearchButton);
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let content = container(match self.current_page {
            Page::LandingPage => column(vec![self.landing_page.view().into()]).spacing(10.),
            Page::Installed => column(vec![self.installed_page.view().into()]).spacing(10.),
            Page::Detail => column(vec![self.app_view_page.view().into()]),
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10.0);

        // toast::Manager::new(content, &self.toasts, Message::Close)
        //     .timeout(self.timeout_secs)
        //     .into()
        content.into()
    }

    fn scale_factor(&self) -> f64 {
        self.scaling_factor
    }
}
