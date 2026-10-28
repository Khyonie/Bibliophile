use std::{fs, io::Error, path::PathBuf};

use gtk4::glib::home_dir;

pub(super) fn get_bookmarks() -> Result<Vec<(PathBuf, String)>, Error> {
    let target_path = home_dir().join(".config/gtk-3.0/bookmarks");

    let lines = fs::read_to_string(target_path)?
        .split("\n")
        .filter(|s| !s.is_empty())
        .map(|line| {
            let (uri, label) = line.split_once(' ').unwrap_or((line, ""));
            let path = PathBuf::from(uri.strip_prefix("file://").unwrap_or(uri));
            let name = if label.is_empty() {
                path.file_name()
                    .map_or(String::from("(Failed to parse)"), |s| {
                        s.to_string_lossy().to_string()
                    })
            } else {
                label.to_string()
            };
            (path, name)
        })
        .collect();

    Ok(lines)
}
