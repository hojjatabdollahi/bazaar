use cosmic_time::{chain, container as c, id, Timeline};
use iced::{
    event, executor,
    futures::{channel::mpsc, StreamExt},
    keyboard::{self, Modifiers},
    subscription,
    widget::{button, column, container, text},
    window, Alignment, Application, Color, Element, Event, Length, Settings,
};

use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    backend::{self, flatpak_backend::PackageId},
    db::{self, Storage},
};

use super::{
    action, appearance,
    custom_widgets::toast::{self, Status, Toast},
    tabs::app_view::AppView,
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
static CONTAINER: Lazy<id::Container> = Lazy::new(id::Container::unique);

pub fn run() -> iced::Result {
    BazaarApp::run(Settings {
        // default_font: Some(appearance::NOTO_SANS),
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
    action: Option<mpsc::Sender<action::Action>>,
    db: Option<Arc<Mutex<Storage>>>,
    db_stream: Option<mpsc::Sender<db::Action>>,
    db_progress: Option<mpsc::Receiver<db::Message>>,
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
    FontLoaded(Result<(), iced::font::Error>),
    RequestRefreshInstalledApps,
    RequestRefreshUpdates,
    RequestRefreshStaffPickApps,
    Install(PackageId),
    Uninstall(PackageId),
    Detail(PackageId),
    ActionMessage(action::Message),
    DBMessage(db::Message),
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
        // let mut db = Storage::new().unwrap();
        // db.create_table().unwrap();
        // db.all_packages = Some(db.all_names().unwrap());
        let config = Config { dark_mode: true };
        let db = None;
        let mut timeline = Timeline::new();
        let animation = chain![
            CONTAINER,
            c(Duration::ZERO).width(0.),
            c(Duration::from_millis(700)).width(800.),
        ];
        timeline.set_chain(animation).start();

        (
            BazaarApp {
                config: config.clone(),
                action: None,
                db,
                db_stream: None,
                db_progress: None,
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
            iced::font::load(include_bytes!("../../fonts/nerd_font.ttf").as_slice())
                .map(Message::FontLoaded),
        )
    }

    fn title(&self) -> String {
        String::from("Bazaar")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::batch([
            action::subscribe().map(Message::ActionMessage),
            db::subscribe().map(Message::DBMessage),
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
        if self.db_progress.is_some() {
            match self.db_progress.as_mut().unwrap().try_next() {
                Ok(Some(db::Message::Progress(p))) => {
                    let _ = self
                        .landing_page
                        .update(LandingPageMessage::DBLoadProgress(p));
                }
                _ => {}
            };
        }
        match message {
            Message::RequestRefreshInstalledApps => {
                let _ = self
                    .action
                    .as_mut()
                    .map(|tx| tx.start_send(action::Action::RefreshInstalled));
            }
            Message::RequestRefreshUpdates => {
                let _ = self
                    .action
                    .as_mut()
                    .map(|tx| tx.start_send(action::Action::RefreshUpdates));
            }
            Message::RequestRefreshStaffPickApps => {
                let _ = self.action.as_mut().map(|tx| {
                    tx.start_send(action::Action::RefreshStaffPicks(
                        self.db.as_ref().unwrap().clone(),
                    ))
                });
            }
            Message::Install(id) => {
                println!("Installing {}", id);
                // TODO
            }
            Message::Uninstall(id) => {
                println!("Uninstalling {}", id);
                let _ = self
                    .action
                    .as_mut()
                    .map(|tx| tx.start_send(action::Action::Uninstall(id.clone())));
            }
            Message::Search(st) => {
                if st.len() >= 3 {
                    println!("searching for {}", st);
                    let _ = self.action.as_mut().map(|tx| {
                        tx.start_send(action::Action::Search((
                            self.db.as_ref().unwrap().clone(),
                            st.clone(),
                        )))
                    });
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
            Message::DBMessage(msg) => match msg {
                db::Message::Ready(tx) => {
                    self.db_stream = Some(tx);
                    let (tx, rx) = mpsc::channel::<db::Message>(10);
                    self.db_progress = Some(rx);
                    let _ = self
                        .db_stream
                        .as_mut()
                        .map(|dbtx| dbtx.start_send(db::Action::Load(tx)));
                }
                db::Message::Progress(_) => {}
                db::Message::Loaded(db) => {
                    self.db = Some(db);
                    let _ = self.landing_page.update(LandingPageMessage::DBLoaded);
                    let _ = self.action.as_mut().map(|tx| {
                        tx.start_send(action::Action::RefreshStaffPicks(
                            self.db.as_ref().unwrap().clone(),
                        ))
                    });
                    let _ = self
                        .action
                        .as_mut()
                        .map(|tx| tx.start_send(action::Action::RefreshInstalled));
                    let _ = self
                        .action
                        .as_mut()
                        .map(|tx| tx.start_send(action::Action::RefreshUpdates));
                }
            },
            Message::ActionMessage(msg) => match msg {
                action::Message::Ready(tx) => {
                    self.action = Some(tx);
                }
                action::Message::Installed(apps) => {
                    self.installed_page
                        .update(InstalledPageMessage::Installed(apps));
                }
                action::Message::Updates(apps) => {
                    self.installed_page
                        .update(InstalledPageMessage::Updates(apps));
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
                    let _ = self
                        .action
                        .as_mut()
                        .map(|tx| tx.start_send(action::Action::RefreshInstalled));
                }
            },
            Message::Tick(now) => self.landing_page.timeline.now(now),
            Message::SearchButton => {
                return self.landing_page.update(LandingPageMessage::SearchButton);
            }
            Message::FontLoaded(_) => (),
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
        // content.into()
        toast::Manager::new(content, &self.toasts, Message::Close)
            .timeout(self.timeout_secs)
            .into()
    }

    fn scale_factor(&self) -> f64 {
        self.scaling_factor
    }
}
