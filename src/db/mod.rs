pub mod search;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, Statement};

use crate::backend::flatpak_backend::Package;

#[derive(Debug)]
pub struct Storage {
    pub conn: Connection,
    pub all_packages: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub data: Option<Vec<u8>>,
}

impl Storage {
    pub fn new() -> rusqlite::Result<Self> {
        let path = PathBuf::from("./apps.db3");
        let conn = Connection::open(path)?;
        Ok(Self {
            conn,
            all_packages: None,
        })
    }
    pub fn create_table(&mut self) -> rusqlite::Result<usize> {
        Ok(self.conn.execute(
            "CREATE TABLE if not exists packages (
                id          INTEGER PRIMARY KEY,
                name        TEXT NOT NULL,
                prettyname  TEXT,
                summary     TEXT,
                iconpath    TEXT,
                desc        TEXT,
                kind        TEXT
                )",
            (),
        )?)
    }

    pub fn insert(&self, package: &Package) -> rusqlite::Result<usize> {
        Ok(self.conn.execute(
            "INSERT INTO packages (name, prettyname, summary, iconpath, desc, kind) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                &package.name,
                &package.pretty_name,
                &package.summary,
                &package
                    .icon_path
                    .clone()
                    .map(|p| p.to_str().unwrap().to_string()),
                &package.description,
                &package.kind.to_string(),
            ),
        )?)
    }
    pub fn insert_batch(&self, packages: &Vec<Package>) -> rusqlite::Result<()> {
        self.conn.execute("begin", ())?;
        for pkg in packages {
            self.conn.execute(
                "INSERT INTO packages (name, prettyname, summary, iconpath, desc, kind) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (
                    &pkg.name,
                    &pkg.pretty_name,
                    &pkg.summary,
                    &pkg.icon_path
                        .clone()
                        .map(|p| p.to_str().unwrap().to_string()),
                    &pkg.description,
                    &pkg.kind.to_string(),
                ),
            )?;
        }
        self.conn.execute("end", ())?;
        Ok(())
    }

    pub fn all_names(&self) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT name FROM packages")?;

        let pkgs = stmt
            .query_map([], |row| Ok(row.get::<usize, String>(0)?))?
            .map(|s| s.unwrap())
            .collect::<Vec<_>>();
        Ok(pkgs)
    }
}
