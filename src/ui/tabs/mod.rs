use iced::Element;
use iced_aw::TabLabel;

use super::appearance::Theme;

pub mod installed_page;
pub mod landing_page;

pub trait Tab {
    type Message;

    fn title(&self) -> String;
    fn tab_label(&self) -> TabLabel;
    fn theme(&self) -> Theme;
    fn view(&self) -> Element<Self::Message, iced::Renderer<Theme>>;
}
