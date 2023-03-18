use backend::flatpak_backend;

pub(crate) mod backend;
pub(crate) mod ui;

fn main() -> iced::Result {
    // flatpak_backend::get_packages_remote();
    // Ok(())
    ui::app::run()
}
