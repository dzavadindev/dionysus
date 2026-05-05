use freedesktop_desktop_entry as fde;
use gtk4::prelude::*;
use gtk4::{Application, gio, glib};
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;

use crate::core;
use crate::ui;

const APPLICATION_ID: &str = "com.dzavadindev.dionysus";

fn install_styles() {
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(include_str!("../assets/style.css"));

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn populate_app_entries() -> Vec<core::AppEntry> {
    let locales = fde::get_languages_from_env();
    let parsed_files = core::desktop::parse_dot_desktop_files(&locales);

    let mut app_entries: Vec<core::AppEntry> = Vec::new();
    let mut positions: HashMap<String, usize> = HashMap::new();
    for (idx, raw_entry) in parsed_files.iter().enumerate() {
        let app_entry = match core::desktop::desktop_file_to_app_entry(raw_entry, &locales) {
            Some(some) => some,
            None => continue,
        };

        app_entries.push(app_entry.clone());
        positions.insert(app_entry.id, idx);
    }

    app_entries
}

fn build_application(runtime: &core::AppRuntime) -> Application {
    let s = runtime.state.clone();

    let app = Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    app.connect_startup(move |app| {
        install_styles();
        let ui = ui::build_ui(app, s.clone());
        let controller = Rc::new(ui::UiController::new(ui, s.clone()));
        let search_worker = core::search_worker::spawn_search_worker(5);
        controller.attach_search_sender(search_worker.command_tx.clone());
        controller.bind_events();

        // Poll search results on GTK thread and refresh visible entries.
        glib::idle_add_local({
            let controller = controller.clone();
            let result_rx = search_worker.result_rx;
            move || match result_rx.try_recv() {
                Ok(result) => {
                    controller.update_entries(&result.items);
                    glib::ControlFlow::Continue
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            }
        });

        app.connect_activate({
            let controller = controller.clone();
            move |_| {
                controller.toggle_visibility();
            }
        });

        let state_populating = s.clone();
        let search_tx = search_worker.command_tx.clone();
        thread::spawn(move || {
            let apps = populate_app_entries();
            let freq = {
                let mut state = state_populating.lock();
                state.init_apps(apps.clone());
                state.freq.clone()
            };

            let _ = search_tx.send(core::search_worker::SearchCommand::ReloadApps(apps));
            let _ = search_tx.send(core::search_worker::SearchCommand::UpdateFreq(freq));
            let _ = search_tx.send(core::search_worker::SearchCommand::Query(String::new()));
        });
    });

    app
}

pub fn run_daemon() {
    let runtime = core::AppRuntime {
        state: core::SharedState::new(),
    };

    match core::freq_store::load_freq() {
        Ok(freq) => {
            let mut state = runtime.state.lock();
            state.freq = freq;
        }
        Err(err) => {
            eprintln!("Failed to load frequency state: {err}");
        }
    }

    let application = build_application(&runtime);

    if application.register(None::<&gio::Cancellable>).is_err() {
        println!("Failed to register the application [{APPLICATION_ID}]");
        return;
    }

    if application.is_remote() {
        println!("An instance of dionysus is already running");
        return;
    }

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
