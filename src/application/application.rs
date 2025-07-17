use crate::application::parser;
use crate::config::{Config, CustomCommands};
use crate::queryable::Queryable;
use evalexpr;
use std::fs;

#[derive(Debug, Clone)]
pub struct Application {
    pub name: String,
    pub exec: String,
    pub description: Option<String>,
}

pub fn scan_applications(config: &Config) -> Vec<Application> {
    let mut apps = Vec::new();
    let mut app_dirs = default_application_folders();
    app_dirs.extend(config.extra_paths.clone());

    for app_dir in app_dirs {
        if let Ok(entries) = fs::read_dir(&app_dir) {
            let mut total_files = 0;
            for entry in entries.flatten() {
                total_files += 1;
                let path = entry.path();
                if let Some(app) = parser::parse_desktop_file(&path) {
                    apps.push(app);
                }
            }
            println!("[INFO] {} Found {} / {}", app_dir, apps.len(), total_files);
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn default_application_folders() -> Vec<String> {
    vec![
        "/run/current-system/sw/share/applications".to_string(),
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
        "/var/lib/flatpak/exports/share/applications".to_string(),
        format!(
            "{}/.local/share/applications",
            dirs::home_dir().unwrap_or_default().display()
        ),
        format!(
            "{}/.local/share/flatpak/exports/share/applications",
            dirs::home_dir().unwrap_or_default().display()
        ),
        format!(
            "{}/.nix-profile/share/applications",
            dirs::home_dir().unwrap_or_default().display()
        ),
    ]
}

/// Represents the type of item in the application list

/// Attempts to evaluate an arithmetic expression
///
/// # Arguments
/// * `expression` - The expression to evaluate
///
/// # Returns
/// Result string if evaluation succeeds, None if invalid
fn try_evaluate_expression(expression: &str) -> Option<String> {
    let has_math_chars = expression.chars().any(|c| "+-*/()^%".contains(c));
    if !has_math_chars {
        return None;
    }

    match evalexpr::eval(expression) {
        Ok(result) => format_expression_result(result),
        Err(_) => None,
    }
}

fn format_expression_result(result: evalexpr::Value) -> Option<String> {
    match result {
        evalexpr::Value::Float(f) => {
            if f.fract() == 0.0 {
                Some(format!("{}", f as i64))
            } else {
                Some(
                    format!("{:.10}", f)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string(),
                )
            }
        }
        evalexpr::Value::Int(i) => Some(format!("{}", i)),
        _ => Some(format!("{}", result)),
    }
}

pub fn filter_applications(
    apps: &[Application],
    custom_commands: &CustomCommands,
    query: &str,
) -> Vec<Queryable> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    let mut results: Vec<Queryable> = Vec::new();

    if let Some(result) = try_evaluate_expression(query) {
        results.push(Queryable::Calculator {
            expression: query.to_string(),
            result,
        });
        return results;
    }

    let query_lower = query.to_lowercase();

    let custom_results = build_custom_commands(query_lower.clone(), custom_commands);

    results.extend(custom_results);

    let app_results: Vec<Queryable> = apps
        .iter()
        .filter(|app| {
            app.name.to_lowercase().contains(&query_lower)
                || app
                    .description
                    .as_ref()
                    .map_or(false, |desc| desc.to_lowercase().contains(&query_lower))
        })
        .map(|app| Queryable::Application(app.clone()))
        .collect();

    results.extend(app_results);

    if results.is_empty() {
        results.push(Queryable::SearchFallback(query.to_string()));
    }

    results
}

fn build_custom_commands(query: String, custom_commands: &CustomCommands) -> Vec<Queryable> {
    let mut custom_results: Vec<Queryable> = Vec::new();

    for command in custom_commands.values() {
        let command_name_lower = command.name.to_lowercase();

        if command.accepts_arguments && query.starts_with(&command_name_lower) {
            let arguments = extract_command_arguments(command.name.clone(), query.clone());
            custom_results.push(Queryable::CustomCommand {
                command: command.clone(),
                arguments,
            });
        } else if !command.accepts_arguments {
            if command.name.to_lowercase().contains(&query)
                || command
                    .description
                    .as_ref()
                    .map_or(false, |desc| desc.to_lowercase().contains(&query))
            {
                custom_results.push(Queryable::CustomCommand {
                    command: command.clone(),
                    arguments: None,
                });
            }
        }
    }

    custom_results
}

fn extract_command_arguments(command_name: String, query: String) -> Option<String> {
    if query.len() > command_name.len() {
        if query.chars().nth(command_name.len()) == Some(' ') {
            let arguments = query[command_name.len() + 1..].to_string();
            if !arguments.trim().is_empty() {
                return Some(arguments);
            }
        }
    }
    None
}
