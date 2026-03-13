use gtk4::Application;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

use crate::core;
use crate::ipc;
use crate::ui;

fn init_gtk_app(state: core::SharedState, ui_handle: Rc<RefCell<Option<ui::UiHandle>>>) {
    let app = Application::builder()
        .application_id("com.dzavadindev.dionysus")
        .build();

    let s = state.clone();

    let h = ui_handle.clone();
    app.connect_activate(move |app| {
        let ui = ui::build_ui(app, s.clone());
        ui.main_window.present();
        *h.borrow_mut() = Some(ui);
    });

    app.run();
}

fn populate_app_entries(state: core::SharedState) {
    let file_paths = core::desktop::get_dot_desktop_files();
    let parsed_files = core::desktop::parse_dot_desktop_files(&file_paths);

    let mut app_entries: Vec<core::AppEntry> = Vec::new();
    for raw_entry in parsed_files.iter() {
        let app_entry =
            match core::desktop::desktop_file_to_app_entry(&raw_entry.1, raw_entry.0.as_path()) {
                Some(some) => some,
                None => continue,
            };

        app_entries.push(app_entry);
    }

    {
        let mut s = state.lock();
        s.init_apps(&app_entries);
    }
}

pub fn start(state: core::SharedState) {
    let runtime = core::AppRuntime {
        state,
        ui: Rc::new(RefCell::new(None)),
    };

    let state_populating = runtime.state.clone();
    thread::spawn(move || {
        populate_app_entries(state_populating);
    });

    init_gtk_app(runtime.state.clone(), runtime.ui.clone());
}
