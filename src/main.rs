use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Entry, EventControllerKey, Label, ListBox, ListBoxRow,
    Orientation, ScrolledWindow, glib,
};

mod application;
mod config;
mod queryable;
mod style;

const APP_ID: &str = "com.example.palet";

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
    ListBox::builder().css_classes(["app-list"]).build()
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
    let main_box = Box::builder().orientation(Orientation::Vertical).build();

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
fn populate_app_list(
    list_box: &ListBox,
    scrolled_window: &ScrolledWindow,
    applications: &[application::Application],
    config: &config::Config,
    query: &str,
) {
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let filtered_items = application::filter_applications(applications, &config, query);

    if filtered_items.is_empty() {
        scrolled_window.set_visible(false);
    } else {
        scrolled_window.set_visible(true);

        for item in filtered_items.iter() {
            let item_box = create_item_widget(item);
            list_box.append(&item_box);
        }

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
fn create_item_widget(item: &queryable::Queryable) -> Box {
    let item_box = Box::builder()
        .orientation(Orientation::Vertical)
        .css_classes(["item-container"])
        .build();

    // Main label (name) with appropriate styling based on item type
    let (name_css_class, desc_css_class) = item.classes();

    let name_label = Label::builder()
        .label(&item.display_name())
        .halign(gtk4::Align::Start)
        .css_classes([name_css_class])
        .build();
    item_box.append(&name_label);

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
        select_and_scroll_to(list_box, first_child)
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
fn setup_text_filtering(
    entry: &Entry,
    list_box: &ListBox,
    scrolled_window: &ScrolledWindow,
    applications: &[application::Application],
    config: &config::Config,
) {
    let applications_clone = applications.to_vec();
    let list_box_clone = list_box.clone();
    let scrolled_window_clone = scrolled_window.clone();
    let config_clone = config.clone();

    entry.connect_changed(move |entry| {
        let text = entry.text();
        populate_app_list(
            &list_box_clone,
            &scrolled_window_clone,
            &applications_clone,
            &config_clone,
            &text,
        );
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

    window.set_size_request(config.width, -1);
    window.set_default_size(config.width, -1);
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
fn handle_item_click(
    row: &ListBoxRow,
    entry: &Entry,
    applications: &[application::Application],
    config: &config::Config,
    window: &ApplicationWindow,
) {
    let index = row.index() as usize;
    let text = entry.text();
    let filtered_items = application::filter_applications(applications, &config, &text);

    if let Some(item) = filtered_items.get(index) {
        item.action(config);
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
fn setup_click_handling(
    list_box: &ListBox,
    entry: &Entry,
    applications: &[application::Application],
    config: &config::Config,
    window: &ApplicationWindow,
) {
    let applications_clone = applications.to_vec();
    let entry_clone = entry.clone();
    let config_clone = config.clone();
    let window_clone = window.clone();

    list_box.connect_row_activated(move |_, row| {
        handle_item_click(
            row,
            &entry_clone,
            &applications_clone,
            &config_clone,
            &window_clone,
        );
    });
}

/// Scrolls to ensure the selected row is visible
///
/// # Arguments
/// * `list_box` - The list box containing the row
/// * `row` - The row to scroll to
fn scroll_to_row(list_box: &ListBox, row: &ListBoxRow) {
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
        glib::idle_add_local_once({
            let row_clone = row.clone();
            let scrolled_window_clone = scrolled_window.clone();
            move || {
                let adjustment = scrolled_window_clone.vadjustment();

                let row_index = row_clone.index() as f64;

                // Estimate row height (approximate) - accounts for description line too
                let estimated_row_height = 80.0; // Approximate row height with description
                let row_position = row_index * estimated_row_height;

                let visible_top = adjustment.value();
                let visible_bottom = visible_top + adjustment.page_size();
                let row_bottom = row_position + estimated_row_height;

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
                    select_and_scroll_to(list_box, next);
                }
            } else if let Some(first) = list_box.first_child() {
                select_and_scroll_to(list_box, first);
            }
            true
        }
        gtk4::gdk::Key::Up => {
            if let Some(selected) = list_box.selected_row() {
                if let Some(prev) = selected.prev_sibling() {
                    select_and_scroll_to(list_box, prev)
                }
            }
            true
        }
        _ => false,
    }
}

fn select_and_scroll_to(list_box: &ListBox, selected: gtk4::Widget) {
    if let Some(inner_item) = selected.downcast_ref::<ListBoxRow>() {
        list_box.select_row(Some(inner_item));
        scroll_to_row(list_box, inner_item);
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
fn handle_enter_key(
    entry: &Entry,
    list_box: &ListBox,
    applications: &[application::Application],
    config: &config::Config,
    window: &ApplicationWindow,
) {
    let text = entry.text();
    let filtered_items = application::filter_applications(applications, &config, &text);

    let index = if let Some(selected_row) = list_box.selected_row() {
        let index = selected_row.index() as usize;
        index
    } else {
        0
    };

    if let Some(item) = filtered_items.get(index) {
        item.action(config);
    } else if !filtered_items.is_empty() {
        if let Some(first_item) = filtered_items.first() {
            first_item.action(config);
        }
    }
    window.close();
}

/// Sets up Enter key handling using Entry's activate signal and separate navigation controller
///
/// # Arguments
/// * `window` - The main window
/// * `entry` - The text entry widget
/// * `list_box` - The list box widget
/// * `applications` - All available applications
/// * `config` - Application configuration
fn setup_keyboard_handling(
    window: &ApplicationWindow,
    entry: &Entry,
    list_box: &ListBox,
    applications: &[application::Application],
    config: &config::Config,
) {
    let window_clone = window.clone();
    let list_box_clone = list_box.clone();
    let applications_clone = applications.to_vec();
    let config_clone = config.clone();
    let entry_clone = entry.clone();

    entry.connect_activate(move |_| {
        handle_enter_key(
            &entry_clone,
            &list_box_clone,
            &applications_clone,
            &config_clone,
            &window_clone,
        );
    });

    let key_controller = EventControllerKey::new();
    let window_clone2 = window.clone();
    let list_box_clone2 = list_box.clone();

    key_controller.connect_key_pressed(move |_, key, _, _| match key {
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
    });

    window.add_controller(key_controller);
}

/// Shows the window and sets initial focus
///
/// # Arguments
/// * `window` - The main window to present
/// * `entry` - The text entry widget to focus
/// * `scrolled_window` - The scrolled window to hide initially
fn show_window(window: &ApplicationWindow, entry: &Entry, scrolled_window: &ScrolledWindow) {
    style::setup(window);

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
    let applications = application::scan_applications(&config);

    let entry = create_entry(&config);
    let list_box = create_list_box();
    let scrolled_window = create_scrolled_window(&list_box, &config);
    let main_box = create_main_container(&entry, &scrolled_window);
    let window = create_window(app, &main_box, &config);

    setup_text_filtering(&entry, &list_box, &scrolled_window, &applications, &config);
    setup_click_handling(&list_box, &entry, &applications, &config, &window);
    setup_keyboard_handling(&window, &entry, &list_box, &applications, &config);

    show_window(&window, &entry, &scrolled_window);
}
