use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn load_freq() -> io::Result<HashMap<String, u64>> {
    let path = freq_file_path();
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<HashMap<String, u64>>(&content)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok(parsed)
}

pub fn save_freq(freq: &HashMap<String, u64>) -> io::Result<()> {
    let path = freq_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp_path = path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(freq)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(&tmp_path, content)?;
    fs::rename(tmp_path, path)?;
    Ok(())
}

fn freq_file_path() -> PathBuf {
    state_dir().join("dionysus").join("freq.json")
}

fn state_dir() -> PathBuf {
    if let Some(state_home) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(state_home);
    }

    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".local").join("state");
    }

    PathBuf::from("/tmp")
}
