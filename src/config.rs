//! XDG configuration lookup and a deliberately small INI parser.

use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "dionysus/config.ini";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    pub ui: UiConfig,
    pub colors: ColorsConfig,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiConfig {
    pub h_scale: f64,
    pub v_scale: f64,
    pub item_scale: f64,
    pub font_scale: f64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            h_scale: 1.0,
            v_scale: 1.0,
            item_scale: 1.0,
            font_scale: 1.0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColorsConfig {
    pub primary: Option<Rgb>,
    pub secondary: Option<Rgb>,
    pub tertiary: Option<Rgb>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
pub enum ConfigError {
    Io { path: PathBuf, source: io::Error },
    Parse { path: PathBuf, source: ParseError },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Parse { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
        }
    }
}

#[derive(Debug)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub config: Config,
}

/// Candidate paths in priority order: user configuration, then system directories.
pub fn config_paths() -> Vec<PathBuf> {
    let user_dir = env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .map(|home| home.join(".config"))
        });

    let system_dirs = env::var_os("XDG_CONFIG_DIRS")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "/etc/xdg".into());
    let system_dirs: Vec<_> = env::split_paths(&system_dirs)
        .filter(|dir| dir.is_absolute())
        .collect();
    paths_for_dirs(user_dir.as_deref(), &system_dirs)
}

/// Builds paths from XDG directories in precedence order.
pub fn paths_for_dirs(user_dir: Option<&Path>, system_dirs: &[PathBuf]) -> Vec<PathBuf> {
    user_dir
        .into_iter()
        .chain(system_dirs.iter().map(PathBuf::as_path))
        .map(|dir| dir.join(CONFIG_FILE))
        .collect()
}

/// Reads the first available config, or returns `None` if none exists.
/// An invalid higher-priority file does not fall back to a lower-priority one.
pub fn load() -> Result<Option<LoadedConfig>, ConfigError> {
    load_from_paths(config_paths())
}

/// Loads the first existing path, useful for checking a specific search order.
pub fn load_from_paths(
    paths: impl IntoIterator<Item = PathBuf>,
) -> Result<Option<LoadedConfig>, ConfigError> {
    for path in paths {
        match fs::symlink_metadata(&path) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => return Err(ConfigError::Io { path, source }),
        }
        let text = fs::read_to_string(&path).map_err(|source| ConfigError::Io {
            path: path.clone(),
            source,
        })?;
        let config = parse(&text).map_err(|source| ConfigError::Parse {
            path: path.clone(),
            source,
        })?;
        return Ok(Some(LoadedConfig { path, config }));
    }
    Ok(None)
}

/// Parses supported INI keys. Comments must occupy a whole line.
pub fn parse(input: &str) -> Result<Config, ParseError> {
    let mut config = Config::default();
    let mut section = "";
    let mut seen = std::collections::HashSet::new();
    let mut sections = std::collections::HashSet::new();

    for (index, raw_line) in input.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let error = |message: &str| ParseError {
            line: line_number,
            message: message.to_owned(),
        };
        if line.starts_with('[') {
            let name = line
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
                .ok_or_else(|| error("invalid section header"))?;
            if name != "ui" && name != "colors" {
                return Err(error("unknown section"));
            }
            if !sections.insert(name) {
                return Err(error("duplicate section"));
            }
            section = name;
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| error("expected key=value"))?;
        let (key, value) = (key.trim(), value.trim());
        if !seen.insert((section, key)) {
            return Err(error("duplicate key"));
        }
        match (section, key) {
            ("ui", "h_scale") => config.ui.h_scale = parse_scale(value, line_number)?,
            ("ui", "v_scale") => config.ui.v_scale = parse_scale(value, line_number)?,
            ("ui", "item_scale") => config.ui.item_scale = parse_scale(value, line_number)?,
            ("ui", "font_scale") => config.ui.font_scale = parse_scale(value, line_number)?,
            ("colors", "primary") => config.colors.primary = Some(parse_color(value, line_number)?),
            ("colors", "secondary") => {
                if config.colors.primary.is_none() {
                    return Err(error("secondary requires primary before it"));
                }
                config.colors.secondary = Some(parse_color(value, line_number)?);
            }
            ("colors", "tertiary") => {
                if config.colors.secondary.is_none() {
                    return Err(error("tertiary requires primary and secondary before it"));
                }
                config.colors.tertiary = Some(parse_color(value, line_number)?);
            }
            ("", _) => return Err(error("key outside a section")),
            _ => return Err(error("unknown key")),
        }
    }
    Ok(config)
}

fn parse_scale(value: &str, line: usize) -> Result<f64, ParseError> {
    let scale = value.parse::<f64>().map_err(|_| ParseError {
        line,
        message: "expected a positive decimal scale".to_owned(),
    })?;
    if !scale.is_finite() || scale <= 0.0 {
        return Err(ParseError {
            line,
            message: "scale must be finite and greater than zero".to_owned(),
        });
    }
    Ok(scale)
}

fn parse_color(value: &str, line: usize) -> Result<Rgb, ParseError> {
    let hex = value
        .strip_prefix('#')
        .filter(|hex| hex.len() == 6 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let parsed = hex.and_then(|hex| {
        Some(Rgb {
            red: u8::from_str_radix(&hex[0..2], 16).ok()?,
            green: u8::from_str_radix(&hex[2..4], 16).ok()?,
            blue: u8::from_str_radix(&hex[4..6], 16).ok()?,
        })
    });
    parsed.ok_or_else(|| ParseError {
        line,
        message: "expected a color in #RRGGBB format".to_owned(),
    })
}
