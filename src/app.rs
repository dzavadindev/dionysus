use std::thread;

use gtk4::prelude::*;
use gtk4::Application;

use crate::core;
use crate::ipc;
use crate::ui::window;

fn init_gtk_app() {
    let app = Application::builder()
        .application_id("com.dzavadindev.dionysus")
        .build();

    app.connect_activate(|app| {
        window::build_launcher_window(&app);
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
    let state_populating = state.clone();

    thread::spawn(move || {
        populate_app_entries(state_populating);
    });

    let _ = ipc::start_listener(|command| {
        if command == ipc::TOGGLE_COMMAND {
            // TODO: Wire this to the GTK main thread to show/hide the window.
            println!("toggle requested");
        }
    });

    init_gtk_app();
}
