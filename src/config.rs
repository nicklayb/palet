use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CustomCommand {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    #[serde(default)]
    pub accepts_arguments: bool,
    #[serde(default)]
    pub tty: bool,
}

pub type CustomCommands = HashMap<String, CustomCommand>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_width")]
    pub width: i32,
    #[serde(default = "default_height")]
    pub height: i32,
    #[serde(default = "default_placeholder")]
    pub placeholder: String,
    #[serde(default = "default_search_url")]
    pub search_url: String,
    #[serde(default = "default_terminal")]
    pub terminal: String,
    #[serde(default)]
    pub custom_commands: CustomCommands,
    #[serde(default)]
    pub extra_paths: Vec<String>,
}

fn default_height() -> i32 {
    700
}
fn default_width() -> i32 {
    700
}

fn default_placeholder() -> String {
    "Search...".to_string()
}
fn default_search_url() -> String {
    "https://www.google.com/search?q={q}".to_string()
}
fn default_terminal() -> String {
    "alacritty -e".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            placeholder: default_placeholder(),
            search_url: default_search_url(),
            terminal: default_terminal(),
            custom_commands: HashMap::new(),
            extra_paths: Vec::new(),
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("palet").join("config.toml"))
}

pub fn load_config() -> Config {
    if let Some(config_path) = get_config_path() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<Config>(&content) {
                return config;
            }
        }
    }
    Config::default()
}
