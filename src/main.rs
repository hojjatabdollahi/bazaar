use std::path::PathBuf;

pub(crate) mod backend;
pub(crate) mod db;
pub(crate) mod ui;

fn main() -> iced::Result {
    let dbpath = PathBuf::from("./apps.db3");
    if !dbpath.exists() {
        let mut d = db::Storage::new().unwrap();
        d.create_table().unwrap();
        backend::flatpak_backend::get_packages_remote(&d);
    }
    ui::app::run()
}
