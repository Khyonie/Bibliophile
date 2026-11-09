use std::{cell::RefCell, path::Path, rc::Rc, sync::atomic::{AtomicUsize, Ordering}};

use gtk4::{Align, Box, CenterBox, CheckButton, CssProvider, Image, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SearchEntry, Separator, gdk, prelude::{BoxExt, ListBoxRowExt, WidgetExt}};
use rusqlite::Error;

use crate::{model::Browser, tag::Tag};

static NEXT_TAG_MENU_ID: AtomicUsize = AtomicUsize::new(0);

pub fn create_tag_menu(path: &Path, browser: &Rc<RefCell<Browser>>) -> Result<Box, Error>
{
    let container = Box::new(Orientation::Vertical, 12);
    container.set_vexpand(true);
    container.set_margin_top(12);
    container.set_margin_bottom(12);
    container.set_margin_start(12);
    container.set_margin_end(12);

    let scope = format!("tag-menu-{}", NEXT_TAG_MENU_ID.fetch_add(1, Ordering::Relaxed));
    container.add_css_class(&scope);
    let mut css = format!(
        ".{scope} .tag-title {{ font-size: 18px; font-weight: 600; }} \
         .{scope} .tag-eyebrow {{ opacity: 0.65; }}"
    );

    // Construct elements
    let top_bar = create_header_bar(path);
    let search_bar = create_search_bar(browser);
    let list_entry = create_tag_list(browser, &scope, &mut css)?;

    // Build
    container.append(&top_bar);
    container.append(&Separator::new(Orientation::Horizontal));
    container.append(&search_bar);
    container.append(&Separator::new(Orientation::Horizontal));
    container.append(&list_entry);
    container.append(&Separator::new(Orientation::Horizontal));

    let style = CssProvider::new();
    style.load_from_data(&css);
    let mapped_style = style.clone();
    container.connect_map(move |widget| {
        gtk4::style_context_add_provider_for_display(
            &widget.display(), &mapped_style, gtk4::STYLE_PROVIDER_PRIORITY_USER + 1,
        );
    });
    container.connect_unmap(move |widget| {
        gtk4::style_context_remove_provider_for_display(&widget.display(), &style);
    });
    
    Ok(container)
}

/// Top bar, with label and icon
fn create_header_bar(path: &Path) -> Box
{
    let icon = Image::from_icon_name("tag-symbolic");

    let label_box = Box::new(Orientation::Vertical, 2);
    label_box.set_hexpand(true);

    // Text
    let eyebrow = Label::new(path.file_name().and_then(|s| s.to_str()));
    let label = Label::new(Some("Assign tags"));
    eyebrow.add_css_class("tag-eyebrow");
    label.add_css_class("tag-title");
    eyebrow.set_xalign(0.0);
    label.set_xalign(0.0);

    label_box.append(&eyebrow);
    label_box.append(&label);

    // Container
    let container = Box::new(Orientation::Horizontal, 16);
    container.set_hexpand(true);
    container.append(&icon);
    container.append(&label_box);

    container
}

fn create_search_bar(browser: &Rc<RefCell<Browser>>) -> Box
{
    let search_bar = SearchEntry::new();
    search_bar.set_hexpand(true);

    let container = Box::new(Orientation::Horizontal, 12);
    container.append(&search_bar);

    container
}

fn create_tag_list(browser: &Rc<RefCell<Browser>>, scope: &str, css: &mut String) -> Result<Box, Error>
{
    let container = Box::new(Orientation::Vertical, 12);
    container.set_vexpand(true);

    // Header box
    let header = CenterBox::new();
    header.set_hexpand(true);

    header.set_start_widget(Some(&Label::new(Some("Available tags"))));

    // Tag list
    let list_container = ScrolledWindow::new();
    list_container.set_propagate_natural_width(true);
    list_container.set_propagate_natural_height(true);
    list_container.set_max_content_height(320);
    list_container.set_vexpand(true);
    let list = ListBox::new();
    list_container.set_child(Some(&list));

    let tags = Tag::get_tags(&browser.borrow().database)?;
    for (index, tag) in tags.into_iter().enumerate()
    {
        let entry = create_tag_entry(&tag, index, scope, css);
        list.append(&entry);
    }

    container.append(&header);
    container.append(&list_container);

    Ok(container)
}

fn create_tag_entry(tag: &Tag, index: usize, scope: &str, css: &mut String) -> ListBoxRow
{
    let container = CenterBox::new();
    let label = Label::new(Some(&tag.name));
    label.set_halign(Align::Start);
    if let Ok(color) = gdk::RGBA::parse(&tag.color) {
        let class = format!("tag-pill-{index}");
        label.add_css_class(&class);
        let mut background = color;
        background.set_alpha(color.alpha() * 0.18);
        css.push_str(&format!(
            " .{scope} .{class} {{ color: {color}; border: 1px solid {color}; background-color: {background}; border-radius: 999px; padding: 4px 10px; }}"
        ));
    }
    let checkbox = CheckButton::new();

    container.set_start_widget(Some(&label));
    container.set_end_widget(Some(&checkbox));

    let entry_box = ListBoxRow::new();
    entry_box.set_hexpand(true);
    entry_box.set_child(Some(&container));

    entry_box
}
