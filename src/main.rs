use std::path::PathBuf;

pub(crate) mod backend;
pub(crate) mod db;
pub(crate) mod ui;

fn main() -> iced::Result {
    ui::main_window::run()
}
