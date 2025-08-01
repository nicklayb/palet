use log::error;
use rusqlite::{Connection, named_params};

pub trait Schema {
    fn table() -> String;
    fn fields() -> Vec<Field>;
    fn primary_key() -> (Field, bool);
}

pub struct Field {
    pub name: String,
    pub nullable: bool,
    pub field_type: String,
}

impl Field {
    pub fn to_insert_statement(&self) -> String {
        let nullable = if self.nullable { "" } else { "NOT NULL" };
        format!("{} {} {}", self.name, self.field_type, nullable)
    }
}

pub fn create_table<T: Schema>(conn: &Connection) {
    let lines = T::fields()
        .iter()
        .map(|field| field.to_insert_statement())
        .collect::<Vec<String>>()
        .join(", ");
    let (primary_key, _) = T::primary_key();

    let statement = format!(
        "CREATE TABLE IF NOT EXISTS {} ({}, {})",
        T::table(),
        primary_key.to_insert_statement(),
        lines
    );

    if let Err(error) = conn.execute(&statement, ()) {
        let table = T::table();
        error!("Error while creating {table}: {error:?}");
    };
}

pub fn drop_table<T: Schema>(conn: &Connection) {
    let statement = format!("DROP TABLE IF EXISTS {}", T::table());
    if let Err(error) = conn.execute(&statement, ()) {
        let table = T::table();
        error!("Error while dropping {table}: {error:?}");
    };
}
