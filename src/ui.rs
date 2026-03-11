mod row;
mod window;

use crate::core;
use gtk4::prelude::*;

#[derive(Clone)]
pub struct UiHandle {
    pub main_window: gtk4::ApplicationWindow,
    pub scrolled_window: gtk4::ScrolledWindow,
    pub entries_list: gtk4::ListBox,
}

pub fn build_ui(app: &gtk4::Application, state: core::SharedState) -> UiHandle {
    let window = window::build_main_window(app);

    // Build the apps list widget
    let list = gtk4::ListBox::new();
    list.set_vexpand(true);

    // Fill it with rows
    let apps = {
        let s = state.lock();
        s.apps.clone()
    };

    for entry in apps {
        let row_widget = row::build_row(entry);
        list.append(&row_widget);
    }

    // Build the scrolling view here
    let scrolled_list = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .hexpand(true)
        .build();

    // Set children appropriately
    scrolled_list.set_child(Some(&list));
    window.set_child(Some(&scrolled_list));

    UiHandle {
        main_window: window,
        scrolled_window: scrolled_list,
        entries_list: list,
    }
}
