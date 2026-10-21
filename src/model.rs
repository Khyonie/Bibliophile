use std::{path::{Path, PathBuf}, rc::Rc};

use crate::layout::{entry::EntryList, topbar::TopBar};

const MAX_STORED_DIRS: usize = 20;

pub struct Browser {
    pub current_dir: PathBuf,
    pub last_dirs: Vec<PathBuf>,
    pub next_dirs: Vec<PathBuf>,
    pub entry_list: Rc<EntryList>,
    pub top_bar: Rc<TopBar>,
}

impl Browser {
    pub fn refresh(&self) {
        if let Err(e) = self.entry_list.rebuild(&self.current_dir) {
            println!("Failed to refresh: {e}");
        }

        if let Err(e) = self.top_bar.rebuild(self) {
            println!("Failed to build location buttons: {e}")
        }
    }

    pub fn move_into(&mut self, path: &Path)
    {
        self.store_current_as_last();
        self.current_dir = path.to_path_buf();
        self.refresh();
    }

    pub fn up(&mut self) {
        if self.current_dir.parent().is_none() {
            return;
        }

        self.store_current_as_last();
        self.current_dir.pop();
        self.refresh();
    }

    pub fn back(&mut self) {
        if self.last_dirs.is_empty() {
            return;
        }

        self.store_current_as_next();

        self.current_dir = self.last_dirs.pop().expect("Last dir list is empty");

        self.refresh();
    }

    pub fn next(&mut self) {
        if self.next_dirs.is_empty() {
            return;
        }

        self.store_current_as_last();
        self.current_dir = self.next_dirs.pop().expect("Next dir list is empty");

        self.refresh();
    }

    /// Stores the current directory, for operations which use the "last_dirs" list.
    /// Retains up to MAX_STORED_LAST_DIRS entries.
    fn store_current_as_last(&mut self) {
        self.last_dirs.push(self.current_dir.clone());

        while self.last_dirs.len() > MAX_STORED_DIRS {
            self.last_dirs.remove(0);
        }
    }

    /// Stores the current directory, for operations which use the "next_dirs" list.
    /// Retains up to MAX_STORED_LAST_DIRS entries.
    fn store_current_as_next(&mut self) {
        self.next_dirs.push(self.current_dir.clone());

        while self.next_dirs.len() > MAX_STORED_DIRS {
            self.next_dirs.remove(0);
        }
    }
}
