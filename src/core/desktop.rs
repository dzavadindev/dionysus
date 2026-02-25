use freedesktop_file_parser as ffp;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_desktop_files(dir: &Path, seen: &mut HashSet<PathBuf>, results: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.is_dir() {
            collect_desktop_files(&path, seen, results);
            continue;
        }

        if path.extension() == Some(std::ffi::OsStr::new("desktop")) {
            if seen.insert(path.clone()) {
                results.push(path);
            }
        }
    }
}

pub fn get_dot_desktop_files() -> Vec<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new();
    let mut data_dirs = xdg_dirs.get_data_dirs();
    if let Some(data_home) = xdg_dirs.get_data_home() {
        if !data_dirs.contains(&data_home) {
            data_dirs.push(data_home);
        }
    }

    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for data_dir in data_dirs {
        let applications_dir = data_dir.join("applications");
        collect_desktop_files(&applications_dir, &mut seen, &mut results);
    }

    results
}

pub fn parse_dot_desktop_files(files: &Vec<PathBuf>) -> Vec<ffp::DesktopFile> {
    let mut entries: Vec<ffp::DesktopFile> = Vec::new();

    for file in files.iter() {
        let content = match fs::read_to_string(file) {
            Ok(ok) => ok,
            Err(_err) => {
                // TODO: Non-existent file, invalid format, whatnot?
                continue;
            }
        };

        let content = match freedesktop_file_parser::parse(&content) {
            Ok(ok) => ok,
            Err(_err) => {
                // TODO: If failed to parse the file, exclude it from the state. Update the warnings
                continue;
            }
        };

        entries.push(content);
    }

    entries
}
