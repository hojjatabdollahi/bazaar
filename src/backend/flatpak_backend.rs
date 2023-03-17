use libflatpak::{
    traits::{InstallationExt, InstalledRefExt, RefExt},
    Installation, RefKind,
};

use std::path::PathBuf;

pub struct Package {
    pub name: String,
    pub description: String,
    pub icon_path: PathBuf,
}

pub fn get_installed_apps() -> Vec<Package> {
    let mut result = vec![];
    let sys = Installation::new_system(libflatpak::gio::Cancellable::NONE).unwrap();
    // TODO: No installed sys packages on my system, will do this later
    let installed_sys = sys
        .list_installed_refs(libflatpak::gio::Cancellable::NONE)
        .unwrap();
    for pkg in installed_sys {
        println!("sys: {:?}", pkg);
    }
    // User packages
    let user = Installation::new_user(libflatpak::gio::Cancellable::NONE).unwrap();
    let installed_user = user
        .list_installed_refs(libflatpak::gio::Cancellable::NONE)
        .unwrap();
    for pkg in installed_user {
        if pkg.kind() == RefKind::App {
            let name = pkg.appdata_name().unwrap().to_string();
            let remote: String = pkg.origin().unwrap().to_string();
            let id: String = pkg.name().unwrap().to_string();
            let arch = pkg.arch().unwrap().to_string();
            let mut path = {
                let mut path = PathBuf::new();
                for size in [64, 128, 32, 256, 16] {
                    let path_str = format!(
                        "{}/.local/share/flatpak/appstream/{}/{}/active/icons/{}x{}/{}.png",
                        env!("HOME"),
                        remote,
                        arch,
                        size,
                        size,
                        id
                    );
                    path.push(path_str);
                    if path.exists() {
                        break;
                    }
                }
                path
            };
            if !path.exists() {
                path = {
                    let mut path = PathBuf::new();
                    for size in [64, 128, 32, 256, 16] {
                        let path_str = format!(
                            "{}/.local/share/flatpak/app/{}/current/active/export/share/icons/hicolor/{}x{}/apps/{}.png",
                            env!("HOME"),
                            id,
                            size,
                            size,
                            id,
                        );
                        path.push(path_str);
                        if path.exists() {
                            break;
                        }
                    }
                    path
                };
            }
            let description = pkg.appdata_summary().unwrap().to_string();

            let package = Package {
                name,
                description,
                icon_path: path.into(),
            };
            result.push(package);
        }

        // let keyfile_bytes = pkg
        //     .load_metadata(libflatpak::gio::Cancellable::NONE)
        //     .unwrap();
        // println!("{:?}", pkg.get_property_appdata_version());
        // println!("{:?}", pkg.appdata_name());
        // println!("{:?}", pkg.name());
        // println!("{:?}", pkg.format_ref());
        // println!("{:?}", pkg.get_property_appdata_name());
        // println!("{:?}", pkg.get_property_appdata_content_rating_type());
        // println!("{:?}", pkg.appdata_version());
        // println!("{:?}", pkg.installed_size());
        // println!("{:?}", pkg.deploy_dir());
        // println!("{:?}", pkg.subpaths());
        // println!("{:?}", pkg.eol());
        // println!("{:?}", pkg.eol_rebase());
        // println!("{:?}", pkg.end_of_life());
        // println!("{:?}", pkg.end_of_life_rebase());
        // println!("{:?}", pkg.origin());
        // println!("{:?}", pkg.appdata_summary());
    }
    result
}
