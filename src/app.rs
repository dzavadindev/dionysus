use gtk4::prelude::*;
use gtk4::{Application, gio, glib};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

use crate::core;
use crate::ui;

const APPLICATION_ID: &str = "com.dzavadindev.dionysus";

fn populate_app_entries(state: core::SharedState) -> Vec<core::AppEntry> {
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

    app_entries
}

fn build_application(runtime: &core::AppRuntime) -> Application {
    let s = runtime.state.clone();

    let app = Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    let startup_handle = runtime.ui.clone();
    app.connect_startup(move |app| {
        let ui = ui::build_ui(app, s.clone());
        ui.main_window.present();
        *startup_handle.borrow_mut() = Some(ui);
    });

    let activate_handle = runtime.ui.clone();
    app.connect_activate(move |_| {
        if let Some(ui) = activate_handle.borrow_mut().as_mut() {
            if ui.main_window.is_visible() {
                ui.main_window.hide();
            } else {
                ui.main_window.present();
            }
        };
    });

    app
}

pub fn run_daemon() {
    let runtime = core::AppRuntime {
        state: core::SharedState::new(),
        ui: Rc::new(RefCell::new(None)),
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

    // FIXME: This is an old convention, need to read up on how new glib channels work
    let (sender, receiver) =
        glib::MainContext::channel::<Vec<core::AppEntry>>(glib::PRIORITY_DEFAULT);
    let ui_handle = runtime.ui.clone();
    receiver.attach(None, move |apps| {
        if let Some(ui) = ui_handle.borrow_mut().as_mut() {
            ui::update_entries(ui, &apps);
        }
        glib::ControlFlow::Continue
    });

    let state_populating = runtime.state.clone();
    thread::spawn(move || {
        let apps = populate_app_entries(state_populating);
        let _ = sender.send(apps);
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
