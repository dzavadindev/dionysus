use gtk4::Application;
use gtk4::prelude::*;

use crate::core;
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

fn init_state() {
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

    for entry in app_entries.iter() {
        print!("{:?}\n\n", entry);
    }
}

pub fn start(_state: core::SharedState) {
    // init_gtk_app();
    init_state();
}
