use std::path::Path;

use gtk4::{Box, Button, Dialog, Entry, HeaderBar, Label, Orientation, Paned, prelude::{BoxExt, GtkWindowExt, WidgetExt}};

use crate::{model::Browser, tag::Tag};

pub fn create_tag_popup(path: &Path, browser: &Browser) {
    let window = Dialog::builder().build();

    // Header
    let header = HeaderBar::builder().hexpand(true).build();

    window.set_titlebar(Some(&header));
    window.set_title(Some("Manage Tags"));

    let pane_container = Paned::new(Orientation::Horizontal);

    // Tag list
    let added_tag_list = Box::new(Orientation::Vertical, 12);
    added_tag_list.set_hexpand(true);
    added_tag_list.set_vexpand(true);
    
    // Add existing tags to list
    let Ok(tags) = Tag::get_tags(&browser.database) else {
        println!("Failed to read database");
        return
    };

    for tag in tags
    {
        let (name, color) = (tag.name, tag.color);
        let label = Label::new(None);
        label.set_markup(&format!("<span foreground='{color}'>{name}</span>"));

        added_tag_list.append(&label);
    }

    // New tab list
    let new_tag_container = Box::new(Orientation::Vertical, 12);
    new_tag_container.set_margin_top(8);
    new_tag_container.set_margin_bottom(8);
    new_tag_container.set_margin_start(8);
    new_tag_container.set_margin_end(8);
    let new_tag_list = Box::builder()
        .orientation(Orientation::Vertical)
        .vexpand(true)
        .build();
    new_tag_container.append(&new_tag_list);

    // Entry line
    let new_tag_entry_container = Box::new(Orientation::Horizontal, 12);
    let new_tag_text_bar = Entry::new();
    new_tag_text_bar.set_hexpand(true);
    let add_tag_button = Button::with_label("+");

    new_tag_entry_container.append(&new_tag_text_bar);
    new_tag_entry_container.append(&add_tag_button);

    new_tag_container.append(&new_tag_entry_container);

    pane_container.set_end_child(Some(&added_tag_list));
    pane_container.set_start_child(Some(&new_tag_container));

    window.set_child(Some(&pane_container));
    window.present();
}
