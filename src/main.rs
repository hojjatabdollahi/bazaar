pub(crate) mod backend;
pub(crate) mod db;
pub(crate) mod ui;

fn main() -> iced::Result {
    // let mut d = db::Storage::new().unwrap();
    // d.create_table().unwrap();
    // d.find()?;
    // // backend::flatpak_backend::get_packages_remote(&d);
    // // d.insert(&db::Person {
    // //     id: 0,
    // //     name: "Steven".into(),
    // //     data: None,
    // // })
    // // .unwrap();
    // // d.print_all().unwrap();
    // Ok(())

    ui::app::run()
}
