use std::path::PathBuf;

use log::{debug, error, info};
use rusqlite::Connection;

type Version = String;

const VERSION: &str = "3";

pub fn initialize(file_name: &PathBuf) -> Option<Connection> {
    match Connection::open(file_name) {
        Ok(conn) => {
            if !database_exists(&conn) {
                info!("Creating database at {file_name:?}");
                drop_database(&conn);
                initialize_database(&conn);
            }

            return Some(conn);
        }
        Err(error) => {
            error!("Could not create database {error:?}")
        }
    }
    None
}

fn drop_database(conn: &Connection) {
    drop_metadata_table(conn);
    drop_entries_table(conn)
}

fn drop_entries_table(conn: &Connection) {
    conn.execute("DROP TABLE IF EXISTS entries", ()).unwrap();
}

fn drop_metadata_table(conn: &Connection) {
    conn.execute("DROP TABLE IF EXISTS metadata", ()).unwrap();
}

fn initialize_database(conn: &Connection) {
    create_metadata_table(conn);
    insert_version(conn);
    create_entries_table(conn);
}

fn insert_version(conn: &Connection) {
    conn.execute(
        "INSERT INTO metadata (name, value) VALUES ('version', :version)",
        &[(":version", VERSION)],
    )
    .unwrap();
}

fn create_entries_table(conn: &Connection) {
    match conn.execute(
        "
    CREATE TABLE IF NOT EXISTS entries (
        id INTEGER PRIMARY KEY,
        name VARCHAR NOT NULL
        description VARCHAR
        actionable TEXT
    )",
        (),
    ) {
        Ok(_) => {
            debug!("Entries table created");
        }
        Err(error) => {
            error!("Could not create entries table {error:?}");
        }
    }
}

fn create_metadata_table(conn: &Connection) {
    match conn.execute(
        "CREATE TABLE IF NOT EXISTS metadata (
        name VARCHAR PRIMARY KEY,
        value VARCHAR NOT NULL
    )",
        (),
    ) {
        Ok(_) => {
            debug!("Metadata table created");
        }
        Err(error) => {
            error!("Could not create metadata table {error:?}");
        }
    }
}

fn database_exists(conn: &Connection) -> bool {
    if let Some(version) = get_current_version(conn) {
        return version == VERSION;
    }
    false
}

fn get_current_version(conn: &Connection) -> Option<Version> {
    if let Ok(mut statement) = conn.prepare("SELECT value from metadata where name = :name") {
        if let Ok(version) =
            statement.query_one(&[(":name", "version")], |row| row.get::<_, Version>(0))
        {
            return Some(version);
        }
    }
    None
}
