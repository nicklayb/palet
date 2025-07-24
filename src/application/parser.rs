use std::path::Path;

use freedesktop_entry_parser::AttrSelector;
use log::warn;

use crate::application::Application;

pub fn parse_desktop_file(path: &Path) -> Option<Application> {
    let mut string_name = "unknown";
    if let Some(string) = path.to_str() {
        string_name = string;
    }
    if !is_desktop_entry(path) {
        warn!("{string_name} invalid .desktop");

        return None;
    }

    let entry = freedesktop_entry_parser::parse_entry(path).ok()?;
    let section = entry.section("Desktop Entry");
    ensure_application(string_name, &section)?;
    ensure_visible(string_name, &section)?;
    let name = extract_name(string_name, &section)?;
    let exec = extract_exec(string_name, &section)?;
    let terminal = extract_terminal(&section);
    let description = section.attr("Comment").map(|value| value.to_string());

    let app = Application {
        name,
        exec,
        description,
        terminal,
    };

    Some(app)
}

fn extract_terminal(section: &AttrSelector<&str>) -> bool {
    section.attr("Terminal").unwrap_or("false") == "true"
}

fn ensure_visible(string_name: &str, section: &AttrSelector<&str>) -> Option<()> {
    let hidden = section.attr("Hidden").unwrap_or("false") == "true";
    let no_display = section.attr("NoDisplay").unwrap_or("false") == "true";

    if hidden || no_display {
        warn!("{string_name} Hidden or NoDisplay (hidden: {hidden}, no_display: {no_display})");
        return None;
    }
    Some(())
}

fn extract_exec(string_name: &str, section: &AttrSelector<&str>) -> Option<String> {
    let exec = section.attr("Exec");

    clean_exec(string_name, exec).map_or_else(
        || {
            warn!("{string_name} Invalid exec");
            return None;
        },
        |item| Some(item.to_string()),
    )
}

fn extract_name(string_name: &str, section: &AttrSelector<&str>) -> Option<String> {
    section.attr("Name").map_or_else(
        || {
            warn!("{string_name} No Name field found");
            None
        },
        |item| Some(item.to_string()),
    )
}

fn ensure_application(string_name: &str, section: &AttrSelector<&str>) -> Option<String> {
    let app_type = section.attr("Type");
    if app_type.as_deref() != Some("Application") {
        warn!("{string_name} Not an Application type (type: {app_type:?})");
        return None;
    }
    app_type.map(|item| item.to_string())
}

// Clean up exec command (remove field codes like %f, %u, %F, %U, etc.)
fn clean_exec(string_name: &str, exec: Option<&str>) -> Option<String> {
    let exec = if let Some(inner_exec) = exec {
        inner_exec
    } else {
        warn!("{string_name} No Exec field found");
        return None;
    };
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

    Some(cleaned_parts.join(" ").to_string())
}

fn is_desktop_entry(path: &Path) -> bool {
    path.extension().map_or(false, |ext| ext == "desktop")
}
