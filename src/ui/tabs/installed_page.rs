use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use iced::{
    theme,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, image, row, scrollable, text,
        Container,
    },
    Length,
};
use iced_aw::{graphics::icons::Icon, wrap, TabLabel};

use crate::{
    backend::flatpak_backend::Package,
    ui::{
        appearance::{self, ButtonStyle, ContainerStyle, Theme},
        custom_widgets::appcard::AppCard,
        main_window::{Config, Message},
    },
};

use super::Tab;

pub struct InstalledPage {
    config: Config,

    pub installed_apps: Arc<Mutex<RefCell<Vec<Package>>>>,
    pub Updatable_apps: Arc<Mutex<RefCell<Vec<Package>>>>,
}

pub enum InstalledPageMessage {
    Installed(Arc<Vec<Package>>),
    Updates(Arc<Vec<Package>>),
}

impl InstalledPage {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            installed_apps: Default::default(),
            Updatable_apps: Default::default(),
        }
    }

    pub fn update(&mut self, message: InstalledPageMessage) {
        match message {
            InstalledPageMessage::Installed(apps) => {
                println!("Refreshed installed apps");
                *self.installed_apps.lock().unwrap().borrow_mut() = Arc::try_unwrap(apps).unwrap();
            }
            InstalledPageMessage::Updates(apps) => {
                println!("Refreshed updates");
                *self.Updatable_apps.lock().unwrap().borrow_mut() = Arc::try_unwrap(apps).unwrap();
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

    fn app_card(&self, package: &Package) -> iced::Element<Message, iced::Renderer<Theme>> {
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
                button(appearance::icon('\u{f1767}'))
                    .on_press(Message::Uninstall(package.name.clone()))
                    .style(ButtonStyle::Icon)
                    .into(),
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

    fn installed_apps_view(&self) -> iced::Element<Message, iced::Renderer<Theme>> {
        container(
            column(vec![
                button(appearance::icon('\u{f030d}'))
                    .on_press(Message::ChangePage(
                        crate::ui::main_window::Page::LandingPage,
                    ))
                    .padding(10.)
                    .style(ButtonStyle::Icon)
                    .into(),
                if let Ok(udpatable_apps) = self.Updatable_apps.try_lock() {
                    let mut apps = vec![];
                    for package in udpatable_apps.borrow().iter() {
                        apps.push(self.app_card(&package));
                    }
                    column(vec![
                        row(vec![
                            text("Updates").size(30).into(),
                            horizontal_space(Length::Fill).into(),
                            button(appearance::icon('\u{eb37}'))
                                .on_press(Message::RequestRefreshInstalledApps)
                                .padding(10.)
                                .style(ButtonStyle::Icon)
                                .into(),
                        ])
                        .into(),
                        horizontal_rule(1.).into(),
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
                if let Ok(installed_apps) = self.installed_apps.try_lock() {
                    let mut apps = vec![];
                    for package in installed_apps.borrow().iter() {
                        apps.push(self.app_card(&package));
                    }
                    column(vec![
                        row(vec![
                            text("Installed Apps").size(30).into(),
                            horizontal_space(Length::Fill).into(),
                            button(appearance::icon('\u{eb37}'))
                                .on_press(Message::RequestRefreshInstalledApps)
                                .padding(10.)
                                .style(ButtonStyle::Icon)
                                .into(),
                        ])
                        .into(),
                        horizontal_rule(1.).into(),
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
            ])
            .spacing(10.0),
        )
        .padding(10.0)
        .style(ContainerStyle::Default)
        .into()
    }
}

impl Tab for InstalledPage {
    type Message = Message;

    fn title(&self) -> String {
        "Installed".into()
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        TabLabel::IconText(Icon::Check.into(), self.title())
    }

    fn theme(&self) -> Theme {
        if self.config.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn view(&self) -> iced::Element<Self::Message, iced::Renderer<Theme>> {
        self.installed_apps_view().into()
    }
}
