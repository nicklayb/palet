use std::path::PathBuf;

use log::{debug, error, info};
use rusqlite::Connection;

use crate::{entry::Entry, metadata::Metadata, schema};

pub fn initialize(file_name: &PathBuf) -> Option<Connection> {
    match Connection::open(file_name) {
        Ok(conn) => {
            if !Metadata::is_up_to_date(&conn) {
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
    info!("DROPPPING");
    schema::drop_table::<Entry>(conn);
    schema::drop_table::<Metadata>(conn);
}

fn initialize_database(conn: &Connection) {
    info!("INIT");
    schema::create_table::<Metadata>(conn);
    schema::create_table::<Entry>(conn);
    Metadata::insert_version(conn);
}
