use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rust_fuzzy_search::fuzzy_search_best_n;

use crate::backend::flatpak_backend::{Package, PackageKind};

use super::Storage;

pub fn search(db: Arc<Mutex<Storage>>, st: &str) -> Vec<Package> {
    let mut res = vec![];
    db.lock().unwrap().all_packages.as_ref().map(|pkgs| {
        let found = fuzzy_search_best_n(st, &pkgs, 10);
        found
            .iter()
            .for_each(|(name, _score)| res.push(name.to_string()));
    });
    let mut final_result = vec![];
    if let Ok(mut stmt) = db.lock().unwrap().conn.prepare(
        "SELECT name, prettyname, summary, iconpath, desc, kind FROM packages WHERE name = :name",
    ) {
        for name in res {
            let _ = stmt
                .query_map(&[(":name", name.as_str())], |row| {
                    let kind = PackageKind::from(row.get::<usize, String>(5).unwrap());
                    if kind == PackageKind::App {
                        let name: String = row.get(0).unwrap();
                        let pretty_name: Option<String> = row.get(1).unwrap();
                        let summary: Option<String> = row.get(2).unwrap();
                        let icon_path: Option<PathBuf> = row
                            .get::<usize, Option<String>>(3)
                            .unwrap()
                            .map(|s| PathBuf::from(s));
                        let description: Option<String> = row.get(4).unwrap();
                        final_result.push(Package::new(
                            name,
                            pretty_name,
                            description,
                            summary,
                            icon_path,
                            kind,
                        ));
                    }
                    Ok(())
                })
                .unwrap()
                .collect::<Vec<_>>();
        }
    }
    return final_result;
}

pub fn get_staff_picks(db: Arc<Mutex<Storage>>) -> Vec<Package> {
    let staff_picks_names = vec![
        "org.blender.Blender",
        "com.logseq.Logseq",
        "com.mattermost.Desktop",
        "im.riot.Riot",
        "com.github.wwmm.easyeffects",
    ];
    let mut final_result = vec![];
    if let Ok(mut stmt) = db.lock().unwrap().conn.prepare(
        "SELECT name, prettyname, summary, iconpath, desc, kind FROM packages WHERE name = :name",
    ) {
        for name in staff_picks_names {
            let _ = stmt
                .query_map(&[(":name", name)], |row| {
                    let kind = PackageKind::from(row.get::<usize, String>(5).unwrap());
                    if kind == PackageKind::App {
                        let name: String = row.get(0).unwrap();
                        let pretty_name: Option<String> = row.get(1).unwrap();
                        let summary: Option<String> = row.get(2).unwrap();
                        let icon_path: Option<PathBuf> = row
                            .get::<usize, Option<String>>(3)
                            .unwrap()
                            .map(|s| PathBuf::from(s));
                        let description: Option<String> = row.get(4).unwrap();
                        final_result.push(Package::new(
                            name,
                            pretty_name,
                            description,
                            summary,
                            icon_path,
                            kind,
                        ));
                    }
                    Ok(())
                })
                .unwrap()
                .collect::<Vec<_>>();
        }
    }
    return final_result;
}
