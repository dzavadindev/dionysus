use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::search::SearchMode;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct LauncherState {
    pub frequencies: HashMap<String, u64>,
    pub mode: SearchMode,
}

/// Loads the frequency and ranking state file.
pub fn load_state() -> io::Result<LauncherState> {
    let path = freq_file_path();
    if !path.exists() {
        return Ok(LauncherState::default());
    }

    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

/// Saves the frequency map and ranking mode using a temporary file and atomic rename.
pub fn save_state(frequencies: &HashMap<String, u64>, mode: SearchMode) -> io::Result<()> {
    #[derive(Serialize)]
    struct StateRef<'a> {
        frequencies: &'a HashMap<String, u64>,
        mode: SearchMode,
    }

    let path = freq_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(&StateRef { frequencies, mode })
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(&tmp_path, content)?;
    fs::rename(tmp_path, path)?;
    Ok(())
}

/// Returns the path of the JSON file of frequencies
fn freq_file_path() -> PathBuf {
    state_dir().join("dionysus").join("freq.json")
}

/// Resolves and returns the state directory
fn state_dir() -> PathBuf {
    if let Some(state_home) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(state_home);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local").join("state");
    }

    PathBuf::from("/tmp")
}
