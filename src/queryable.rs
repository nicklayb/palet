use log::debug;

use crate::{
    application::Application,
    config::{Config, CustomCommand, SearchUrl},
};

#[derive(Debug, Clone)]
pub enum Queryable {
    Application(Application),
    CustomCommand {
        command: crate::config::CustomCommand,
        arguments: Option<String>,
    },
    Calculator {
        expression: String,
        result: String,
    },
    SearchFallback {
        search_url: SearchUrl,
        query: String,
    },
}

impl Queryable {
    pub fn display_name(&self) -> String {
        match self {
            Queryable::Application(app) => app.name.clone(),
            Queryable::CustomCommand { command, arguments } => {
                if let Some(args) = arguments {
                    format!("{} {}", command.name, args)
                } else {
                    command.name.clone()
                }
            }
            Queryable::Calculator { expression, result } => format!("{} = {}", expression, result),
            Queryable::SearchFallback {
                search_url: SearchUrl { name, .. },
                ..
            } => format!("Search {}", name),
        }
    }

    pub fn description(&self) -> Option<String> {
        match self {
            Queryable::Application(app) => app.description.clone(),
            Queryable::CustomCommand { command, arguments } => {
                if arguments.is_some() {
                    Some(format!(
                        "{} (with arguments)",
                        command.description.as_deref().unwrap_or("Custom command")
                    ))
                } else {
                    command.description.clone()
                }
            }
            Queryable::Calculator { .. } => Some("Copy result to clipboard".to_string()),
            Queryable::SearchFallback {
                search_url: SearchUrl { name, .. },
                query,
            } => Some(format!("Search '{}' on {}", query, name)),
        }
    }

    pub fn action(&self, config: &Config) {
        match self {
            Queryable::Application(app) => {
                launch_application(app, &config.terminal);
            }
            Queryable::CustomCommand { command, arguments } => {
                execute_custom_command(command, arguments.as_deref(), &config.terminal);
            }
            Queryable::Calculator { result, .. } => {
                copy_to_clipboard(result);
            }
            Queryable::SearchFallback { search_url, query } => {
                perform_web_search(query, search_url);
            }
        }
    }

    pub fn classes(&self) -> (&str, &str) {
        return match self {
            Queryable::Application(_) => ("app-name", "description"),
            Queryable::CustomCommand { .. } => ("custom-command", "description"),
            Queryable::Calculator { .. } => ("calculator-result", "description"),
            Queryable::SearchFallback { .. } => ("search-item", "description"),
        };
    }
}

fn launch_application(app: &Application, terminal: &str) {
    if app.terminal {
        let command_name = format!("{} {}", terminal, app.exec);
        spawn_shell(&command_name, vec![]);
    } else {
        spawn_shell(&app.exec, vec![]);
    }
}

/// Performs a web search using the configured search URL
///
/// # Arguments
/// * `query` - The search query
/// * `search_url_template` - URL template with {q} placeholder
fn perform_web_search(query: &str, search_url: &SearchUrl) {
    let search_url = search_url.build(query);
    spawn("xdg-open", vec![&search_url]);
}

/// Copies text to the system clipboard
///
/// # Arguments
/// * `text` - The text to copy
fn copy_to_clipboard(text: &str) {
    spawn_shell(
        &format!("echo -n '{}' | xclip -selection clipboard", text),
        vec![],
    );
}

/// Executes a custom command with optional arguments
///
/// # Arguments
/// * `cmd` - The custom command to execute
/// * `arguments` - Optional arguments to pass to the command
/// * `terminal` - Terminal command to use when tty is true
fn execute_custom_command(cmd: &CustomCommand, arguments: Option<&str>, terminal: &str) {
    let command_to_run = if let Some(args) = arguments {
        format!("{} {}", cmd.command, args)
    } else {
        cmd.command.clone()
    };

    let final_command = if cmd.tty {
        format!("{} {}", terminal, command_to_run)
    } else {
        command_to_run.clone()
    };

    spawn_shell(&final_command, vec![]);
}

fn spawn_shell(command: &str, arguments: Vec<&str>) {
    let mut spawning_arguments = vec!["-c", command];
    spawning_arguments.extend(arguments);
    spawn("sh", spawning_arguments)
}

fn spawn(command_name: &str, arguments: Vec<&str>) {
    debug!("Spawing {command_name} {arguments:?}");
    let _ = std::process::Command::new(command_name)
        .args(&arguments)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}
