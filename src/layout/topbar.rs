use std::{
    cell::RefCell,
    io::Error,
    path::{Component, Path, PathBuf},
    rc::{Rc, Weak},
};

use gtk4::{
    Box, Button, CenterBox, SearchEntry,
    prelude::{BoxExt, ButtonExt, WidgetExt},
};

use crate::model::Browser;

pub struct TopBar {
    pub content: CenterBox,
    pub navigation_box: Box,
    pub location_box: Box,
    pub browser: RefCell<Weak<RefCell<Browser>>>,
}

impl TopBar {
    pub fn initialize(&self, browser: &Rc<RefCell<Browser>>) -> Result<(), Error> {
        *self.browser.borrow_mut() = Rc::downgrade(browser);
        self.content.set_start_widget(Some(&self.navigation_box));
        self.content.set_center_widget(Some(&self.location_box));

        // Back, forward, up, and refresh
        self.build_navigation_buttons(browser);

        // Current location buttons
        self.build_location_buttons(&browser.borrow().current_dir)?;

        // Search bar
        let search_bar = SearchEntry::new();
        self.content.set_end_widget(Some(&search_bar));

        Ok(())
    }

    pub fn rebuild(&self, browser: &Browser) -> Result<(), Error> {
        self.clear();
        self.build_location_buttons(&browser.current_dir)
    }

    fn clear(&self) {
        while let Some(child) = self.location_box.first_child() {
            self.location_box.remove(&child);
        }
    }

    fn build_location_buttons(&self, path: &Path) -> Result<(), Error> {
        let mut target_path = PathBuf::new();
        for component in path.components() {
            target_path.push(component.as_os_str());
            let button = match component {
                Component::RootDir => Some(Button::with_label("/")),
                Component::Normal(os_str) => Some(Button::with_label(&os_str.to_string_lossy())),
                Component::Prefix(_) => todo!("Windows components are not supported at this time."),
                _ => None,
            };

            if button.is_none() {
                println!("Failed to build location buttons");
                break;
            }

            if let Some(button) = button {
                let target_path = target_path.clone();
                let browser = self.browser.borrow().clone();
                button.connect_clicked(move |_| {
                    if let Some(browser) = browser.upgrade() {
                        browser.borrow_mut().move_into(&target_path);
                    }
                });
                self.location_box.append(&button);
            }
        }

        Ok(())
    }

    fn build_navigation_buttons(&self, browser: &Rc<RefCell<Browser>>) {
        let back_button = Button::from_icon_name("go-previous-symbolic");
        let forward_button = Button::from_icon_name("go-next-symbolic");
        let up_button = Button::from_icon_name("go-up-symbolic");
        let refresh_button = Button::from_icon_name("view-refresh-symbolic");

        // Actions
        let refresh_browser = browser.clone();
        refresh_button.connect_clicked(move |_| refresh_browser.borrow().refresh());

        let up_browser = browser.clone();
        up_button.connect_clicked(move |_| up_browser.borrow_mut().up());

        let back_browser = browser.clone();
        back_button.connect_clicked(move |_| back_browser.borrow_mut().back());

        let next_browser = browser.clone();
        forward_button.connect_clicked(move |_| next_browser.borrow_mut().next());

        // Placement
        self.navigation_box.append(&back_button);
        self.navigation_box.append(&forward_button);
        self.navigation_box.append(&up_button);
        self.navigation_box.append(&refresh_button);
    }
}
