use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use iced::{
    theme,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, image, row, scrollable, text,
        Column, Container,
    },
    Length,
};
use iced_aw::{graphics::icons::Icon, wrap, TabLabel};

use crate::{
    backend::flatpak_backend::Package,
    ui::{
        appearance::{self, ContainerStyle, Theme},
        main_window::{Config, Message},
    },
};

use super::Tab;

pub struct AppView {
    config: Config,
    package: Option<Package>,
}

impl AppView {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            package: None,
        }
    }

    // pub fn update(&mut self, message: InstalledPageMessage) {}

    fn app_icon<'a>(&self, width: u16) -> iced::Element<Message, iced::Renderer<Theme>> {
        let path = if let Some(package) = &self.package {
            package.icon_path.clone()
        } else {
            None
        };
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
        .into()
    }
    // fn app_card(&self, package: &Package) -> iced::Element<Message, iced::Renderer<Theme>> {
    //     container(row(vec![
    //         self.app_icon(64, &package.icon_path).into(),
    //         column(vec![
    //             text(&package.pretty_name.clone().unwrap_or("".to_string()))
    //                 .width(Length::Fixed(250.))
    //                 // .style(theme::Text::Color(self.style_sheet().app_card_text_color))
    //                 .size(28)
    //                 .into(),
    //             text(&package.summary.clone().unwrap_or(String::from("")))
    //                 .width(Length::Fixed(250.))
    //                 // .style(theme::Text::Color(self.style_sheet().app_card_text_color))
    //                 .size(18)
    //                 .into(),
    //         ])
    //         .width(Length::Shrink)
    //         .into(),
    //         button(appearance::icon('\u{f1767}'))
    //             .on_press(Message::Uninstall(package.name.clone()))
    //             // .style(theme::Button::Text)
    //             .into(),
    //     ]))
    //     .width(Length::Fixed(300.0))
    //     .style(ContainerStyle::AppCard)
    //     .padding(10.0)
    //     .width(Length::Shrink)
    //     .height(Length::Shrink)
    //     .into()
    // }
    fn header(&self) -> iced::Element<Message, iced::Renderer<Theme>> {
        // <space> <content> <space>
        // and content lookslike:
        //          <Name>
        // <Icon>   <company name>
        //          <source>
        //<size> <version> <rating>
        //
        let app_dashboard = row(vec![self.app_icon(128)]);

        container(
            Column::new()
                .push(horizontal_space(Length::Fill))
                .push(app_dashboard)
                .push(horizontal_space(Length::Fill)),
        )
        .into()
    }
    fn app_view(&self) -> iced::Element<Message, iced::Renderer<Theme>> {
        container(
            column(vec![
                self.header(),
                scrollable(
                    container(
                        text("Testing"), // wrap::Wrap::with_elements(apps)
                                         //     .spacing(10.0)
                                         //     .line_spacing(10.0),
                    )
                    .width(Length::Fill)
                    .center_x(),
                )
                .into(),
            ])
            .spacing(10.0),
        )
        .padding(10.0)
        .style(ContainerStyle::Section)
        .into()
    }
}

impl Tab for AppView {
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
        self.app_view().into()
    }
}
