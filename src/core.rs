//! UI-independent application-launcher functionality.
//!
//! This module contains the data model and operations that a user interface
//! can build on.

pub mod desktop;
pub mod freq_store;
pub mod launcher;
pub mod search;
pub mod watcher;

pub use launcher::{LaunchError, Launcher};

use std::path::PathBuf;

/// A launchable application parsed from a freedesktop `.desktop` file.
#[derive(Clone, Debug)]
pub struct AppEntry {
    /// The desktop-entry identifier.
    pub id: String,
    /// The source `.desktop` file.
    pub desktop_path: PathBuf,
    /// The localized display name.
    pub name: String,
    /// The executable command from the desktop entry.
    pub exec: String,
    /// The icon name or path, when one was provided.
    pub icon: Option<IconRef>,
    /// Whether the application expects to run in a terminal.
    pub terminal: bool,
    /// Whether the desktop entry requested that it not be displayed.
    pub nodisplay: bool,
    /// Whether the desktop entry is hidden.
    pub hidden: bool,
}

/// A desktop-entry icon represented without any UI toolkit types.
#[derive(Debug, Clone)]
pub enum IconRef {
    /// A name to be resolved by the eventual UI or icon theme implementation.
    ThemedName(String),
    /// An explicit icon file path.
    FilePath(PathBuf),
}
