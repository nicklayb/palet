use std::fs;
use std::path::Path;
use evalexpr::eval;

#[derive(Debug, Clone)]
pub struct Application {
    pub name: String,
    pub exec: String,
    pub description: Option<String>,
}

pub fn scan_applications() -> Vec<Application> {
    let mut apps = Vec::new();
    
    // Scan multiple application directories that drun uses (NixOS paths)
    let app_dirs = vec![
        "/run/current-system/sw/share/applications".to_string(),
        "/usr/share/applications".to_string(),
        "/usr/local/share/applications".to_string(),
        "/var/lib/flatpak/exports/share/applications".to_string(),
        format!("{}/.local/share/applications", dirs::home_dir().unwrap_or_default().display()),
        format!("{}/.local/share/flatpak/exports/share/applications", dirs::home_dir().unwrap_or_default().display()),
        format!("{}/.nix-profile/share/applications", dirs::home_dir().unwrap_or_default().display()),
    ];
    
    for app_dir in app_dirs {
        println!("Scanning directory: {}", app_dir);
        if let Ok(entries) = fs::read_dir(&app_dir) {
            let mut count = 0;
            let mut total_files = 0;
            for entry in entries.flatten() {
                total_files += 1;
                let path = entry.path();
                println!("  Checking file: {}", path.display());
                if let Some(app) = parse_desktop_file(&path) {
                    apps.push(app);
                    count += 1;
                } else {
                    println!("    -> Skipped (not a valid app)");
                }
            }
            println!("Found {} applications out of {} files in {}", count, total_files, app_dir);
        } else {
            println!("Could not read directory: {}", app_dir);
        }
    }
    
    println!("Total applications found: {}", apps.len());
    
    // Sort by name
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

fn parse_desktop_file(path: &Path) -> Option<Application> {
    if !path.extension().map_or(false, |ext| ext == "desktop") {
        println!("    -> Not a .desktop file");
        return None;
    }
    
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut description = None;
    let mut hidden = false;
    let mut no_display = false;
    let mut app_type = None;
    let mut in_desktop_entry = false;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        
        // Check for section headers
        if line.starts_with('[') && line.ends_with(']') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        
        // Only parse if we're in the [Desktop Entry] section
        if !in_desktop_entry {
            continue;
        }
        
        if line.starts_with("Name=") && !line.contains('[') {
            // Use the default name (without locale)
            name = Some(line[5..].to_string());
        } else if line.starts_with("Exec=") {
            exec = Some(line[5..].to_string());
        } else if line.starts_with("Comment=") && !line.contains('[') {
            description = Some(line[8..].to_string());
        } else if line.starts_with("Hidden=") {
            hidden = line[7..].trim().to_lowercase() == "true";
        } else if line.starts_with("NoDisplay=") {
            no_display = line[10..].trim().to_lowercase() == "true";
        } else if line.starts_with("Type=") {
            app_type = Some(line[5..].to_string());
        }
    }
    
    // Only include Application type entries
    if app_type.as_deref() != Some("Application") {
        println!("    -> Not an Application type (type: {:?})", app_type);
        return None;
    }
    
    if hidden || no_display {
        println!("    -> Hidden or NoDisplay (hidden: {}, no_display: {})", hidden, no_display);
        return None;
    }
    
    let name = if let Some(n) = name {
        n
    } else {
        println!("    -> No Name field found");
        return None;
    };
    
    let mut exec = if let Some(e) = exec {
        e
    } else {
        println!("    -> No Exec field found");
        return None;
    };
    
    // Clean up exec command (remove field codes like %f, %u, %F, %U, etc.)
    let parts: Vec<&str> = exec.split_whitespace().collect();
    let mut cleaned_parts = Vec::new();
    
    for part in parts {
        if part.starts_with('%') {
            break; // Stop at first field code
        }
        cleaned_parts.push(part);
    }
    
    if cleaned_parts.is_empty() {
        return None;
    }
    
    exec = cleaned_parts.join(" ");
    
    println!("Parsed app: {} -> {}", name, exec);
    
    Some(Application {
        name,
        exec,
        description,
    })
}

/// Represents the type of item in the application list
#[derive(Debug, Clone)]
pub enum ListItem {
    Application(Application),
    CustomCommand { 
        command: crate::config::CustomCommand, 
        arguments: Option<String> 
    },
    Calculator { expression: String, result: String },
    SearchFallback(String), // Contains the search query
}

