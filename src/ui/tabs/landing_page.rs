use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use cosmic_time::{chain, container as c, id, Chain};
use cosmic_time::{Ease, Timeline};
use iced::{
    mouse::Button,
    widget::{
        self, button, column, container, horizontal_rule, horizontal_space, image, row, scrollable,
        text, text_input, Column, Container, Row,
    },
    Command, Length,
};
use iced_aw::{graphics::icons::Icon, wrap, TabLabel};
use iced_native::Widget;
use once_cell::sync::Lazy;

use super::Tab;
use crate::{
    backend::flatpak_backend::Package,
    db::search,
    ui::{
        appearance::{self, ButtonStyle, ContainerStyle, Theme},
        custom_widgets::appcard::AppCard,
        main_window::{Config, Message},
    },
};

static CONTAINER: Lazy<id::Container> = Lazy::new(id::Container::unique);

fn anim_searchbox_open() -> Chain {
    chain![
        CONTAINER,
        c(Duration::ZERO).width(0.),
        c(Duration::from_millis(200))
            .width(400.)
            .ease(Ease::Exponential(cosmic_time::Exponential::InOut)),
    ]
    .into()
}

fn anim_searchbox_close() -> Chain {
    chain![
        CONTAINER,
        c(Duration::ZERO).width(400.),
        c(Duration::from_millis(200))
            .width(0.)
            .ease(Ease::Exponential(cosmic_time::Exponential::InOut)),
    ]
    .into()
}

pub struct LandingPage {
    pub search_term: String,
    pub found_apps: Arc<Mutex<RefCell<Vec<Package>>>>,
    pub staff_pick_apps: Arc<Mutex<RefCell<Vec<Package>>>>,
    theme: Theme,
    config: Config,
    pub timeline: Timeline,
    status: Status,
    db_loading_progress: u32,
    db_loaded: bool,
}

enum Status {
    Default,
    Searching,
    StoppingSearch,
}

pub enum LandingPageMessage {
    Search(String),
    StopSearch,
    Found(Arc<Vec<Package>>),
    StaffPicks(Arc<Vec<Package>>),
    SearchButton,
    DBLoaded,
    DBLoadProgress(u32),
}

impl LandingPage {
    pub fn new(config: Config) -> Self {
        let timeline = Timeline::new();
        Self {
            search_term: Default::default(),
            found_apps: Default::default(),
            staff_pick_apps: Default::default(),
            theme: Default::default(),
            config,
            timeline,
            status: Status::Default,
            db_loading_progress: 0,
            db_loaded: false,
        }
    }

    pub fn update(&mut self, message: LandingPageMessage) -> Command<Message> {
        match message {
            LandingPageMessage::Search(st) => {
                if st.len() < 3 {
                    self.found_apps.lock().unwrap().get_mut().clear();
                }
                self.search_term = st;
                Command::none()
            }
            LandingPageMessage::StopSearch => {
                self.status = Status::StoppingSearch;
                self.search_term.clear();
                self.found_apps.lock().unwrap().get_mut().clear();
                self.timeline
                    .set_chain(anim_searchbox_close())
                    .resume(CONTAINER.clone())
                    .start();
                Command::none()
            }
            LandingPageMessage::Found(apps) => {
                println!("Found apps:{}", apps.len());
                *self.found_apps.lock().unwrap().borrow_mut() = Arc::try_unwrap(apps).unwrap();
                Command::none()
            }
            LandingPageMessage::StaffPicks(apps) => {
                println!("Staff pick apps:{}", apps.len());
                *self.staff_pick_apps.lock().unwrap().borrow_mut() = Arc::try_unwrap(apps).unwrap();
                Command::none()
            }
            LandingPageMessage::SearchButton => {
                self.status = Status::Searching;
                self.timeline
                    .set_chain(anim_searchbox_open())
                    .resume(CONTAINER.clone())
                    .start();
                widget::focus_next()
            }
            LandingPageMessage::DBLoaded => {
                self.db_loaded = true;
                Command::none()
            }
            LandingPageMessage::DBLoadProgress(progress) => {
                self.db_loading_progress = progress;
                Command::none()
            }
        }
    }

