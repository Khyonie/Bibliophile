use std::{io::Error, path::{Component, Path}, process::exit};

use gtk4::{Application, ApplicationWindow, Box, Button, CenterBox, ListBox, Orientation, ScrolledWindow, SearchEntry, gio::prelude::{ApplicationExt, ApplicationExtManual}, glib::{ExitCode, home_dir}, prelude::{BoxExt, GtkWindowExt}};

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
    let back_button = Button::with_label("< Back");
    let forward_button = Button::with_label("Forward >");
    let up_button = Button::with_label("^");

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
    let shortcut_list = ListBox::new();
    let shortcut_scroller = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();
    shortcut_scroller.set_child(Some(&shortcut_list));
    content.append(&shortcut_scroller);

    window.set_child(Some(&content));

    // Finish window
    window.present();
}

fn build_location_buttons(path: &Path) -> Result<Box, Error>
{
    let widget = Box::new(Orientation::Horizontal, 4);
    
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
