use gtk4::ApplicationWindow;
use log::info;

/// Loads CSS from XDG config directory or uses default styles
///
/// # Returns
/// CSS content as a string
fn load_css_styles() -> String {
    // Try to load custom CSS from XDG config directory
    if let Some(config_dir) = dirs::config_dir() {
        let css_path = config_dir.join("palet").join("style.css");
        if let Ok(custom_css) = std::fs::read_to_string(&css_path) {
            let css_file = css_path.display();
            info!("Loaded custom CSS from: {css_file}");
            return custom_css;
        }
    }

    // Default CSS styles
    r#"
        /* Application launcher window */
        window {
            background-color: @theme_bg_color;
        }
        
        /* Text entry styling */
        .app-entry {
            margin: 12px;
            padding: 8px;
        }
        
        /* Application list */
        .app-list {
            margin: 0 12px 12px 12px;
        }
        
        /* Scrolled window */
        .app-scroll {
            margin: 0;
        }
        
        /* List item container */
        .item-container {
            margin: 8px;
            padding: 4px;
            spacing: 2px;
        }
        
        /* Application name label */
        .app-name {
            font-weight: bold;
        }
        
        /* Custom command styling */
        .custom-command {
            font-weight: bold;
            color: @theme_selected_bg_color;
        }
        
        /* Description label - smaller and dimmed */
        .description {
            font-size: 0.9em;
            opacity: 0.7;
            margin-top: 2px;
        }
        
        /* Calculator result styling */
        .calculator-result {
            font-family: monospace;
            color: @theme_fg_color;
            font-weight: bold;
        }
        
        /* Search item styling */
        .search-item {
            font-style: italic;
            opacity: 0.8;
        }
    "#
    .to_string()
}

/// Sets up CSS styling for the application
///
/// # Arguments
/// * `window` - The main window to apply styles to
pub fn setup(window: &ApplicationWindow) {
    let css_provider = gtk4::CssProvider::new();
    let css_content = load_css_styles();
    css_provider.load_from_data(&css_content);

    let window_display = gtk4::prelude::WidgetExt::display(window);
    gtk4::style_context_add_provider_for_display(
        &window_display,
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
