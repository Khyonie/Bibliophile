use std::{cell::RefCell, path::PathBuf, process::exit, rc::Rc};

use gtk4::{
    Application, ApplicationWindow, Box, Button, CenterBox, GestureClick, Image, Label, ListBox,
    ListBoxRow, Orientation, Paned, Popover, ScrolledWindow, Separator, Stack, StackTransitionType,
    gdk::{self, Rectangle},
    gio::{
        AppInfo, AppLaunchContext, File,
        prelude::{ApplicationExt, ApplicationExtManual, FileExt},
    },
    glib::{ExitCode, clone::Downgrade, home_dir},
    graphene::Point,
    prelude::{BoxExt, ButtonExt, GtkWindowExt, ListBoxRowExt, PopoverExt, WidgetExt},
};
use rusqlite::Connection;

use crate::{
    db, fs::bookmark::get_bookmarks, model::Browser, ui::{entry::EntryList, topbar::TopBar},
};

pub mod entry;
pub mod topbar;
pub mod tag;

pub fn initialize() -> ExitCode {
    let app = Application::builder()
        .application_id("coffee.khyonieheart.Bibliophile")
        .build();

    app.connect_activate(build_layout);

    app.run()
}

fn build_layout(application: &Application) {
    let Ok(database) = db::open() else {
        println!("Failed to open database");
        exit(1)
    };

    let window = build_window(application);
    let content = Box::new(Orientation::Vertical, 8);
    let top_bar = build_top_bar();
    let entries = build_entry_list();
    let browser = build_browser(&top_bar, &entries, database);

    connect_entry_activation(&entries, &browser);
    initialize_widgets(&top_bar, &entries, &browser);

    content.append(&top_bar.content);
    content.append(&build_main_content(&entries, &browser));
    window.set_child(Some(&content));
    window.present();
}

fn build_window(application: &Application) -> ApplicationWindow {
    ApplicationWindow::builder()
        .application(application)
        .title("Bibliophile")
        .default_width(1000)
        .default_height(650)
        .build()
}

fn build_top_bar() -> Rc<TopBar> {
    let location_box = Box::new(Orientation::Horizontal, 1);
    let navigation_box = Box::new(Orientation::Horizontal, 8);

    let top_bar = CenterBox::builder()
        .hexpand(true)
        .margin_top(8)
        .margin_start(8)
        .margin_end(8)
        .build();
    Rc::new(TopBar {
        content: top_bar,
        location_box,
        navigation_box,
        browser: RefCell::new(std::rc::Weak::new()),
    })
}

fn build_entry_list() -> Rc<EntryList> {
    let entry_box = ListBox::builder()
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    entry_box.set_activate_on_single_click(false);

    let entries = EntryList {
        container: entry_box,
        entries: RefCell::new(Vec::new()),
    };

    Rc::new(entries)
}

fn build_browser(top_bar: &Rc<TopBar>, entries: &Rc<EntryList>, database: Connection) -> Rc<RefCell<Browser>> {
    Rc::new(RefCell::new(Browser {
        current_dir: home_dir(),
        last_dirs: Vec::new(),
        next_dirs: Vec::new(),
        entry_list: entries.clone(),
        top_bar: top_bar.clone(),
        database
    }))
}

fn connect_entry_activation(entries: &Rc<EntryList>, browser: &Rc<RefCell<Browser>>) {
    let entries_ref = entries.downgrade();
    let browser_ref = browser.downgrade();
    entries.container.connect_row_activated(move |_, row| {
        // What is this black magic?
        let (Some(browser_ref), Some(entries_ref)) = (browser_ref.upgrade(), entries_ref.upgrade())
        else {
            return;
        };

        // The first two rows are the header and separator
        let Some(index) = usize::try_from(row.index())
            .ok()
            .and_then(|index| index.checked_sub(2))
        else {
            return;
        };

        let path = entries_ref.entries.borrow().get(index).cloned();
        if let Some(path) = path {
            if path.is_dir() {
                browser_ref.borrow_mut().move_into(&path);
            } else if path.is_symlink() {
                let target_path = match path.read_link() {
                    Ok(p) => p,
                    Err(e) => {
                        println!("Failed to read symlink link: {e}");
                        return;
                    }
                };

                browser_ref.borrow_mut().move_into(&target_path);
            } else {
                let file = File::for_path(path).uri();

                if let Err(e) = AppInfo::launch_default_for_uri(&file, None::<&AppLaunchContext>) {
                    println!("Failed to open file: {e}");
                };
            }
        }
    });

    let right_click_ref = entries.downgrade();
    let right_click_browser = browser.downgrade();

    let gesture = GestureClick::builder()
        .button(gdk::BUTTON_SECONDARY)
        .build();

    gesture.connect_pressed(move |_gesture, presses, x, y| {
        let (Some(list_box), Some(browser)) = (right_click_ref.upgrade(), right_click_browser.upgrade()) else {
            return;
        };

        if presses == 1
            && let Some(row) = list_box.container.row_at_y(y as i32)
        {
            if row.index() < 2 {
                return;
            }

            handle_row_right_click(browser, row.index() - 2, x, y);
        }
    });

    entries.container.add_controller(gesture);
}
fn initialize_widgets(top_bar: &TopBar, entries: &EntryList, browser: &Rc<RefCell<Browser>>) {
    if let Err(e) = top_bar.initialize(browser) {
        println!("Failed to build top bar: {e}");
        exit(1)
    };

    if let Err(e) = entries.initialize(browser) {
        println!("Failed to build entry box: {e}");
        exit(1)
    };
}

