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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchUrl {
    pub name: String,
    pub url: String,
}

pub type SearchUrls = HashMap<String, SearchUrl>;

pub type CustomCommands = HashMap<String, CustomCommand>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_width")]
    pub width: i32,
    #[serde(default = "default_height")]
    pub height: i32,
    #[serde(default = "default_placeholder")]
    pub placeholder: String,
    #[serde(default = "default_search_urls")]
    pub search_urls: SearchUrls,
    #[serde(default = "default_terminal")]
    pub terminal: String,
    #[serde(default)]
    pub custom_commands: CustomCommands,
    #[serde(default)]
    pub extra_paths: Vec<String>,
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,
}

fn default_db_path() -> PathBuf {
    dirs::config_dir()
        .map(|dir| dir.join("palet").join("index.db"))
        .unwrap()
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
fn default_search_urls() -> SearchUrls {
    HashMap::from([(
        "google".to_string(),
        SearchUrl {
            name: "Google".to_string(),
            url: "https://www.google.com/search?q={q}".to_string(),
        },
    )])
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
            search_urls: default_search_urls(),
            terminal: default_terminal(),
            db_path: default_db_path(),
            custom_commands: HashMap::new(),
            extra_paths: Vec::new(),
        }
    }
}

fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("palet").join("config.toml"))
}

fn create_config_folder() {
    let folder = dirs::config_dir().map(|dir| dir.join("palet")).unwrap();

    fs::create_dir(folder).unwrap();
}

pub fn load_config() -> Config {
    create_config_folder();

    if let Some(config_path) = get_config_path() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<Config>(&content) {
                return config;
            }
        }
    }
    Config::default()
}

impl SearchUrl {
    pub fn build(&self, query: &str) -> String {
        let encoded_query = urlencoding::encode(query);
        self.url.replace("{q}", &encoded_query)
    }
}
