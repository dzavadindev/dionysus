//! Command-line entry point for the Dionysus application process.

mod ui;

use dionysus::core::{Launcher, desktop};
use gtk4::{Application, gio, prelude::*};
use relm4::RelmApp;
use std::env;
use ui::compose::DModel;

const APPLICATION_ID: &str = "com.dzavadindev.relm4-dionysus";

/// Starts the long-running GTK application instance.
fn run_daemon() {
    let launcher = match Launcher::from_catalog(desktop::load_app_entries(), 10) {
        Ok(launcher) => launcher,
        Err(error) => {
            eprintln!("Failed to initialize launcher: {error}");
            return;
        }
    };

    let application = Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    RelmApp::from_app(application)
        .visible_on_activate(false)
        .run::<DModel>(launcher);
}

/// Activates the already-running application instance.
fn activate_existing() -> bool {
    let application = Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    if application.register(None::<&gio::Cancellable>).is_err() {
        return false;
    }

    if application.is_remote() {
        application.activate();
        true
    } else {
        false
    }
}

/// Prints the available command-line commands.
fn print_help() {
    println!(
        "dionysus - application launcher\n\n\
USAGE:\n    dionysus <COMMAND>\n\n\
COMMANDS:\n    init\n        Start the Dionysus application instance.\n\n\
    toggle\n        Toggle the running Dionysus application.\n\n\
    help\n        Show this help message."
    );
}

fn main() {
    match env::args().nth(1).as_deref() {
        Some("init") => run_daemon(),
        Some("toggle") => {
            if !activate_existing() {
                eprintln!("No running dionysus instance found.");
            }
        }
        Some("help") | None => print_help(),
        Some(_) => print_help(),
    }
}
