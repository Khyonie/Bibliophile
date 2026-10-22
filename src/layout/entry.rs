use std::{
    cell::RefCell,
    io::Error,
    path::{Path, PathBuf},
    rc::Rc,
};

use gtk4::{
    Align, Box, Image, Label, ListBox, ListBoxRow, Orientation, Separator,
    prelude::{BoxExt, ListBoxRowExt, WidgetExt},
};

use crate::model::Browser;

mod fs;

use self::fs::get_entries;

pub struct EntryList {
    pub container: ListBox,
    pub entries: RefCell<Vec<PathBuf>>,
}

pub struct FileEntry {
    pub path: PathBuf,
}

impl FileEntry {
    pub(crate) fn build_widget(&self) -> Result<Box, Error> {
        let (filename, has_control) = self.filename();
        let file_label = Label::new(None);
        if has_control {
            file_label.set_markup(&format!("<i>{filename}</i>"));
        } else {
            file_label.set_text(&filename);
        }

        file_label.set_hexpand(true);
        file_label.set_halign(Align::Start);

        let filetype = if !self.path.is_dir() {
            self.path
                .extension()
                .map_or(String::from("File"), |s| s.to_string_lossy().to_string())
        } else {
            String::from("Folder")
        };

        let filetype_label = Label::new(Some(&filetype));

        // Icon
        let icon_name = if self.path.is_dir() {
            "folder-symbolic"
        } else {
            "text-x-generic-symbolic"
        };
        let icon = Image::from_icon_name(icon_name);

        // Metadata
        let modified_text = self.modified_text();
        let modified_label = Label::new(Some(&modified_text));
        modified_label.set_xalign(0.0);
        modified_label.set_width_chars(25);

        // Attach together
        let row_box = Box::new(Orientation::Horizontal, 16);
        row_box.append(&icon);
        row_box.append(&file_label);
        row_box.append(&filetype_label);
        row_box.append(&modified_label);

        Ok(row_box)
    }
}

impl EntryList {
    pub fn initialize(&self, browser: &Rc<RefCell<Browser>>) -> Result<(), Error> {
        self.build(&browser.clone().borrow().current_dir)
    }

    pub fn rebuild(&self, path: &Path) -> Result<(), Error> {
        println!("Rebuilding view");
        self.clear();
        self.build(path)
    }

    fn clear(&self) {
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
    }

    fn build(&self, path: &Path) -> Result<(), Error> {
        // Header bar
        let header_box = Box::new(Orientation::Horizontal, 12);

        let file_header = Label::new(Some("Filename"));
        file_header.set_hexpand(true);
        file_header.set_halign(Align::Start);
        header_box.append(&file_header);

        header_box.append(&Label::new(Some("Type")));

        let modified_header = Label::new(Some("Modified"));
        modified_header.set_width_chars(25);
        modified_header.set_xalign(0.0);
        header_box.append(&modified_header);

        let header = ListBoxRow::new();
        header.set_activatable(false);
        header.set_selectable(false);
        header.set_child(Some(&header_box));

        let separator = Separator::new(Orientation::Horizontal);
        let separator_box = ListBoxRow::new();
        separator_box.set_activatable(false);
        separator_box.set_selectable(false);
        separator_box.set_child(Some(&separator));

        self.container.append(&header);
        self.container.append(&separator_box);

        let files = get_entries(path)?;

        let mut entries = self.entries.borrow_mut();
        entries.clear();
        for entry in files {
            let entry_box = FileEntry::new(entry.path().as_path()).build_widget()?;

            self.container.append(&entry_box);
            entries.push(entry.path());
        }

        Ok(())
    }
}
