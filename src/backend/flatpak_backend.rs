use appstream::{enums::Bundle, AppId, Collection, Component};
use libflatpak::{
    gio::{traits::FileExt, Cancellable},
    glib::GString,
    prelude::*,
    traits::{InstallationExt, InstalledRefExt, RefExt, RemoteExt, RemoteRefExt},
    Installation, InstalledRef, RefKind, RemoteRef, Transaction,
};

use std::path::PathBuf;

use crate::db::Storage;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum PackageKind {
    #[default]
    App,
    Runtime,
    Extension,
}

impl PackageKind {
    pub fn to_string(&self) -> String {
        match self {
            PackageKind::App => "App".into(),
            PackageKind::Runtime => "Runtime".into(),
            PackageKind::Extension => "Extension".into(),
        }
    }
}

impl From<String> for PackageKind {
    fn from(value: String) -> Self {
        match value.as_str() {
            "App" => Self::App,
            "Runtime" => Self::Runtime,
            "Extension" => Self::Extension,
            _ => unreachable!(),
        }
    }
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

#[derive(Debug, Default, Clone)]
pub struct Package {
    pub name: PackageId,
    pub pretty_name: Option<String>,
    pub description: Option<String>,
    pub summary: Option<String>,
    pub icon_path: Option<PathBuf>,
    pub kind: PackageKind,
}

impl From<InstalledRef> for Package {
    fn from(pkg: InstalledRef) -> Self {
        let pretty_name = pkg.appdata_name().map(|s| s.to_string());
        let remote: String = pkg.origin().unwrap().to_string();
        let name: String = pkg.name().unwrap().to_string();
        let arch = pkg.arch().unwrap().to_string();
        let icon_path = get_icon_path(&name, &remote, &arch);
        let kind = PackageKind::from(pkg.kind());
        let summary = pkg.appdata_summary().map(|s| s.to_string());
        let description = None;

        Package {
            name,
            pretty_name,
            description,
            summary,
            icon_path,
            kind,
        }
    }
}

impl From<RemoteRef> for Package {
    fn from(pkg: RemoteRef) -> Self {
        let pretty_name = pkg.name().map(|s| s.to_string());
        let remote: String = pkg.remote_name().unwrap().to_string();
        let name: String = pkg.name().unwrap().to_string();
        let arch = pkg.arch().unwrap().to_string();
        let kind = PackageKind::from(pkg.kind());
        let description = None;
        let summary = None;
        let icon_path = get_icon_path(&name, &remote, &arch);
        Package {
            name,
            pretty_name,
            description,
            summary,
            icon_path,
            kind,
        }
    }
}

impl Package {
    pub fn new(
        name: String,
        pretty_name: Option<String>,
        description: Option<String>,
        summary: Option<String>,
        icon_path: Option<PathBuf>,
        kind: PackageKind,
    ) -> Self {
        Self {
            name,
            pretty_name,
            description,
            summary,
            icon_path,
            kind,
        }
    }

    pub fn with_description(mut self, desc: Option<String>) -> Self {
        self.description = desc;
        self
    }

    pub fn with_summary(mut self, summary: Option<String>) -> Self {
        self.summary = summary;
        self
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
    for pkg in installed_user {
        if pkg.kind() == RefKind::App {
            result.push(Package::from(pkg));
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

pub fn get_remote_ref_by_name(name: &str) -> Option<RemoteRef> {
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
            // println!("Ref: {}", pkg.name().unwrap().to_string());
            if pkg.name().unwrap().to_string().as_str() == name {
                return Some(pkg.to_owned());
            }
        }
    }
    None
}

pub fn get_packages_remote(storage: &Storage) {
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
            // println!("Ref: {}", pkg.name().unwrap().to_string());
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
            // println!("Found package {}", ref_name);
            let arch = remote_ref.arch().unwrap().to_string();
            if arch != std::env::consts::ARCH {
                println!("Not the same arch");
                continue;
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
            let name = remote_ref.name().unwrap().to_string();
            let (desc, summary, pretty_name) = component.map_or((None, None, None), |c| {
                (
                    c.description
                        .and_then(|d| d.get_default().map(String::to_owned)),
                    c.summary
                        .and_then(|d| d.get_default().map(String::to_owned)),
                    c.name.get_default().map(String::to_owned),
                )
            });
            let icon_path = get_icon_path(&name, &remote.to_string(), &arch);
            let package = Package {
                name,
                pretty_name,
                kind: PackageKind::from(remote_ref.kind()),
                icon_path,
                description: desc,
                summary,
            };
            db_packages.push(package);
        }
        storage.insert_batch(&db_packages).unwrap();
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

fn get_icon_path(name: &str, remote: &str, arch: &str) -> Option<PathBuf> {
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
            for size in [
                ("64", "png"),
                ("128", "png"),
                ("32", "png"),
                ("256", "png"),
                ("16", "png"),
                ("scalable", "svg"),
            ] {
                let path_str = format!(
                            "{}/.local/share/flatpak/app/{}/current/active/export/share/icons/hicolor/{}x{}/apps/{}.{}",
                            env!("HOME"),
                            name,
                            size.0,
                            size.0,
                            name,
                            size.1,
                        );
                path.push(path_str);
                if path.exists() {
                    break;
                }
            }
            path
        };
    }
    if path.exists() {
        return Some(path);
    }
    None
}
