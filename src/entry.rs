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

pub struct Entry {
    pub name: String,
    pub description: Option<String>,
    pub actionable: Option<Actionable>,
}
