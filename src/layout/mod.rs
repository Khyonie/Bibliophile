use std::{cell::RefCell, process::exit, rc::Rc};

use gtk4::{
    Application, ApplicationWindow, Box, CenterBox, ListBox, Orientation, Paned, ScrolledWindow, gio::prelude::{ApplicationExt, ApplicationExtManual}, glib::{ExitCode, clone::Downgrade, home_dir}, prelude::{BoxExt, GtkWindowExt, ListBoxRowExt},
};

use crate::{
    layout::{entry::EntryList, topbar::TopBar}, model::Browser,
};

pub mod entry;
pub mod topbar;

pub fn initialize() -> ExitCode {
    let app = Application::builder()
        .application_id("coffee.khyonieheart.Bibliophile")
        .build();

    app.connect_activate(build_layout);

    app.run()
}

fn build_layout(application: &Application) {
    let window = ApplicationWindow::builder()
        .application(application)
        .title("Bibliophile")
        .default_width(1000)
        .default_height(650)
        .build();

    let content = Box::new(Orientation::Vertical, 8);
    let location_box = Box::new(Orientation::Horizontal, 1);
    let navigation_box = Box::new(Orientation::Horizontal, 8);

    let top_bar = CenterBox::builder()
        .hexpand(true)
        .margin_top(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    let top_bar = Rc::new(TopBar {
        content: top_bar,
        location_box,
        navigation_box,
        browser: RefCell::new(std::rc::Weak::new()),
    });

    let entry_box = ListBox::builder()
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    entry_box.set_activate_on_single_click(false);

    let entries = EntryList{ container: entry_box, entries: RefCell::new(Vec::new()) };

    let entries = Rc::new(entries);
    let browser = Rc::new(RefCell::new(Browser {
        current_dir: home_dir(),
        last_dirs: Vec::new(),
        next_dirs: Vec::new(),
        entry_list: entries.clone(),
        top_bar: top_bar.clone(),
    }));

    let entries_ref = entries.downgrade();
    let browser_ref  = browser.downgrade();
    entries.container.connect_row_activated(move |_, row| {
        // What is this black magic?
        let (Some(browser_ref), Some(entries_ref)) = (browser_ref.upgrade(), entries_ref.upgrade())
        else {
            return;
        };

        // The first two rows are the header and separator
        let Some(index) = usize::try_from(row.index()).ok().and_then(|index| index.checked_sub(2)) else {
            return;
        };

        let path = entries_ref.entries.borrow().get(index).cloned();
        if let Some(path) = path.filter(|path| path.is_dir()) {
            browser_ref.borrow_mut().move_into(&path);
        }
    });

    if let Err(e) = top_bar.initialize(&browser) {
        println!("Failed to build top bar: {e}");
        exit(1)
    };

    if let Err(e) = entries.initialize(&browser) {
        println!("Failed to build entry box: {e}");
        exit(1)
    };

    content.append(&top_bar.content);

    // Main body
    let main_content = Paned::builder().hexpand(true).vexpand(true).build();
    main_content.set_position(200);
    let shortcut_list = ListBox::new();
    let shortcut_scroller = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();
    shortcut_scroller.set_child(Some(&shortcut_list));
    main_content.set_start_child(Some(&shortcut_scroller));

    let entry_scroller = ScrolledWindow::builder().vexpand(true).build();
    entry_scroller.set_child(Some(&entries.container));

    main_content.set_end_child(Some(&entry_scroller));
    content.append(&main_content);

    window.set_child(Some(&content));

    // Finish window
    window.present();
}
