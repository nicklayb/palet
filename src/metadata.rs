use rusqlite::Connection;

use crate::schema::{Field, Schema};

type Version = String;

pub const VERSION: &str = "1";

#[derive(Debug)]
pub struct Metadata {
    pub name: String,
    pub value: String,
}

impl Schema for Metadata {
    fn table() -> String {
        "metadata".to_string()
    }
    fn primary_key() -> (Field, bool) {
        (
            Field {
                name: "name".to_string(),
                field_type: "VARCHAR".to_string(),
                nullable: false,
            },
            false,
        )
    }
    fn fields() -> Vec<Field> {
        vec![Field {
            name: "value".to_string(),
            field_type: "VARCHAR".to_string(),
            nullable: false,
        }]
    }
}

impl Metadata {
    pub fn current_version(conn: &Connection) -> Option<Metadata> {
        if let Ok(mut statement) = conn.prepare("SELECT value from metadata where name = :name") {
            if let Ok(version) =
                statement.query_one(&[(":name", "version")], |row| row.get::<_, Version>(0))
            {
                return Some(Metadata {
                    name: "version".to_string(),
                    value: version,
                });
            }
        }
        None
    }
    pub fn insert_version(conn: &Connection) {
        conn.execute(
            "INSERT INTO metadata (name, value) VALUES ('version', :version)",
            &[(":version", VERSION)],
        )
        .unwrap();
    }

    pub fn is_up_to_date(conn: &Connection) -> bool {
        if let Some(Metadata { value, .. }) = Self::current_version(conn) {
            return value == VERSION;
        }
        false
    }
}
