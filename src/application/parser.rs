use std::fs;
use std::path::Path;

use crate::application::Application;

pub fn parse_desktop_file(path: &Path) -> Option<Application> {
    let mut string_name = "unknown";
    if let Some(string) = path.to_str() {
        string_name = string;
    }
    if !is_desktop_entry(path) {
        println!("[WARNING] {} invalid .desktop", string_name);

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
        println!(
            "[WARNING] {} Not an Application type (type: {:?})",
            string_name, app_type
        );
        return None;
    }

    if hidden || no_display {
        println!(
            "[WARNING] {} Hidden or NoDisplay (hidden: {}, no_display: {})",
            string_name, hidden, no_display
        );
        return None;
    }

    let name = if let Some(n) = name {
        n
    } else {
        println!("[WARNING] {} No Name field found", string_name);
        return None;
    };

    let mut exec = if let Some(e) = exec {
        e
    } else {
        println!("[WARNING] {} No Exec field found", string_name);
        return None;
    };

    // Clean up exec command (remove field codes like %f, %u, %F, %U, etc.)
    let parts: Vec<&str> = exec.split_whitespace().collect();
    let mut cleaned_parts = Vec::new();

    for part in parts {
        if part.starts_with('%') {
            break;
        }
        cleaned_parts.push(part);
    }

    if cleaned_parts.is_empty() {
        return None;
    }

    exec = cleaned_parts.join(" ");

    Some(Application {
        name,
        exec,
        description,
    })
}

fn is_desktop_entry(path: &Path) -> bool {
    path.extension().map_or(false, |ext| ext == "desktop")
}
