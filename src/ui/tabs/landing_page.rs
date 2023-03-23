use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use cosmic_time::{keyframes, Ease, Timeline};
use iced::{
    widget::{
        self, button, column, container, horizontal_rule, horizontal_space, image, row, scrollable,
        text, text_input, Container,
    },
    Command, Length,
};
use iced_aw::{graphics::icons::Icon, wrap, TabLabel};
use once_cell::sync::Lazy;

use super::Tab;
use crate::{
    backend::flatpak_backend::Package,
    ui::{
        appearance::{self, ContainerStyle, Theme},
        main_window::{Config, Message},
    },
};

static CONTAINER: Lazy<keyframes::container::Id> = Lazy::new(keyframes::container::Id::unique);

fn anim_searchbox_open() -> cosmic_time::container::Chain {
    cosmic_time::container::Chain::new(CONTAINER.clone())
        .link(keyframes::Container::new(Duration::ZERO).width(Length::Fixed(0.)))
        .link(
            keyframes::Container::new(Duration::from_millis(200))
                .width(Length::Fixed(400.))
                .ease(Ease::Exponential(cosmic_time::Exponential::InOut)),
        )
}

pub struct LandingPage {
    pub search_term: String,
    pub found_apps: Arc<Mutex<RefCell<Vec<Package>>>>,
    theme: Theme,
    config: Config,
    pub timeline: Timeline,
    status: Status,
}

enum Status {
    Default,
    Searching,
}

pub enum LandingPageMessage {
    Search(String),
    Found(Arc<Vec<Package>>),
    SearchButton,
}

impl LandingPage {
    pub fn new(config: Config) -> Self {
        let timeline = Timeline::new();
        Self {
            search_term: Default::default(),
            found_apps: Default::default(),
            theme: Default::default(),
            config,
            timeline,
            status: Status::Default,
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
            LandingPageMessage::Found(apps) => {
                println!("Found apps:{}", apps.len());
                *self.found_apps.lock().unwrap().borrow_mut() = Arc::try_unwrap(apps).unwrap();
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
    // fn style_sheet(&self) -> StyleSheet {
    //     appearance::StyleSheet::from_theme(&self.theme())
    // }
    fn app_card(&self, package: &Package) -> iced::Element<Message, iced::Renderer<Theme>> {
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
            button(appearance::icon('\u{f1767}'))
                .on_press(Message::Uninstall(package.name.clone()))
                // .style(theme::Button::Text)
                .into(),
        ]))
        .style(ContainerStyle::AppCard)
        .width(Length::Fixed(300.0))
        // .style(theme::Container::Custom(Box::new(
        //     appearance::AppCardStyle {},
        // )))
        .padding(10.0)
        .width(Length::Shrink)
        .height(Length::Shrink)
        .into()
    }

    fn search_view(&self) -> iced::Element<Message, iced::Renderer<Theme>> {
        let mut apps = vec![];
        if let Ok(found_apps) = self.found_apps.try_lock() {
            for package in found_apps.borrow().iter() {
                apps.push(self.app_card(&package));
            }

            container(
                column(vec![
                    row(vec![
                        match self.status {
                            Status::Searching => keyframes::Container::as_widget(
                                CONTAINER.clone(),
                                &self.timeline,
                                text_input("Search Term", &self.search_term, Message::Search)
                                    .padding([4.0, 12.0, 4.0, 12.0]),
                            )
                            .into(),
                            Status::Default => button(appearance::icon('\u{ea6d}'))
                                .on_press(Message::SearchButton)
                                .into(),
                        },
                        horizontal_space(Length::Fill).into(),
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
                .spacing(10.0),
            )
            .style(ContainerStyle::Section)
            .padding(10.0)
            // .style(theme::Container::Custom(Box::new(
            //     appearance::SectionsStyle {},
            // )))
            .into()
        } else {
            container(
                column(vec![
                    text("Loading").size(30).into(),
                    horizontal_rule(1.).into(),
                ])
                .spacing(10.0),
            )
            .style(ContainerStyle::Default)
            .padding(10.0)
            // .style(theme::Container::Custom(Box::new(
            //     appearance::SectionsStyle {},
            // )))
            .into()
        }
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
