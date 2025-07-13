use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow, Entry, EventControllerKey, Box, Orientation, ScrolledWindow, ListBox, Label, ListBoxRow};

mod config;
mod apps;

const APP_ID: &str = "com.example.palet";

/// Launches an application in a detached process
/// 
/// # Arguments
/// * `app` - The application to launch
fn launch_application(app: &apps::Application) {
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(&app.exec)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Performs a web search using the configured search URL
/// 
/// # Arguments
/// * `query` - The search query
/// * `search_url_template` - URL template with {q} placeholder
fn perform_web_search(query: &str, search_url_template: &str) {
    let encoded_query = urlencoding::encode(query);
    let search_url = search_url_template.replace("{q}", &encoded_query);
    
    let _ = std::process::Command::new("xdg-open")
        .arg(&search_url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Copies text to the system clipboard
/// 
/// # Arguments
/// * `text` - The text to copy
fn copy_to_clipboard(text: &str) {
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(&format!("echo -n '{}' | xclip -selection clipboard", text))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Executes a custom command with optional arguments
/// 
/// # Arguments
/// * `cmd` - The custom command to execute
/// * `arguments` - Optional arguments to pass to the command
/// * `terminal` - Terminal command to use when tty is true
fn execute_custom_command(cmd: &config::CustomCommand, arguments: Option<&str>, terminal: &str) {
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
    
    println!("Executing custom command: {} -> {} (tty: {})", cmd.name, final_command, cmd.tty);
    let _ = std::process::Command::new("sh")
        .arg("-c")
        .arg(&final_command)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Entry point for the application
fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

/// Creates the text entry widget
/// 
/// # Arguments
/// * `config` - Application configuration containing placeholder text
/// 
/// # Returns
/// The configured Entry widget
fn create_entry(config: &config::Config) -> Entry {
    Entry::builder()
        .placeholder_text(&config.placeholder)
        .css_classes(["app-entry"])
        .build()
}

/// Creates the application list widget
/// 
/// # Returns
/// The configured ListBox widget
fn create_list_box() -> ListBox {
    ListBox::builder()
        .css_classes(["app-list"])
        .build()
}

/// Creates the scrolled window containing the application list
/// 
/// # Arguments
/// * `list_box` - The list box to wrap
/// * `config` - Application configuration for height constraints
/// 
/// # Returns
/// The configured ScrolledWindow widget
fn create_scrolled_window(list_box: &ListBox, config: &config::Config) -> ScrolledWindow {
    ScrolledWindow::builder()
        .child(list_box)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .propagate_natural_height(true)
        .min_content_height(1) // Minimum when no apps
        .max_content_height(config.height.saturating_sub(60).max(100)) // Max minus input space, at least 100
        .css_classes(["app-scroll"])
        .build()
}

/// Creates the main window container
/// 
/// # Arguments
/// * `entry` - The text entry widget
/// * `scrolled_window` - The scrolled window containing the app list
/// 
/// # Returns
/// The configured Box container
fn create_main_container(entry: &Entry, scrolled_window: &ScrolledWindow) -> Box {
    let main_box = Box::builder()
        .orientation(Orientation::Vertical)
        .build();
    
    main_box.append(entry);
    main_box.append(scrolled_window);
    main_box
}

/// Populates the list box with filtered items and shows/hides the scrolled window
/// 
/// # Arguments
/// * `list_box` - The list box to populate
/// * `scrolled_window` - The scrolled window to show/hide
/// * `applications` - All available applications
/// * `config` - Application configuration containing custom commands
/// * `query` - The search query to filter by
fn populate_app_list(list_box: &ListBox, scrolled_window: &ScrolledWindow, applications: &[apps::Application], config: &config::Config, query: &str) {
    // Clear existing items
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }
    
    let filtered_items = apps::filter_applications(applications, &config.custom_commands, query);
    println!("Query: '{}', Found {} items", query, filtered_items.len());
    
    // Show/hide the scrolled window based on whether we have items
    if filtered_items.is_empty() {
        scrolled_window.set_visible(false);
    } else {
        scrolled_window.set_visible(true);
        
        // Add filtered items
        for item in filtered_items.iter() {
            let item_box = create_item_widget(item);
            list_box.append(&item_box);
        }
        
        // Auto-select first item
        select_first_item(list_box);
    }
}

/// Creates a widget for displaying a list item with name and description
/// 
/// # Arguments
/// * `item` - The list item to create a widget for
/// 
/// # Returns
/// A Box widget containing the formatted item
fn create_item_widget(item: &apps::ListItem) -> Box {
    let item_box = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["item-container"])
        .build();
    
    // Main label (name) with appropriate styling based on item type
    let (name_css_class, desc_css_class) = match item {
        apps::ListItem::Application(_) => ("app-name", "description"),
        apps::ListItem::CustomCommand { .. } => ("custom-command", "description"),
        apps::ListItem::Calculator { .. } => ("calculator-result", "description"),
        apps::ListItem::SearchFallback(_) => ("search-item", "description"),
    };
    
    let name_label = Label::builder()
        .label(&item.display_name())
        .halign(gtk4::Align::Start)
        .css_classes([name_css_class])
        .build();
    item_box.append(&name_label);
    
    // Description label (if available)
    if let Some(description) = item.description() {
        let desc_label = Label::builder()
            .label(&description)
            .halign(gtk4::Align::Start)
            .css_classes([desc_css_class])
            .build();
        
        item_box.append(&desc_label);
    }
    
    item_box
}

/// Selects the first item in the list box and scrolls to it
/// 
/// # Arguments
/// * `list_box` - The list box to select the first item in
fn select_first_item(list_box: &ListBox) {
    if let Some(first_child) = list_box.first_child() {
        if let Some(first_row) = first_child.downcast_ref::<ListBoxRow>() {
            list_box.select_row(Some(first_row));
            scroll_to_row(list_box, first_row);
        }
    }
}

/// Sets up text filtering for the entry widget
/// 
/// # Arguments
/// * `entry` - The text entry widget
/// * `list_box` - The list box to update
/// * `scrolled_window` - The scrolled window to show/hide
/// * `applications` - All available applications
/// * `config` - Application configuration containing custom commands
fn setup_text_filtering(entry: &Entry, list_box: &ListBox, scrolled_window: &ScrolledWindow, applications: &[apps::Application], config: &config::Config) {
    let applications_clone = applications.to_vec();
    let list_box_clone = list_box.clone();
    let scrolled_window_clone = scrolled_window.clone();
    let config_clone = config.clone();
    
    entry.connect_changed(move |entry| {
        let text = entry.text();
        populate_app_list(&list_box_clone, &scrolled_window_clone, &applications_clone, &config_clone, &text);
    });
}

/// Creates and configures the main application window
/// 
/// # Arguments
/// * `app` - The GTK Application instance
/// * `main_box` - The main container widget
/// * `config` - Application configuration
/// 
/// # Returns
/// The configured ApplicationWindow
fn create_window(app: &Application, main_box: &Box, config: &config::Config) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Palet")
        .default_width(config.width)
        .child(main_box)
        .resizable(false)
        .build();
    
    // Set window constraints: fixed width, let height adapt naturally
    window.set_size_request(config.width, -1);
    window.set_default_size(config.width, -1);
    
    // Make window floating and always on top for tiling WMs
    window.set_modal(true);
    
    window
}

/// Handles item launch when a list item is clicked
/// 
/// # Arguments
/// * `row` - The clicked row
/// * `entry` - The text entry widget
/// * `applications` - All available applications
/// * `config` - Application configuration
/// * `window` - The main window to close after launch
fn handle_item_click(row: &ListBoxRow, entry: &Entry, applications: &[apps::Application], config: &config::Config, window: &ApplicationWindow) {
    let index = row.index() as usize;
    let text = entry.text();
    let filtered_items = apps::filter_applications(applications, &config.custom_commands, &text);
    
    if let Some(item) = filtered_items.get(index) {
        match item {
            apps::ListItem::Application(app) => {
                launch_application(app);
            }
            apps::ListItem::CustomCommand { command, arguments } => {
                execute_custom_command(command, arguments.as_deref(), &config.terminal);
            }
            apps::ListItem::Calculator { result, .. } => {
                copy_to_clipboard(result);
                println!("Copied calculation result to clipboard: {}", result);
            }
            apps::ListItem::SearchFallback(query) => {
                perform_web_search(query, &config.search_url);
            }
        }
        window.close();
    }
}

/// Sets up click handling for list items
/// 
/// # Arguments
/// * `list_box` - The list box widget
/// * `entry` - The text entry widget
/// * `applications` - All available applications
/// * `config` - Application configuration
/// * `window` - The main window
fn setup_click_handling(list_box: &ListBox, entry: &Entry, applications: &[apps::Application], config: &config::Config, window: &ApplicationWindow) {
    let applications_clone = applications.to_vec();
    let entry_clone = entry.clone();
    let config_clone = config.clone();
    let window_clone = window.clone();
    
    list_box.connect_row_activated(move |_, row| {
        handle_item_click(row, &entry_clone, &applications_clone, &config_clone, &window_clone);
    });
}

/// Scrolls to ensure the selected row is visible
/// 
/// # Arguments
/// * `list_box` - The list box containing the row
/// * `row` - The row to scroll to
fn scroll_to_row(list_box: &ListBox, row: &ListBoxRow) {
    // Walk up the widget hierarchy to find the ScrolledWindow
    let mut current = Some(list_box.clone().upcast::<gtk4::Widget>());
    let mut scrolled_window: Option<ScrolledWindow> = None;
    
    while let Some(widget) = current {
        if let Ok(sw) = widget.clone().downcast::<ScrolledWindow>() {
            scrolled_window = Some(sw);
            break;
        }
        current = widget.parent();
    }
    
    if let Some(scrolled_window) = scrolled_window {
        // Use a simple approach: just ensure the row is visible by scrolling to it
        glib::idle_add_local_once({
            let row_clone = row.clone();
            let scrolled_window_clone = scrolled_window.clone();
            move || {
                // Get the adjustment
                let adjustment = scrolled_window_clone.vadjustment();
                
                // Get row index to calculate approximate position
                let row_index = row_clone.index() as f64;
                
                // Estimate row height (approximate) - accounts for description line too
                let estimated_row_height = 80.0; // Approximate row height with description
                let row_position = row_index * estimated_row_height;
                
                let visible_top = adjustment.value();
                let visible_bottom = visible_top + adjustment.page_size();
                let row_bottom = row_position + estimated_row_height;
                
                // Scroll if needed
                if row_position < visible_top {
                    adjustment.set_value(row_position.max(0.0));
                } else if row_bottom > visible_bottom {
                    let new_value = (row_bottom - adjustment.page_size()).max(0.0);
                    let max_value = adjustment.upper() - adjustment.page_size();
                    adjustment.set_value(new_value.min(max_value.max(0.0)));
                }
            }
        });
    }
}

/// Handles keyboard navigation (up/down arrows)
/// 
/// # Arguments
/// * `key` - The pressed key
/// * `list_box` - The list box to navigate
/// 
/// # Returns
/// Whether the key was handled
fn handle_navigation_key(key: gtk4::gdk::Key, list_box: &ListBox) -> bool {
    match key {
        gtk4::gdk::Key::Down => {
            if let Some(selected) = list_box.selected_row() {
                if let Some(next) = selected.next_sibling() {
                    if let Some(next_row) = next.downcast_ref::<ListBoxRow>() {
                        list_box.select_row(Some(next_row));
                        scroll_to_row(list_box, next_row);
                    }
                }
            } else if let Some(first) = list_box.first_child() {
                if let Some(first_row) = first.downcast_ref::<ListBoxRow>() {
                    list_box.select_row(Some(first_row));
                    scroll_to_row(list_box, first_row);
                }
            }
            true
        }
        gtk4::gdk::Key::Up => {
            if let Some(selected) = list_box.selected_row() {
                if let Some(prev) = selected.prev_sibling() {
                    if let Some(prev_row) = prev.downcast_ref::<ListBoxRow>() {
                        list_box.select_row(Some(prev_row));
                        scroll_to_row(list_box, prev_row);
                    }
                }
            }
            true
        }
        _ => false
    }
}

/// Handles item launch via Enter key
/// 
/// # Arguments
/// * `entry` - The text entry widget
/// * `list_box` - The list box widget
/// * `applications` - All available applications
/// * `config` - Application configuration
/// * `window` - The main window to close after launch
fn handle_enter_key(entry: &Entry, list_box: &ListBox, applications: &[apps::Application], config: &config::Config, window: &ApplicationWindow) {
    let text = entry.text();
    let filtered_items = apps::filter_applications(applications, &config.custom_commands, &text);
    println!("Enter pressed - query: '{}', filtered items: {}", text, filtered_items.len());
    
    let index = if let Some(selected_row) = list_box.selected_row() {
        let idx = selected_row.index() as usize;
        println!("Selected row index: {}", idx);
        idx
    } else {
        println!("No row selected, using index 0");
        0 // Default to first item if none selected
    };
    
    if let Some(item) = filtered_items.get(index) {
        match item {
            apps::ListItem::Application(app) => {
                println!("Launching app: {} -> {}", app.name, app.exec);
                launch_application(app);
            }
            apps::ListItem::CustomCommand { command, arguments } => {
                let cmd_str = if let Some(args) = arguments {
                    format!("{} {}", command.command, args)
                } else {
                    command.command.clone()
                };
                println!("Executing custom command: {} -> {}", command.name, cmd_str);
                execute_custom_command(command, arguments.as_deref(), &config.terminal);
            }
            apps::ListItem::Calculator { result, .. } => {
                println!("Copying calculation result to clipboard: {}", result);
                copy_to_clipboard(result);
            }
            apps::ListItem::SearchFallback(query) => {
                println!("Performing web search for: {}", query);
                perform_web_search(query, &config.search_url);
            }
        }
        window.close();
    } else if !filtered_items.is_empty() {
        println!("Index out of bounds, launching first item");
        if let Some(first_item) = filtered_items.first() {
            match first_item {
                apps::ListItem::Application(app) => launch_application(app),
                apps::ListItem::CustomCommand { command, arguments } => execute_custom_command(command, arguments.as_deref(), &config.terminal),
                apps::ListItem::Calculator { result, .. } => copy_to_clipboard(result),
                apps::ListItem::SearchFallback(query) => perform_web_search(query, &config.search_url),
            }
        }
        window.close();
    } else {
        println!("No items to launch");
    }
}

/// Sets up Enter key handling using Entry's activate signal and separate navigation controller
/// 
/// # Arguments
/// * `window` - The main window
/// * `entry` - The text entry widget
/// * `list_box` - The list box widget
/// * `applications` - All available applications
/// * `config` - Application configuration
fn setup_keyboard_handling(window: &ApplicationWindow, entry: &Entry, list_box: &ListBox, applications: &[apps::Application], config: &config::Config) {
    // Handle Enter key via Entry's activate signal (fired when Enter is pressed)
    let window_clone = window.clone();
    let list_box_clone = list_box.clone();
    let applications_clone = applications.to_vec();
    let config_clone = config.clone();
    let entry_clone = entry.clone();
    
    entry.connect_activate(move |_| {
        println!("Entry activated (Enter pressed)");
        handle_enter_key(&entry_clone, &list_box_clone, &applications_clone, &config_clone, &window_clone);
    });
    
    // Handle other keys (Escape, navigation) on the window level
    let key_controller = EventControllerKey::new();
    let window_clone2 = window.clone();
    let list_box_clone2 = list_box.clone();
    
    key_controller.connect_key_pressed(move |_, key, _, _| {
        println!("Window key pressed: {}", key);
        match key {
            gtk4::gdk::Key::Escape => {
                window_clone2.close();
                gtk4::glib::Propagation::Stop
            }
            _ => {
                if handle_navigation_key(key, &list_box_clone2) {
                    gtk4::glib::Propagation::Stop
                } else {
                    gtk4::glib::Propagation::Proceed
                }
            }
        }
    });
    
    window.add_controller(key_controller);
}

/// Loads CSS from XDG config directory or uses default styles
/// 
/// # Returns
/// CSS content as a string
fn load_css_styles() -> String {
    // Try to load custom CSS from XDG config directory
    if let Some(config_dir) = dirs::config_dir() {
        let css_path = config_dir.join("palet").join("style.css");
        if let Ok(custom_css) = std::fs::read_to_string(&css_path) {
            println!("Loaded custom CSS from: {}", css_path.display());
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
    "#.to_string()
}

/// Sets up CSS styling for the application
/// 
/// # Arguments
/// * `window` - The main window to apply styles to
fn setup_css_styling(window: &ApplicationWindow) {
    let css_provider = gtk4::CssProvider::new();
    let css_content = load_css_styles();
    css_provider.load_from_data(&css_content);
    
    let window_display = gtk4::prelude::WidgetExt::display(window);
    gtk4::style_context_add_provider_for_display(
        &window_display,
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION
    );
}

/// Shows the window and sets initial focus
/// 
/// # Arguments
/// * `window` - The main window to present
/// * `entry` - The text entry widget to focus
/// * `scrolled_window` - The scrolled window to hide initially
fn show_window(window: &ApplicationWindow, entry: &Entry, scrolled_window: &ScrolledWindow) {
    // Set up CSS styling
    setup_css_styling(window);
    
    // Hide the list initially since there are no items
    scrolled_window.set_visible(false);
    
    window.present();
    entry.grab_focus();
}

/// Main UI building function
/// 
/// # Arguments
/// * `app` - The GTK Application instance
fn build_ui(app: &Application) {
    let config = config::load_config();
    let applications = apps::scan_applications();
    println!("Loaded {} applications", applications.len());
    
    // Create UI components
    let entry = create_entry(&config);
    let list_box = create_list_box();
    let scrolled_window = create_scrolled_window(&list_box, &config);
    let main_box = create_main_container(&entry, &scrolled_window);
    let window = create_window(app, &main_box, &config);
    
    // Set up event handling
    setup_text_filtering(&entry, &list_box, &scrolled_window, &applications, &config);
    setup_click_handling(&list_box, &entry, &applications, &config, &window);
    setup_keyboard_handling(&window, &entry, &list_box, &applications, &config);
    
    // Show the window
    show_window(&window, &entry, &scrolled_window);
}
