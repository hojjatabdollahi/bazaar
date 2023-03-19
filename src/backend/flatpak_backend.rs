use appstream::{enums::Bundle, AppId, Collection, Component};
use libflatpak::{
    builders::TransactionBuilder,
    ffi::FlatpakTransaction,
    gio::{traits::FileExt, Cancellable},
    prelude::*,
    traits::{InstallationExt, InstalledRefExt, RefExt, RemoteExt, RemoteRefExt},
    Installation, InstalledRef, RefKind, RemoteRef, Transaction,
};

use std::path::PathBuf;

#[derive(Debug)]
pub enum PackageKind {
    App,
    Runtime,
    Extension,
}

pub type PackageId = String;

impl From<RefKind> for PackageKind {
    fn from(value: RefKind) -> Self {
        match value {
            RefKind::App => PackageKind::App,
            RefKind::Runtime => PackageKind::Runtime,
            _ => PackageKind::Extension,
        }
    }
}

#[derive(Debug)]
pub struct Package {
    pub name: PackageId,
    pub pretty_name: Option<String>,
    pub description: Option<String>,
    pub icon_path: Option<PathBuf>,
    pub kind: PackageKind,
}

impl From<InstalledRef> for Package {
    fn from(pkg: InstalledRef) -> Self {
        let pretty_name = pkg.appdata_name().map(|s| s.to_string());
        let remote: String = pkg.origin().unwrap().to_string();
        let name: String = pkg.name().unwrap().to_string();
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
                    name
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
                            name,
                            size,
                            size,
                            name,
                        );
                    path.push(path_str);
                    if path.exists() {
                        break;
                    }
                }
                path
            };
        }
        let icon_path;
        if path.exists() {
            icon_path = Some(path);
        } else {
            icon_path = None;
        }

        let kind = PackageKind::from(pkg.kind());
        let description = pkg.appdata_summary().map(|s| s.to_string());

        Package {
            name,
            pretty_name,
            description,
            icon_path,
            kind,
        }
    }
}

pub fn get_installed_apps() -> Vec<Package> {
    println!("Getting installed packages");
    let mut result = vec![];
    // let sys = Installation::new_system(libflatpak::gio::Cancellable::NONE).unwrap();
    // // TODO: No installed sys packages on my system, will do this later
    // let installed_sys = sys
    //     .list_installed_refs(libflatpak::gio::Cancellable::NONE)
    //     .unwrap();
    // for pkg in installed_sys {
    //     println!("sys: {:?}", pkg);
    // }
    // User packages
    let user = Installation::new_user(libflatpak::gio::Cancellable::NONE).unwrap();
    let installed_user = user
        .list_installed_refs(libflatpak::gio::Cancellable::NONE)
        .unwrap();
    let mut package_id = 0;
    for pkg in installed_user {
        if pkg.kind() == RefKind::App {
            result.push(Package::from(pkg));
            package_id += 1;
        }
    }
    result
}

pub fn uninstall(name: &str) {
    let user = Installation::new_user(libflatpak::gio::Cancellable::NONE).unwrap();
    let installed_user = user
        .list_installed_refs(libflatpak::gio::Cancellable::NONE)
        .unwrap();
    for pkg in installed_user {
        if pkg.kind() == RefKind::App {
            if pkg.name().unwrap_or("".into()).to_string() == name {
                let t = Transaction::for_installation(
                    user.as_ref() as &Installation,
                    Cancellable::NONE,
                )
                .unwrap();
                let res = t.add_uninstall(pkg.format_ref().unwrap().as_str());
                println!("Added the transaction: {:?}", res);
                let res = t.run(Cancellable::NONE);
                println!("Finished the transaction: {:?}", res);
                break;
            }
        }
    }
}

pub fn get_packages_remote() {
    println!("Refreshing");

    let sys = Installation::new_system(libflatpak::gio::Cancellable::NONE).unwrap();
    // TODO: No installed sys packages on my system, will do this later
    let remotes = sys.list_remotes(Cancellable::NONE).unwrap();
    for remote in remotes {
        println!("remote: {:?}", remote);
    }
    let user = Installation::new_user(libflatpak::gio::Cancellable::NONE).unwrap();
    let remotes = user.list_remotes(Cancellable::NONE).unwrap();
    for remote in remotes {
        println!("remote: {:?}", remote);
        let url = remote.url().unwrap().to_string();
        let remote_name = remote.name().unwrap().to_string();
        println!("Load remote {}: {}", remote_name, url);
        let packages = user
            .list_remote_refs_sync(&remote_name, Cancellable::NONE)
            .unwrap();
        for pkg in &packages {
            println!("Ref: {}", pkg.name().unwrap().to_string());
        }
        let mut appstream_file = PathBuf::new();
        let appstream_dir = remote.appstream_dir(Some(std::env::consts::ARCH)).unwrap();
        appstream_file.push(appstream_dir.path().unwrap());
        appstream_file.push("appstream.xml");

        println!("Parsing appstream xml {:?}", appstream_file);
        let appdata_collection = match Collection::from_path(appstream_file.clone()) {
            Ok(collection) => {
                println!(
                    "Successfully parsed appstream XML for remote {}",
                    remote_name
                );
                Some(collection)
            }
            Err(err) => {
                eprintln!(
                    "Unable to parse appstream XML for {:?} : {}",
                    err.to_string(),
                    remote_name
                );
                None
            }
        };
        let mut db_packages = Vec::new();
        for remote_ref in &packages {
            let ref_name = remote_ref.format_ref().unwrap().to_string();
            println!("Found package {}", ref_name);
            if remote_ref.arch().unwrap().to_string() != std::env::consts::ARCH {
                println!("Not the same arch");
            }
            let component: Option<Component> = match &appdata_collection {
                Some(collection) => {
                    let app_id = AppId(remote_ref.name().unwrap().to_string());
                    let components = collection.find_by_id(app_id);
                    components
                        .into_iter()
                        .find(|c| get_ref_name(c) == ref_name)
                        .cloned()
                }
                None => {
                    println!("Unable to find appstream data for {}", ref_name);
                    None
                }
            };
            println!("{:?}", component);
            let package = Package {
                name: remote_ref.name().unwrap().to_string(),
                pretty_name: Some(ref_name),
                kind: PackageKind::from(remote_ref.kind()),
                icon_path: None,
                description: Some("".into()),
            };
            db_packages.push(package);
        }
    }
}

fn get_ref_name(component: &Component) -> String {
    for bundle in &component.bundles {
        match bundle {
            Bundle::Flatpak {
                runtime: _,
                sdk: _,
                reference,
            } => return reference.clone(),
            _ => (),
        }
    }
    String::new()
}
