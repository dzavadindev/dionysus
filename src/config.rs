//! XDG configuration lookup and a small INI parser.

use std::collections::{BTreeMap, HashSet};
use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CONFIG_DIR: &str = "dionysus";
const CONFIG_FILE: &str = "config.ini";

/// Validated, merged settings. Omitted keys retain their default values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Config {
    pub ui: UiConfig,
    pub colors: ColorsConfig,
}

/// Winning value plus its origin, retained for override warnings and parse errors.
struct SourcedSetting {
    value: String,
    path: PathBuf,
    line: usize,
}

/// Identity of a supported key, shared by duplicate detection, merging, and
/// typed parsing so those stages refer to the same setting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SettingKey {
    HScale,
    VScale,
    ItemScale,
    FontScale,
    Primary,
    Secondary,
    Tertiary,
}

impl SettingKey {
    /// Resolves an INI section and key to the typed identity shared by the
    /// duplicate tracker, merge map, and value parser.
    fn from_parts(section: &str, name: &str) -> Option<Self> {
        match (section, name) {
            ("ui", "h_scale") => Some(Self::HScale),
            ("ui", "v_scale") => Some(Self::VScale),
            ("ui", "item_scale") => Some(Self::ItemScale),
            ("ui", "font_scale") => Some(Self::FontScale),
            ("colors", "primary") => Some(Self::Primary),
            ("colors", "secondary") => Some(Self::Secondary),
            ("colors", "tertiary") => Some(Self::Tertiary),
            _ => None,
        }
    }
}

/// Independent UI size multipliers; each defaults to `1.0` and must be positive.
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

/// Explicit palette colors, if supplied. Secondary requires primary, and
/// tertiary requires both; unspecified colors remain `None` for later styling.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColorsConfig {
    pub primary: Option<Rgb>,
    pub secondary: Option<Rgb>,
    pub tertiary: Option<Rgb>,
}

/// An RGB color parsed from the six hexadecimal digits of an INI value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl fmt::Display for Rgb {
    /// Prints a color as lowercase `#RRGGBB` for CLI output.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.red, self.green, self.blue)
    }
}

/// Invalid INI syntax or setting value, located by line within one input file.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for ParseError {
    /// Adds the original input line number to a parsing diagnostic.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for ParseError {}

/// A discovery, read, or parse failure with the relevant filesystem path;
/// callers discard the entire configuration and use defaults on this error.
#[derive(Debug)]
pub enum ConfigError {
    /// A configuration directory could not be listed or an INI file read.
    Io { path: PathBuf, source: io::Error },
    /// A scanned or winning setting was invalid in the named file.
    Parse { path: PathBuf, source: ParseError },
}

impl fmt::Display for ConfigError {
    /// Prefixes a filesystem or parser error with its source file path.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::Parse { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for ConfigError {
    /// Exposes the underlying I/O or parse error to error-reporting callers.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
        }
    }
}

/// Successful layered load: the effective configuration, its source files,
/// and nonfatal warnings describing values overridden during the merge.
#[derive(Debug)]
pub struct LoadedConfig {
    /// Files read, in descending priority order.
    pub paths: Vec<PathBuf>,
    /// Parsed values after applying all overrides and defaults.
    pub config: Config,
    /// Duplicate-key messages with old and replacement values and their origins.
    pub warnings: Vec<String>,
}

/// One borrowed assignment emitted by `scan`; keeps the original spelling
/// for warnings and its typed key for later merging or validation.
struct Setting<'a> {
    section: &'a str,
    name: &'a str,
    key: SettingKey,
    value: &'a str,
    line: usize,
}

// -----------------------------------------------------------------------

