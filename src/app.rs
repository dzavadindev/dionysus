use gtk4::prelude::*;
use gtk4::{Application, gio, glib};
use std::collections::HashMap;
use std::thread;

use crate::core;
use crate::ui;

const APPLICATION_ID: &str = "com.dzavadindev.dionysus";

fn populate_app_entries(state: core::SharedState) {
    let file_paths = core::desktop::get_dot_desktop_files();
    let parsed_files = core::desktop::parse_dot_desktop_files(file_paths);

    let mut app_entries: Vec<core::AppEntry> = Vec::new();
    let mut positions: HashMap<String, usize> = HashMap::new();
    for (idx, raw_entry) in parsed_files.iter().enumerate() {
        let app_entry =
            match core::desktop::desktop_file_to_app_entry(&raw_entry.1, raw_entry.0.as_path()) {
                Some(some) => some,
                None => continue,
            };

        app_entries.push(app_entry.clone());
        positions.insert(app_entry.id, idx);
    }

    let mut s = state.lock();
    s.init_apps(app_entries);
}

fn build_application(runtime: &core::AppRuntime) -> Application {
    let s = runtime.state.clone();

    let app = Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    app.connect_startup(move |app| {
        let ui = ui::build_ui(app, s.clone());
        ui.main_window.present();
        ui.selection_model.set_selected(0);
        ui.entries_list.grab_focus();
        ui::UI_HANDLE.with(|cell| {
            *cell.borrow_mut() = Some(ui);
        });
    });

    app.connect_activate(move |_| {
        ui::UI_HANDLE.with(|cell| {
            if let Some(ui) = cell.borrow_mut().as_mut() {
                if ui.main_window.is_visible() {
                    ui.main_window.hide();
                } else {
                    ui.main_window.present();
                    ui.selection_model.set_selected(0);
                    ui.entries_list.grab_focus();
                }
            };
        });
    });

    app
}

pub fn run_daemon() {
    let runtime = core::AppRuntime {
        state: core::SharedState::new(),
    };

    let application = build_application(&runtime);

    if application.register(None::<&gio::Cancellable>).is_err() {
        println!("Failed to register the application [{APPLICATION_ID}]");
        return;
    }

    if application.is_remote() {
        println!("An instance of dionysus is already running");
        return;
    }

    let state_populating = runtime.state.clone();
    let state_for_ui = runtime.state.clone();
    thread::spawn(move || {
        populate_app_entries(state_populating);

        glib::MainContext::default().invoke(move || {
            ui::UI_HANDLE.with(|cell| {
                if let Some(ui_handle) = cell.borrow_mut().as_mut() {
                    let s = state_for_ui.lock();
                    ui::update_entries(&ui_handle, &s.apps);
                }
            })
        });
    });

    application.run();
}

pub fn activate_existing() -> bool {
    let app = Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    if app.register(None::<&gio::Cancellable>).is_err() {
        return false;
    }

    if app.is_remote() {
        app.activate();
        return true;
    }

    false
}
