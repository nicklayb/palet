use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub width: i32,
    pub height: i32,
    pub placeholder: String,
    pub search_url: String,
    #[serde(default = "default_terminal")]
    pub terminal: String,
    #[serde(default)]
    pub custom_commands: HashMap<String, CustomCommand>,
}

fn default_terminal() -> String {
    "alacritty -e".to_string()
}

impl Default for Config {
    fn default() -> Self {
        let mut custom_commands = HashMap::new();
        
        // Add some example custom commands
        custom_commands.insert("sleep".to_string(), CustomCommand {
            name: "Sleep".to_string(),
            command: "systemctl suspend".to_string(),
            description: Some("Put the system to sleep".to_string()),
            accepts_arguments: false,
            tty: false,
        });
        
        custom_commands.insert("lock".to_string(), CustomCommand {
            name: "Lock Screen".to_string(),
            command: "loginctl lock-session".to_string(),
            description: Some("Lock the current session".to_string()),
            accepts_arguments: false,
            tty: false,
        });
        
        custom_commands.insert("echo".to_string(), CustomCommand {
            name: "echo".to_string(),
            command: "echo".to_string(),
            description: Some("Echo the provided arguments".to_string()),
            accepts_arguments: true,
            tty: true,
        });
        
        Self {
            width: 700,
            height: 700,
            placeholder: "Type here...".to_string(),
            search_url: "https://www.google.com/search?q={q}".to_string(),
            terminal: default_terminal(),
            custom_commands,
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