/// Resolves XDG environment variables (and their defaults), then discovers
/// `.ini` files in their `dionysus` subdirectories in descending priority order.
pub fn config_paths() -> Result<Vec<PathBuf>, ConfigError> {
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

/// Lists files in each directory and groups them by priority: user `config.ini`,
/// user other `.ini`, system `config.ini`, then system other `.ini` files.
/// System directory order and alphabetical filenames break ties; repeated paths
/// are included once. Missing directories contribute no files.
pub fn paths_for_dirs(
    user_dir: Option<&Path>,
    system_dirs: &[PathBuf],
) -> Result<Vec<PathBuf>, ConfigError> {
    let mut paths = Vec::new();
    if let Some(dir) = user_dir {
        let (config, others) = ini_files(dir)?;
        paths.extend(config);
        paths.extend(others);
    }
    let mut system_configs = Vec::new();
    let mut system_others = Vec::new();
    for dir in system_dirs {
        let (config, others) = ini_files(dir)?;
        system_configs.extend(config);
        system_others.extend(others);
    }
    paths.extend(system_configs);
    paths.extend(system_others);
    let mut seen = HashSet::new();
    paths.retain(|path| seen.insert(path.clone()));
    Ok(paths)
}

/// Separates `config.ini` from alphabetically sorted other `.ini` files in one
/// `dionysus` directory; ignores non-INI entries and absent directories.
fn ini_files(dir: &Path) -> Result<(Option<PathBuf>, Vec<PathBuf>), ConfigError> {
    let dir = dir.join(CONFIG_DIR);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok((None, Vec::new())),
        Err(source) => return Err(ConfigError::Io { path: dir, source }),
    };
    let mut config = None;
    let mut others = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ConfigError::Io {
            path: dir.clone(),
            source,
        })?;
        let path = entry.path();
        if path.extension() == Some(OsStr::new("ini")) {
            if path.file_name() == Some(OsStr::new(CONFIG_FILE)) {
                config = Some(path);
            } else {
                others.push(path);
            }
        }
    }
    others.sort();
    Ok((config, others))
}

/// Discovers XDG files and loads their merged configuration, or returns `None`
/// if none exist. A malformed or unreadable file rejects the entire merge.
pub fn load() -> Result<Option<LoadedConfig>, ConfigError> {
    load_from_paths(config_paths()?)
}

/// Scans files ordered from highest to lowest priority, merges their keys, and
/// parses only the winning values. Files are read in reverse order so later
/// inserts override lower-priority values and generate warnings with both sources.
/// Missing files are skipped; syntax and I/O errors identify the offending file.
pub fn load_from_paths(
    paths: impl IntoIterator<Item = PathBuf>,
) -> Result<Option<LoadedConfig>, ConfigError> {
    let mut files = Vec::new();
    for path in paths {
        match fs::symlink_metadata(&path) {
            Ok(_) => files.push(path),
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => return Err(ConfigError::Io { path, source }),
        }
    }
    if files.is_empty() {
        return Ok(None);
    }

    let mut merged: BTreeMap<SettingKey, SourcedSetting> = BTreeMap::new();
    let mut warnings = Vec::new();
    for path in files.iter().rev() {
        let text = fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.clone(),
            source,
        })?;
        for setting in scan(&text).map_err(|source| ConfigError::Parse {
            path: path.clone(),
            source,
        })? {
            let new = SourcedSetting {
                value: setting.value.to_owned(),
                path: path.clone(),
                line: setting.line,
            };
            if let Some(old) = merged.insert(setting.key, new) {
                warnings.push(format!(
                    "duplicate [{}] {}: {}:{} value '{}' overwritten by {}:{} value '{}'",
                    setting.section,
                    setting.name,
                    old.path.display(),
                    old.line,
                    old.value,
                    path.display(),
                    setting.line,
                    setting.value
                ));
            }
        }
    }

    // Only winning values are parsed: an overridden invalid value is not applied.
    // Color prerequisites are checked after parsing, regardless of file or line order.
    let config = parse_values(
        merged
            .iter()
            .map(|(&key, setting)| (key, setting.value.as_str(), setting.line)),
    )
    .map_err(|(key, source)| ConfigError::Parse {
        path: merged[&key].path.clone(),
        source,
    })?;
    Ok(Some(LoadedConfig {
        paths: files,
        config,
        warnings,
    }))
}

