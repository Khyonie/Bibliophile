use std::fs;

use gtk4::glib::home_dir;
use rusqlite::{Connection, Error};

use crate::tag::Tag;

const DATABASE_HOME: &str = ".local/share/bibliophile/";
const DATABASE_PATH: &str = "bibliophile.db";

pub fn open() -> Result<Connection, Error> {
    let path = home_dir().join(DATABASE_HOME);
    let db_path = path.join(DATABASE_PATH);
    let exists = db_path.exists();

    // Check if the db parent folder exists
    if !path.exists() {
        fs::create_dir_all(path).map_err(|_| Error::InvalidPath(db_path.clone()))?
    }

    // If the database file doesn't exist, initialize it
    let connection = Connection::open(&db_path)?;
    if !exists {
        connection.execute("PRAGMA foreign_keys = ON;", ())?;
        initialize_db(&connection)?;
    }

    Ok(connection)
}

fn initialize_db(connection: &Connection) -> Result<(), Error> {
    connection.execute(
        r#"
        CREATE TABLE files (
            id INTEGER PRIMARY KEY,
            path BLOB NOT NULL UNIQUE
        );
    "#,
        (),
    )?;

    connection.execute(
        r#"
        CREATE TABLE tags (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            color TEXT
        );
    "#,
        (),
    )?;
    connection.execute(
        r#"
        CREATE TABLE file_tags (
            file_id INTEGER NOT NULL
                REFERENCES files(id) ON DELETE CASCADE,
            tag_id INTEGER NOT NULL
                REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (file_id, tag_id)
        );
    "#,
        (),
    )?;

    Tag::new_tag("Invoices", "#FF0000", connection)?;
    Tag::new_tag("Fish", "#00FFFF", connection)?;
    Tag::new_tag("Drugs", "#FFFFFF", connection)?;
    Tag::new_tag("Yellow", "#FFFF00", connection)?;
    Tag::new_tag("Green", "#0000FF", connection)?;

    println!("Initialized database");
    Ok(())
}
