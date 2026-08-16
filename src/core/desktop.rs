use crate::core;

use freedesktop_desktop_entry as fde;
use std::process::{Command, Stdio};

/// Gets all available desktop files from the standard freedesktop locations.
pub fn parse_dot_desktop_files(locales: &[String]) -> Vec<fde::DesktopEntry> {
    fde::Iter::new(fde::default_paths())
        .entries(Some(locales))
        .collect()
}

/// Discovers and converts visible desktop files into launchable applications.
pub fn load_app_entries() -> Vec<core::AppEntry> {
    let locales = fde::get_languages_from_env();
    parse_dot_desktop_files(&locales)
        .iter()
        .filter_map(|entry| desktop_file_to_app_entry(entry, &locales))
        .collect()
}

/// Converts one freedesktop desktop entry into an application entry.
pub fn desktop_file_to_app_entry(
    entry: &fde::DesktopEntry,
    locales: &[String],
) -> Option<core::AppEntry> {
    let hidden = entry.hidden();
    let nodisplay = entry.no_display();
    if hidden || nodisplay {
        return None;
    }

    let name = entry
        .full_name(locales)
        .or_else(|| entry.name(locales))
        .map(|name| name.into_owned())
        .unwrap_or_else(|| entry.appid.clone());

    let exec = entry.exec()?.to_string();
    let terminal = entry.terminal();
    let id = entry.id().to_string();
    let desktop_path = entry.path.clone();

    let icon = entry
        .icon()
        .map(fde::IconSource::from_unknown)
        .and_then(icon_from_source);

    Some(core::AppEntry {
        id,
        desktop_path,
        name,
        exec,
        icon,
        terminal,
        nodisplay,
        hidden,
    })
}

/// Resolves dionysus native IconRef from an fde IconSource
fn icon_from_source(icon: fde::IconSource) -> Option<core::IconRef> {
    match icon {
        fde::IconSource::Name(name) => Some(core::IconRef::ThemedName(name)),
        fde::IconSource::Path(path) => Some(core::IconRef::FilePath(path)),
    }
}

#[derive(Debug)]
pub enum LaunchError {
    /// The executable command was empty after desktop-entry field removal.
    EmptyCommand,
    /// The operating system rejected the process launch.
    Io(std::io::Error),
}

impl From<std::io::Error> for LaunchError {
    fn from(value: std::io::Error) -> Self {
        LaunchError::Io(value)
    }
}

impl std::fmt::Display for LaunchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCommand => formatter.write_str("desktop entry has an empty command"),
            Self::Io(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for LaunchError {}

/// Gets rid of the template placeholders
///
/// TODO:
///
/// It would make a lot of sense to change the behaviour of this if dionysus
/// is to support anything more then a simple launcher, which is not in the
/// plans. But the decision must be made, and be final. TODO for now
fn strip_exec_fields(exec: &str) -> &str {
    match exec.find('%') {
        Some(idx) => exec[..idx].trim_end(),
        None => exec.trim(),
    }
}

/// Make a list of arguments from an exec string
pub fn exec_to_argv(exec: &str) -> Result<Vec<String>, LaunchError> {
    let cleaned = strip_exec_fields(exec);
    if cleaned.is_empty() {
        return Err(LaunchError::EmptyCommand);
    }

    let argv = cleaned
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    if argv.is_empty() {
        return Err(LaunchError::EmptyCommand);
    }

    Ok(argv)
}

/// Starts an executable command detached from the launcher process.
pub fn launch_exec(exec: &str) -> Result<(), LaunchError> {
    let argv = exec_to_argv(exec)?;
    let (program, args) = argv.split_first().ok_or(LaunchError::EmptyCommand)?;
    Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}
