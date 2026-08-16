//! Command-line entry point for the Dionysus application process.

mod ui;

use gtk4::{Application, gio, prelude::*};
use std::env;

const APPLICATION_ID: &str = "com.dzavadindev.dionysus";

/// Starts the long-running GTK application instance.
fn run_daemon() {
    let application = Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    if application.register(None::<&gio::Cancellable>).is_err() {
        eprintln!("Failed to register the application [{APPLICATION_ID}]");
        return;
    }

    if application.is_remote() {
        eprintln!("An instance of dionysus is already running");
        return;
    }

    // The Relm4 root component will be started here once the UI is restored.
    application.run();
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
