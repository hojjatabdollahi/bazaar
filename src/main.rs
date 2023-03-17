pub(crate) mod backend;
pub(crate) mod ui;

fn main() -> iced::Result {
    ui::app::run()
}
