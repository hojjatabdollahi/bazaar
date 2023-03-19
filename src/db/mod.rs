use rusqlite::{Connection, Statement};

use crate::backend::flatpak_backend::Package;

#[derive(Debug)]
pub struct Storage {
    pub conn: Connection,
}

#[derive(Debug)]
pub struct Person {
    pub id: i32,
    pub name: String,
    pub data: Option<Vec<u8>>,
}

impl Storage {
    pub fn new() -> rusqlite::Result<Self> {
        let path = "./apps.db3";
        let conn = Connection::open(path)?;
        // Use the database somehow...
        println!("{}", conn.is_autocommit());
        Ok(Self { conn })
    }
    pub fn create_table(&mut self) -> rusqlite::Result<usize> {
        Ok(self.conn.execute(
            "CREATE TABLE if not exists packages (
                id     INTEGER PRIMARY KEY,
                name   TEXT NOT NULL,
                desc   TEXT
                )",
            (),
        )?)
    }
    pub fn insert(&self, package: &Package) -> rusqlite::Result<usize> {
        Ok(self.conn.execute(
            "INSERT INTO packages (name, desc) VALUES (?1, ?2)",
            (&package.name, &package.description),
        )?)
    }
    pub fn insert_batch(&self, packages: &Vec<Package>) -> rusqlite::Result<()> {
        self.conn.execute("begin", ())?;

        for pkg in packages {
            self.conn.execute(
                "INSERT INTO packages (name, desc) VALUES (?1, ?2)",
                (&pkg.name, &pkg.description),
            )?;
        }

        self.conn.execute("end", ())?;
        Ok(())
    }

    pub fn find(&self) -> rusqlite::Result<()> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, desc FROM packages WHERE name = :name")?;
        let pkg_iter =
            stmt.query_map(&[(":name", "us.zoom.Zoom".to_string().as_str())], |row| {
                Ok(Package {
                    name: row.get(0)?,
                    description: row.get(1)?,
                    ..Package::default()
                })
            })?;

        for pkg in pkg_iter {
            println!("Found package {:?}", pkg.unwrap());
        }
        Ok(())
    }
}
