mod ui;

use dionysus::{
    config::{self, Config, Rgb},
    core::{Launcher, desktop},
};
use relm4::{
    RelmApp,
    gtk::{Application, gio, prelude::*},
};
use std::env;
use ui::window::WindowModel;

const APPLICATION_ID: &str = "com.dzavadindev.relm4-dionysus";

/// Starts the long-running GTK application instance.
fn run_daemon() {
    // Appearance settings will be consumed by the UI in the scaling/color patches.
    let _config = match config::load() {
        Ok(Some(loaded)) => {
            for warning in &loaded.warnings {
                eprintln!("Config warning: {warning}");
            }
            loaded.config
        }
        Ok(None) => Config::default(),
        Err(error) => {
            eprintln!("Invalid config: {error}. Config not applied; using defaults.");
            Config::default()
        }
    };

    let launcher = match Launcher::from_catalog(desktop::load_app_entries(), 15) {
        Ok(launcher) => launcher,
        Err(error) => {
            eprintln!("Failed to initialize launcher: {0}", error);
            return;
        }
    };

    let application = Application::builder()
        .application_id(APPLICATION_ID)
        .build();

    RelmApp::from_app(application)
        .visible_on_activate(false)
        // `init` is a Dionysus command, not a file for GApplication to open.
        .with_args(vec![
            env::args().next().unwrap_or_else(|| "dionysus".to_owned()),
        ])
        .run::<WindowModel>(launcher);
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
        application.activate_action("toggle", None);
        true
    } else {
        false
    }
}

fn print_config() {
    let (config, source) = match config::load() {
        Ok(Some(loaded)) => {
            for warning in &loaded.warnings {
                eprintln!("Config warning: {warning}");
            }
            let sources = loaded
                .paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            (loaded.config, sources)
        }
        Ok(None) => (Config::default(), "no config file (defaults)".to_owned()),
        Err(error) => {
            eprintln!("Invalid config: {error}. Config not applied; using defaults.");
            (Config::default(), "defaults (invalid config)".to_owned())
        }
    };
    println!("Config source: {source}");
    println!(
        "[ui] h_scale={} v_scale={} item_scale={} font_scale={}",
        config.ui.h_scale, config.ui.v_scale, config.ui.item_scale, config.ui.font_scale
    );
    let show_color =
        |color: Option<Rgb>| color.map_or_else(|| "default".to_owned(), |color| color.to_string());
    println!(
        "[colors] primary={} secondary={} tertiary={}",
        show_color(config.colors.primary),
        show_color(config.colors.secondary),
        show_color(config.colors.tertiary)
    );
    println!("Appearance values are parsed but not yet applied to the UI.");
}

/// Prints the available command-line commands.
fn print_help() {
    println!(
        "dionysus - application launcher\n\n\
USAGE:\n    dionysus <COMMAND>\n\n\
COMMANDS:\n    init\n        Start the Dionysus application instance.\n\n\
    toggle\n        Toggle the running Dionysus application.\n\n\
    config\n        Validate and print the merged .ini configuration or defaults.\n\n\
    help, --help\n        Show this help message.\n\n\
CONFIG FILES:\n    All .ini files in $XDG_CONFIG_HOME/dionysus (default: ~/.config/dionysus),\n    then $XDG_CONFIG_DIRS/dionysus (default: /etc/xdg/dionysus).\n    Priority: user config.ini, user other .ini files, system config.ini files,\n    system other .ini files. System directories follow XDG_CONFIG_DIRS order;\n    other files are ordered by filename. Duplicate values produce warnings.\n\n\
[ui] h_scale, v_scale, item_scale, font_scale: positive decimal values (default: 1).\n\
[colors] primary, secondary, tertiary: #RRGGBB in any order.\n    Secondary requires primary; tertiary requires both."
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
        Some("config") => print_config(),
        Some("help") | Some("--help") | None => print_help(),
        Some(_) => print_help(),
    }
}