/// Parses a single INI string: scan assignments, reject duplicate keys, then
/// parse values and check color prerequisites. Comments occupy whole lines;
/// colors need their prerequisites but may appear in any textual order.
pub fn parse(input: &str) -> Result<Config, ParseError> {
    let settings = scan(input)?;
    let mut seen = HashSet::new();
    for setting in &settings {
        if !seen.insert(setting.key) {
            return Err(ParseError {
                line: setting.line,
                message: "duplicate key".to_owned(),
            });
        }
    }
    parse_values(
        settings
            .iter()
            .map(|setting| (setting.key, setting.value, setting.line)),
    )
    .map_err(|(_, source)| source)
}

/// Collects ordered assignments with their source lines,
/// checking INI syntax known sections, and known keys
fn scan(input: &str) -> Result<Vec<Setting<'_>>, ParseError> {
    let mut settings = Vec::new();
    let mut section = "";
    let mut sections = HashSet::new();

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
        let (name, value) = line
            .split_once('=')
            .ok_or_else(|| error("expected key=value"))?;
        let (name, value) = (name.trim(), value.trim());
        if section.is_empty() {
            return Err(error("key outside a section"));
        }
        let key = SettingKey::from_parts(section, name).ok_or_else(|| error("unknown key"))?;
        settings.push(Setting {
            section,
            name,
            key,
            value,
            line: line_number,
        });
    }
    Ok(settings)
}

/// Converts assignments to typed config values, then checks that secondary
/// and tertiary colors have their prerequisites regardless of input order.
/// Returns the offending key so the loader can recover its source file path.
fn parse_values<'a>(
    settings: impl IntoIterator<Item = (SettingKey, &'a str, usize)>,
) -> Result<Config, (SettingKey, ParseError)> {
    let mut config = Config::default();
    let mut secondary_line = None;
    let mut tertiary_line = None;
    for (key, value, line) in settings {
        apply_setting(&mut config, key, value, line).map_err(|error| (key, error))?;
        match key {
            SettingKey::Secondary => secondary_line = Some(line),
            SettingKey::Tertiary => tertiary_line = Some(line),
            _ => {}
        }
    }
    if let Some(line) = secondary_line
        && config.colors.primary.is_none()
    {
        return Err((
            SettingKey::Secondary,
            ParseError {
                line,
                message: "secondary requires primary".to_owned(),
            },
        ));
    }
    if let Some(line) = tertiary_line
        && config.colors.secondary.is_none()
    {
        return Err((
            SettingKey::Tertiary,
            ParseError {
                line,
                message: "tertiary requires primary and secondary".to_owned(),
            },
        ));
    }
    Ok(config)
}

/// Parses one recognized value into its `Config` field, retaining its source
/// line for errors; cross-field color dependencies are checked afterward.
fn apply_setting(
    config: &mut Config,
    key: SettingKey,
    value: &str,
    line: usize,
) -> Result<(), ParseError> {
    match key {
        SettingKey::HScale => config.ui.h_scale = parse_scale(value, line)?,
        SettingKey::VScale => config.ui.v_scale = parse_scale(value, line)?,
        SettingKey::ItemScale => config.ui.item_scale = parse_scale(value, line)?,
        SettingKey::FontScale => config.ui.font_scale = parse_scale(value, line)?,
        SettingKey::Primary => config.colors.primary = Some(parse_color(value, line)?),
        SettingKey::Secondary => config.colors.secondary = Some(parse_color(value, line)?),
        SettingKey::Tertiary => config.colors.tertiary = Some(parse_color(value, line)?),
    }
    Ok(())
}

/// Accepts only finite positive scale factors, rejecting zero, NaN, and infinity.
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

/// Parses `#RRGGBB` without inline-comment handling; validates ASCII hex
/// before byte slicing so malformed Unicode input cannot panic.
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
