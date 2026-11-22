//! Filesystem watching for freedesktop application directories.

use freedesktop_desktop_entry as fde;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

/// Starts watching the standard freedesktop application directories.
///
/// The callback receives every event produced by the watcher, including
/// watcher errors. The returned watcher must be kept alive for monitoring to
/// continue; dropping it stops filesystem monitoring.
pub fn init_watcher<F>(callback: F) -> notify::Result<RecommendedWatcher>
where
    F: FnMut(notify::Result<Event>) + Send + 'static,
{
    let mut watcher = notify::recommended_watcher(callback)?;

    for path in fde::default_paths() {
        // XDG application directories are optional; a missing one must not
        // prevent monitoring the directories that do exist.
        if path.is_dir() {
            watcher.watch(&path, RecursiveMode::NonRecursive)?;
        }
    }

    Ok(watcher)
}