impl ListItem {
    pub fn display_name(&self) -> String {
        match self {
            ListItem::Application(app) => app.name.clone(),
            ListItem::CustomCommand { command, arguments } => {
                if let Some(args) = arguments {
                    format!("{} {}", command.name, args)
                } else {
                    command.name.clone()
                }
            },
            ListItem::Calculator { expression, result } => format!("{} = {}", expression, result),
            ListItem::SearchFallback(_) => "Search".to_string(),
        }
    }
    
    pub fn description(&self) -> Option<String> {
        match self {
            ListItem::Application(app) => app.description.clone(),
            ListItem::CustomCommand { command, arguments } => {
                if arguments.is_some() {
                    Some(format!("{} (with arguments)", command.description.as_deref().unwrap_or("Custom command")))
                } else {
                    command.description.clone()
                }
            },
            ListItem::Calculator { .. } => Some("Copy result to clipboard".to_string()),
            ListItem::SearchFallback(query) => Some(format!("Search the web for '{}'", query)),
        }
    }
}

/// Attempts to evaluate an arithmetic expression
/// 
/// # Arguments
/// * `expression` - The expression to evaluate
/// 
/// # Returns
/// Result string if evaluation succeeds, None if invalid
fn try_evaluate_expression(expression: &str) -> Option<String> {
    // Check if it looks like a math expression (contains arithmetic operators)
    let has_math_chars = expression.chars().any(|c| "+-*/()^%".contains(c));
    if !has_math_chars {
        return None;
    }
    
    // Try to evaluate the expression
    match eval(expression) {
        Ok(result) => {
            // Format the result nicely
            match result {
                evalexpr::Value::Float(f) => {
                    if f.fract() == 0.0 {
                        Some(format!("{}", f as i64))
                    } else {
                        Some(format!("{:.10}", f).trim_end_matches('0').trim_end_matches('.').to_string())
                    }
                }
                evalexpr::Value::Int(i) => Some(format!("{}", i)),
                _ => Some(format!("{}", result)),
            }
        }
        Err(_) => None,
    }
}

pub fn filter_applications(apps: &[Application], custom_commands: &std::collections::HashMap<String, crate::config::CustomCommand>, query: &str) -> Vec<ListItem> {
    // Return empty list when no query
    if query.trim().is_empty() {
        return Vec::new();
    }
    
    let mut results: Vec<ListItem> = Vec::new();
    
    // Check if query is a math expression first
    if let Some(result) = try_evaluate_expression(query) {
        results.push(ListItem::Calculator {
            expression: query.to_string(),
            result,
        });
    }
    
    let query_lower = query.to_lowercase();
    
    // Filter custom commands (including argument-accepting ones)
    let mut custom_results: Vec<ListItem> = Vec::new();
    
    for cmd in custom_commands.values() {
        let cmd_name_lower = cmd.name.to_lowercase();
        
        // Check if command accepts arguments and query starts with command name
        if cmd.accepts_arguments && query_lower.starts_with(&cmd_name_lower) {
            // Check if there's a space after the command name
            if query.len() > cmd.name.len() && query.chars().nth(cmd.name.len()) == Some(' ') {
                // Extract arguments (everything after the command name and space)
                let arguments = query[cmd.name.len() + 1..].to_string();
                if !arguments.trim().is_empty() {
                    custom_results.push(ListItem::CustomCommand {
                        command: cmd.clone(),
                        arguments: Some(arguments),
                    });
                }
            } else if query_lower == cmd_name_lower {
                // Exact match without arguments
                custom_results.push(ListItem::CustomCommand {
                    command: cmd.clone(),
                    arguments: None,
                });
            }
        } else if !cmd.accepts_arguments {
            // Regular filtering for non-argument commands
            if cmd.name.to_lowercase().contains(&query_lower) ||
               cmd.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query_lower)) {
                custom_results.push(ListItem::CustomCommand {
                    command: cmd.clone(),
                    arguments: None,
                });
            }
        }
    }
    
    results.extend(custom_results);
    
    // Filter applications
    let app_results: Vec<ListItem> = apps.iter()
        .filter(|app| {
            app.name.to_lowercase().contains(&query_lower) ||
            app.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&query_lower))
        })
        .map(|app| ListItem::Application(app.clone()))
        .collect();
    
    results.extend(app_results);
    
    // If no results found (no calculator, no custom commands, and no apps), add search fallback
    if results.is_empty() {
        results.push(ListItem::SearchFallback(query.to_string()));
    }
    
    results
}