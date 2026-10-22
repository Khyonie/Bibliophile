use std::{fs::DirEntry, io::Error, path::Path};

use chrono::{DateTime, Utc};

use super::FileEntry;

impl FileEntry {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub(super) fn filename(&self) -> (String, bool) {
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

    pub(super) fn modified_text(&self) -> String {
        let modified = self
            .path
            .metadata()
            .and_then(|metadata| metadata.modified());
        modified.map_or_else(
            |_| String::from("12:00, January 1, 1970"),
            |time| {
                let modified: DateTime<Utc> = time.into();
                modified.format("%H:%M, %B %e, %Y").to_string()
            },
        )
    }
}

pub(super) fn get_entries(path: &Path) -> Result<Vec<DirEntry>, Error> {
    let mut files: Vec<DirEntry> = path.read_dir()?.filter_map(|e| e.ok()).collect();

    // Sort by directories, then by name
    files.sort_by_key(|d| {
        let is_dir = d.path().is_dir();

        (!is_dir, d.file_name())
    });

    Ok(files)
}

fn normalize_escape_characters(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_control() {
                String::from("�")
            } else {
                c.to_string()
            }
        })
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
