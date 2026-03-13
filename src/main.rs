mod app;
mod core;
mod ipc;
mod ui;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if let Some(command) = args.get(1) {
        match command.as_str() {
            "init" => {
                app::run_daemon();
            }
            "toggle" => {
                if !app::activate_existing() {
                    println!("No running dionysus instance found.");
                }
            }
            _ => help(),
        }
    } else {
        help();
    }
}

fn help() {
    println!(
        "dionysus - lightweight application launcher

USAGE:
    dionysus <COMMAND>

COMMANDS:
    init
        Start the Dionysus launcher daemon.

        This initializes the background process, loads application
        entries from .desktop files, and prepares the UI. The launcher
        will remain running in the background so that future commands
        respond instantly.

    toggle
        Toggle the launcher window.

        If the launcher UI is hidden, it will be shown.
        If the launcher UI is visible, it will be hidden.

        This command requires that the launcher has already been started
        with `dionysus init`.

    help
        Show this help message.

EXAMPLES:
    Start the launcher daemon:
        dionysus init

    Toggle the launcher window:
        dionysus toggle

    Typical window manager binding:
        bind = SUPER, Space, exec, dionysus toggle"
    );
}