    fn app_icon<'a>(
        &self,
        width: u16,
        path: &Option<PathBuf>,
    ) -> Container<'a, Message, iced::Renderer<Theme>> {
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
    fn app_card(
        &self,
        package: &Package,
        show_buttons: bool,
        installed: bool,
    ) -> iced::Element<Message, iced::Renderer<Theme>> {
        AppCard::new(
            container(row(vec![
                self.app_icon(64, &package.icon_path).into(),
                column(vec![
                    text(&package.pretty_name.clone().unwrap_or("".to_string()))
                        .width(Length::Fixed(250.))
                        // .style(theme::Text::Color(self.style_sheet().app_card_text_color))
                        .size(28)
                        .into(),
                    text(&package.summary.clone().unwrap_or(String::from("")))
                        .width(Length::Fixed(250.))
                        // .style(theme::Text::Color(self.style_sheet().app_card_text_color))
                        .size(18)
                        .into(),
                ])
                .width(Length::Shrink)
                .into(),
                if show_buttons {
                    if installed {
                        button(appearance::icon('\u{f1767}'))
                            .on_press(Message::Uninstall(package.name.clone()))
                            .style(ButtonStyle::Icon)
                            .into()
                    } else {
                        button(appearance::icon('\u{f498}'))
                            .on_press(Message::Uninstall(package.name.clone()))
                            .style(ButtonStyle::Icon)
                            .into()
                    }
                } else {
                    Row::new().into()
                },
            ])),
            package.name.clone(),
            Message::Detail,
        )
        .width(Length::Fixed(300.0))
        .padding(10.0)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into()
    }

    fn search_view(&self) -> iced::Element<Message, iced::Renderer<Theme>> {
        let loading: iced::Element<_, _> = Column::new()
            .push(text(format!(
                "Loading the db: {}%",
                self.db_loading_progress
            )))
            .into();
        let staff_picks = column(vec![
            text("Staff Picks!").size(30).into(),
            horizontal_rule(1.).into(),
            if let Ok(staff_picks) = self.staff_pick_apps.try_lock() {
                let mut apps = vec![];
                for package in staff_picks.borrow().iter() {
                    apps.push(self.app_card(&package, false, false));
                }
                column(vec![scrollable(
                    container(
                        wrap::Wrap::with_elements(apps)
                            .spacing(10.0)
                            .line_spacing(10.0),
                    )
                    .width(Length::Fill)
                    .center_x(),
                )
                .into()])
                .spacing(10.0)
                .into()
            } else {
                column(vec![
                    text("Loading").size(30).into(),
                    horizontal_rule(1.).into(),
                ])
                .spacing(10.0)
                .into()
            },
        ])
        .spacing(10.0);

        let mut apps = vec![];
        container(if self.db_loaded {
            column(vec![
                if let Ok(found_apps) = self.found_apps.try_lock() {
                    for package in found_apps.borrow().iter() {
                        apps.push(self.app_card(&package, true, false)); // TODO: check if it is
                                                                         // installed
                    }

                    column(vec![
                        row(vec![
                            match self.status {
                                Status::Searching => id::Container::as_widget(
                                    CONTAINER.clone(),
                                    &self.timeline,
                                    text_input("Search Term", &self.search_term)
                                        .on_input(Message::Search)
                                        .padding([4.0, 12.0, 4.0, 12.0]),
                                )
                                .into(),
                                Status::StoppingSearch => {
                                    let search_box = id::Container::as_widget(
                                        CONTAINER.clone(),
                                        &self.timeline,
                                        text_input("Search Term", &self.search_term)
                                            .on_input(Message::Search)
                                            .padding([4.0, 12.0, 4.0, 12.0]),
                                    );
                                    if let Length::Fixed(width) =
                                        iced_core::widget::Widget::width(&search_box)
                                    {
                                        if width < 1. {
                                            button(appearance::icon('\u{ea6d}'))
                                                .style(ButtonStyle::Icon)
                                                .on_press(Message::SearchButton)
                                                .into()
                                        } else {
                                            search_box.into()
                                        }
                                    } else {
                                        search_box.into()
                                    }
                                }
                                Status::Default => button(appearance::icon('\u{ea6d}'))
                                    .style(ButtonStyle::Icon)
                                    .on_press(Message::SearchButton)
                                    .into(),
                            },
                            horizontal_space(Length::Fill).into(),
                            horizontal_space(Length::Fixed(10.0)).into(),
                            button(
                                row![appearance::icon('\u{f06b0}'), text("Updates"),].spacing(10.),
                            )
                            .style(ButtonStyle::Primary)
                            .padding([10, 20])
                            .on_press(Message::ChangePage(crate::ui::main_window::Page::Installed))
                            .into(),
                        ])
                        .into(),
                        horizontal_rule(4.).into(),
                        scrollable(
                            container(
                                wrap::Wrap::with_elements(apps)
                                    .spacing(10.0)
                                    .line_spacing(10.0),
                            )
                            .width(Length::Fill)
                            .center_x(),
                        )
                        .into(),
                    ])
                    .spacing(10.0)
                    .into()
                } else {
                    column(vec![
                        text("Loading").size(30).into(),
                        horizontal_rule(1.).into(),
                    ])
                    .spacing(10.0)
                    .into()
                },
                staff_picks.into(),
            ])
            .spacing(10.0)
            .into()
        } else {
            loading
        })
        .style(ContainerStyle::Default)
        .padding(10.0)
        .into()
    }
}

impl Tab for LandingPage {
    type Message = Message;

    fn title(&self) -> String {
        "Explore".into()
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        TabLabel::IconText(Icon::Check.into(), self.title())
    }

    fn theme(&self) -> appearance::Theme {
        if self.config.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn view(&self) -> iced::Element<Self::Message, iced::Renderer<Theme>> {
        self.search_view().into()
    }
}
