use log::error;
use rusqlite::Connection;
use rusqlite::named_params;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplicationConfiguration {
    pub exec: String,
    pub terminal: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomCommandConfiguration {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    #[serde(default)]
    pub accepts_arguments: bool,
    #[serde(default)]
    pub tty: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Actionable {
    Application(ApplicationConfiguration),
    CustomCommand(CustomCommandConfiguration),
}

#[derive(Debug)]
pub struct Entry {
    pub id: u16,
    pub name: String,
    pub description: Option<String>,
    pub actionable: Option<Actionable>,
}

impl Entry {
    pub fn select(conn: &Connection, input: String) -> Option<Vec<Entry>> {
        let mut statement = conn
            .prepare(
                "
                SELECT id, name, description, actionable
                FROM entries 
                WHERE 
                    LOWER(name) like LOWER(:query) OR
                    LOWER(description) like LOWER(:query)
                ",
            )
            .unwrap();
        let results =
            statement.query_map(named_params! {":query": &format!("%{}%", input)}, |row| {
                let id = row.get(0).unwrap();
                let name = row.get(1).unwrap();
                let description = row.get(2).ok();
                let actionable: Option<Actionable> = row
                    .get::<_, String>(3)
                    .map(|json| serde_json::from_str(&json).unwrap())
                    .ok();

                Ok(Entry {
                    id,
                    name,
                    description,
                    actionable,
                })
            });
        match results {
            Ok(rows) => {
                let mut array: Vec<Entry> = Vec::new();

                for row in rows {
                    if let Ok(entry) = row {
                        array.push(entry);
                    }
                }

                Some(array)
            }
            Err(error) => {
                error!("SQL Error {error:?}");
                None
            }
        }
    }
    pub fn insert(&self, conn: &Connection) -> bool {
        let mut statement = conn.prepare("INSERT INTO entries (name, description, actionable) VALUES (:name, :description, :actionable)").unwrap();

        let actionable_string = serde_json::to_string(&self.actionable).unwrap();

        let result = statement.execute(named_params! {
            ":name": &self.name,
            ":description": &self.description,
            ":actionable": &actionable_string,
        });

        match result {
            Ok(_) => true,
            Err(error) => {
                error!("Error inserting {error:?}");
                false
            }
        }
    }
}
