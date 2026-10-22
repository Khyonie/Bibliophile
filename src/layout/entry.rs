use std::{cell::RefCell, fs::DirEntry, io::Error, path::{Path, PathBuf}, rc::Rc};

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

    fn filename(&self) -> (String, bool) {
        let mut filename = self.path.file_stem().map_or(
            self.path
                .file_name()
                .map_or(String::from("Unnamed"), |f| f.to_string_lossy().to_string()),
            |f| f.to_string_lossy().to_string(),
        );

        let has_control = filename.chars().any(|c| c.is_control());
        filename = normalize_escape_characters(&filename);

        if self.path.is_symlink() {
            match self.path.read_link() {
                Ok(target) => filename.push_str(&format!(" → {}", target.to_string_lossy())),
                Err(_) => filename.push_str(" → (Broken symlink)"),
            }
        }

        (filename, has_control)
    }

    fn modified_text(&self) -> String {
        let modified = self.path.metadata().and_then(|metadata| metadata.modified());
        modified.map_or_else(
            |_| String::from("12:00, January 1, 1970"),
            |time| {
                let modified: DateTime<Utc> = time.into();
                modified.format("%H:%M, %B %e, %Y").to_string()
            },
        )
    }

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
            self
                .path
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

fn normalize_escape_characters(name: &str) -> String
{
    name.chars()
        .map(|c| if c.is_control() { String::from("�") } else { c.to_string() })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{os::unix::fs, path::PathBuf};

    use super::FileEntry;

    #[test]
    pub fn file_entry_handles_broken_symlinks() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target_path = temp_dir.path().join("missing_target");
        let target_link_path = temp_dir.path().join("broken_symlink");
        fs::symlink(&target_path, &target_link_path).unwrap();

        let entry = FileEntry::new(&target_link_path);
        assert_eq!(
            entry.filename().0,
            format!("broken_symlink → {}", target_path.display())
        );
        assert_eq!(entry.modified_text(), "12:00, January 1, 1970");
    }

    #[test]
    pub fn file_entry_replaces_control_characters() {
        let file = PathBuf::from("/tmp/\r");
        let entry = FileEntry::new(&file);

        assert_eq!(&entry.filename().0, "�")
    }
}
