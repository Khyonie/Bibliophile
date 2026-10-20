use std::{fs::{self, DirEntry}, io::Error, path::{Component, Path}, process::exit};

use chrono::{DateTime, Utc};
use gtk4::{Align, Application, ApplicationWindow, Box, Button, CenterBox, Image, Label, ListBox, Orientation, Paned, ScrolledWindow, SearchEntry, Separator, gio::prelude::{ApplicationExt, ApplicationExtManual}, glib::{ExitCode, home_dir}, prelude::{BoxExt, GtkWindowExt, WidgetExt}};

pub fn initialize() -> ExitCode
{
    let app = Application::builder()
        .application_id("com.example.Tagfiles")
        .build();

    app.connect_activate(build_layout);

    app.run()
}

fn build_layout(application: &Application)
{
    let window = ApplicationWindow::builder()
        .application(application)
        .title("Bibliophile")
        .default_width(1000)
        .default_height(650)
        .build();

    let content = Box::new(Orientation::Vertical, 8);

    // Top bar
    let navigation_buttons = Box::new(Orientation::Horizontal, 8);
    let back_button = Button::with_label("❮");
    let forward_button = Button::with_label("❯");
    let up_button = Button::with_label("⌃");

    navigation_buttons.append(&back_button);
    navigation_buttons.append(&forward_button);
    navigation_buttons.append(&up_button);

    let path = home_dir();
    let location_buttons = match build_location_buttons(&path) {
        Ok(b) => b,
        Err(e) => {
            println!("Failed to build location buttons: {e}");
            exit(1)
        }
    };

    let search_bar = SearchEntry::new();
    let action_bar = CenterBox::builder()
        .hexpand(false)
        .margin_top(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    action_bar.set_start_widget(Some(&navigation_buttons));
    action_bar.set_center_widget(Some(&location_buttons));
    action_bar.set_end_widget(Some(&search_bar));

    content.append(&action_bar);

    // Main body
    let main_content = Paned::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    let shortcut_list = ListBox::new();
    let shortcut_scroller = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();
    shortcut_scroller.set_child(Some(&shortcut_list));
    main_content.set_start_child(Some(&shortcut_scroller));

    let entries = match build_entry_list(&path) {
        Ok(e) => e,
        Err(e) => {
            println!("Failed to build list entries: {e}");
            exit(1)
        }
    };

    let entry_scroller = ScrolledWindow::builder()
        .vexpand(true)
        .build();
    entry_scroller.set_child(Some(&entries));

    main_content.set_end_child(Some(&entry_scroller));
    content.append(&main_content);

    window.set_child(Some(&content));

    // Finish window
    window.present();
}

fn build_location_buttons(path: &Path) -> Result<Box, Error>
{
    let widget = Box::new(Orientation::Horizontal, 1);
    
    let path = path.canonicalize()?;
    for component in path.components()
    {
        let button = match component {
            Component::RootDir => Some(Button::with_label("/")),
            Component::Normal(os_str) => Some(Button::with_label(&os_str.to_string_lossy())),
            Component::Prefix(_) => todo!("Windows components are not supported at this time."),
            _ => None
        };

        if let Some(button) = button
        {
            widget.append(&button);
        }
    }

    Ok(widget)
}

fn build_entry_list(path: &Path) -> Result<Box, Error>
{
    let content = Box::new(Orientation::Vertical, 12);
    content.set_margin_top(8);
    content.set_margin_start(8);
    content.set_margin_end(8);
    content.set_margin_bottom(8);

    // Header bar
    let header = Box::new(Orientation::Horizontal, 12);

    let file_header = Label::new(Some("File Name"));
    file_header.set_hexpand(true);
    file_header.set_halign(Align::Start);
    header.append(&file_header);

    header.append(&Label::new(Some("Modified")));

    content.append(&header);
    content.append(&Separator::new(Orientation::Horizontal));
    
    let files: Vec<DirEntry> = path.read_dir()?
        .filter_map(|e| e.ok())
        .collect();

    for entry in files
    {
        let entry_box = Box::new(Orientation::Horizontal, 8);
        let metadata = entry.metadata()?;

        // Icon
        let icon_name = if entry.path().is_dir() {
            "folder-symbolic"
        } else {
            "text-x-generic-symbolic"
        };

        let icon = Image::from_icon_name(icon_name);
        icon.set_pixel_size(24);
        entry_box.append(&icon);
        
        // Name
        let mut filename = entry.file_name().to_string_lossy().to_string();
        if filename.trim().is_empty()
        {
            filename = String::from("(Unnamed file)");
        }

        if metadata.is_symlink()
        {
            let target = fs::read_link(entry.path())?;
            filename.push_str(&format!(" → {}", target.to_string_lossy()));
        }

        let label = Label::new(Some(&filename));
        label.set_hexpand(true);
        label.set_xalign(0.0);
        entry_box.append(&label);

        // Metadata
        let modified: DateTime<Utc> = metadata.modified()?.into();
        let modified_label = modified.format("%H:%M, %B %e, %Y").to_string();
        let modified_label = Label::new(Some(&modified_label));
        entry_box.append(&modified_label);

        content.append(&entry_box);
    }

    Ok(content)
}
