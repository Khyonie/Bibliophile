use std::{
    cell::RefCell, fs::DirEntry, io::Error, path::{Path, PathBuf}, rc::Rc,
};

use chrono::{DateTime, Utc};
use gtk4::{
    Align, Box, Image, Label, ListBox, ListBoxRow, Orientation, Separator, prelude::{BoxExt, ListBoxRowExt, WidgetExt},
};

use crate::model::Browser;

pub struct EntryList {
    pub container: ListBox,
    pub entries: RefCell<Vec<PathBuf>>
}

pub struct FileEntry {
    pub path: PathBuf,
}

impl FileEntry {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub(crate) fn build_widget(&self) -> Result<Box, Error> {
        let mut filename = self.path.file_stem().map_or(
            self.path
                .file_name()
                .map_or(String::from("Unnamed"), |f| f.to_string_lossy().to_string()),
            |f| f.to_string_lossy().to_string(),
        );

        let file_label = Label::new(Some(&filename));
        file_label.set_hexpand(true);
        file_label.set_halign(Align::Start);

        let filetype = self
            .path
            .extension()
            .map_or(String::from("File"), |s| s.to_string_lossy().to_string());

        let filetype_label = Label::new(Some(&filetype));

        if self.path.is_symlink() {
            let symlink_path = self.path.read_link()?;
            filename.push_str(&format!(" → {}", symlink_path.to_string_lossy()));
        }

        // Icon
        let icon_name = if self.path.is_dir() {
            "folder-symbolic"
        } else {
            "text-x-generic-symbolic"
        };
        let icon = Image::from_icon_name(icon_name);

        // Metadata
        let modified: DateTime<Utc> = self.path.metadata()?.modified()?.into();
        let modified_label = modified.format("%H:%M, %B %e, %Y").to_string();
        let modified_label = Label::new(Some(&modified_label));
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
    pub fn initialize(&self, browser: &Rc<RefCell<Browser>>) -> Result<(), Error>
    {
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

fn get_entries(path: &Path) -> Result<Vec<DirEntry>, Error>
{
    let mut files: Vec<DirEntry> = path.read_dir()?.filter_map(|e| e.ok()).collect();

    // Sort by directories, then by name
    files.sort_by_key(|d| {
        let is_dir = d.path().is_dir();

        (!is_dir, d.file_name())
    });

    Ok(files)
}