fn build_main_content(entries: &EntryList, browser: &Rc<RefCell<Browser>>) -> Paned {
    // Main body
    let main_content = Paned::builder().hexpand(true).vexpand(true).build();
    main_content.set_position(200);
    let shortcut_list = build_bookmark_list(browser);
    let shortcut_scroller = ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();

    shortcut_scroller.set_child(Some(&shortcut_list));
    main_content.set_start_child(Some(&shortcut_scroller));

    let entry_scroller = ScrolledWindow::builder().vexpand(true).build();
    entry_scroller.set_child(Some(&entries.container));

    main_content.set_end_child(Some(&entry_scroller));
    main_content
}

fn build_bookmark_list(browser: &Rc<RefCell<Browser>>) -> ListBox {
    // TODO Silent error
    let bookmark_paths = get_bookmarks().unwrap_or_else(|_| {
        println!("Failed to read bookmarks file, falling back to empty list");
        Vec::new()
    });

    let bookmark_list = ListBox::new();

    for (_, file_name) in bookmark_paths.iter() {
        let bookmark_entry = ListBoxRow::new();
        bookmark_entry.set_hexpand(true);

        let bookmark_container = Box::new(Orientation::Horizontal, 12);

        // Bookmark icon
        let icon = match file_name.as_str() {
            "Downloads" => Image::from_icon_name("folder-download-symbolic"),
            "Videos" => Image::from_icon_name("folder-videos-symbolic"),
            "Pictures" => Image::from_icon_name("folder-pictures-symbolic"),
            "Documents" => Image::from_icon_name("folder-documents-symbolic"),
            "Music" => Image::from_icon_name("folder-music-symbolic"),
            _ => Image::from_icon_name("user-bookmarks-symbolic"),
        };
        bookmark_container.append(&icon);

        // Bookmark label
        let label = Label::new(None);
        label.set_markup(&format!(
            "<b>{}</b>",
            gtk4::glib::markup_escape_text(file_name)
        ));
        label.set_xalign(0.0);
        bookmark_container.append(&label);

        bookmark_entry.set_child(Some(&bookmark_container));

        bookmark_list.append(&bookmark_entry)
    }

    connect_bookmark_activation(
        &bookmark_list,
        bookmark_paths.into_iter().map(|(path, _)| path).collect(),
        browser,
    );

    bookmark_list
}

fn connect_bookmark_activation(
    bookmark_list: &ListBox,
    bookmark_paths: Vec<PathBuf>,
    browser: &Rc<RefCell<Browser>>,
) {
    let browser = browser.downgrade();
    bookmark_list.connect_row_activated(move |_, row| {
        if let Some(browser) = browser.upgrade() {
            let index = row.index() as usize;
            let path = bookmark_paths
                .get(index)
                .expect("Attempted to index into non-existent bookmark");

            browser.borrow_mut().move_into(path);
        }
    });
}

fn handle_row_right_click(browser: Rc<RefCell<Browser>>, index: i32, x: f64, y: f64) {
    let entries = browser.borrow().entry_list.clone();
    let row = entries
        .container
        .row_at_index(index + 2)
        .expect("Recieved non-existent row");
    let Some(path) = entries.entries.borrow().get(index as usize).cloned() else {
        return;
    };

    let Some(point) = entries
        .container
        .compute_point(&row, &Point::new(x as f32, y as f32))
    else {
        return;
    };
    let position = Rectangle::new(point.x() as i32, point.y() as i32, 1, 1);

    let stack = Stack::new();
    stack.set_hhomogeneous(false);
    stack.set_vhomogeneous(false);
    stack.set_interpolate_size(true);
    stack.set_transition_type(StackTransitionType::SlideLeftRight);

    let menu_page = Box::new(Orientation::Vertical, 0);
    menu_page.set_width_request(180);
    for label in ["Open", "Open with"] {
        let button = context_menu_button(label);
        button.set_sensitive(false);
        menu_page.append(&button);
    }
    match tag::create_tag_menu(&path, &browser) {
        Ok(tag_content) => {
            let tag_button = context_menu_button("Tag ›");
            let tag_stack = stack.downgrade();
            tag_button.connect_clicked(move |_| {
                if let Some(stack) = tag_stack.upgrade() {
                    stack.set_visible_child_name("tags");
                }
            });
            menu_page.append(&tag_button);

            let tag_page = Box::new(Orientation::Vertical, 0);
            tag_page.set_size_request(440, 480);
            let back_button = context_menu_button("‹ Back");
            let main_stack = stack.downgrade();
            back_button.connect_clicked(move |_| {
                if let Some(stack) = main_stack.upgrade() {
                    stack.set_visible_child_name("main");
                }
            });
            tag_page.append(&back_button);
            tag_page.append(&tag_content);
            stack.add_named(&tag_page, Some("tags"));
        }
        Err(error) => {
            eprintln!("Failed to load tags: {error}");
        }
    }
    menu_page.append(&Separator::new(Orientation::Horizontal));
    let copy_button = context_menu_button("Copy");
    copy_button.set_sensitive(false);
    menu_page.append(&copy_button);
    menu_page.append(&Separator::new(Orientation::Horizontal));
    for label in ["Rename", "Move to trash", "Properties"] {
        let button = context_menu_button(label);
        button.set_sensitive(false);
        menu_page.append(&button);
    }
    stack.add_named(&menu_page, Some("main"));
    stack.set_visible_child_name("main");

    let popup = Popover::new();
    popup.add_css_class("menu");
    popup.set_child(Some(&stack));
    popup.set_pointing_to(Some(&position));
    popup.set_parent(&row);
    popup.connect_closed(|popup| popup.unparent());

    // Present
    popup.popup();
}

fn context_menu_button(text: &str) -> Button {
    let button = Button::new();
    button.add_css_class("flat");
    let label = Label::new(Some(text));
    label.set_hexpand(true);
    label.set_xalign(0.0);
    button.set_child(Some(&label));
    button
}
