use log::error;
use rusqlite::{Connection, named_params};

pub trait DatabaseSchema {
    fn schema() -> Schema;
}

pub struct Schema {
    pub table: String,
    pub fields: Vec<Field>,
    pub primary_key: Field,
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

impl Schema {
    pub fn create_table(&self, conn: &Connection) {
        let lines = self
            .fields
            .iter()
            .map(|field| field.to_insert_statement())
            .collect::<Vec<String>>()
            .join(", ");

        let statement = format!(
            "CREATE TABLE IF NOT EXISTS {} ({}, {})",
            self.table,
            self.primary_key.to_insert_statement(),
            lines
        );

        if let Err(error) = conn.execute(&statement, ()) {
            let table = &self.table;
            error!("Error while creating {table}: {error:?}");
        };
    }

    pub fn drop_table(&self, conn: &Connection) {
        let statement = format!("DROP TABLE IF EXISTS {}", self.table);
        if let Err(error) = conn.execute(&statement, ()) {
            let table = &self.table;
            error!("Error while dropping {table}: {error:?}");
        };
    }
}
