use dionysus::config::{self, Config, ConfigError, Rgb};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "dionysus-config-test-{}-{}",
            std::process::id(),
            NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn empty_config_uses_all_defaults() {
    assert_eq!(
        config::parse("\n# comment\n; comment\n[ui]\n[colors]\n").unwrap(),
        Config::default()
    );
}

#[test]
fn parses_sample_color_and_scales() {
    let parsed = config::parse("[colors]\nprimary=#fff000\nsecondary = #123ABC\ntertiary=#000000\n[ui]\nh_scale=0.75\nv_scale=1.25\nitem_scale=2\nfont_scale=1.5\n").unwrap();
    assert_eq!(
        parsed.colors.primary,
        Some(Rgb {
            red: 255,
            green: 240,
            blue: 0
        })
    );
    assert_eq!(
        parsed.colors.secondary,
        Some(Rgb {
            red: 18,
            green: 58,
            blue: 188
        })
    );
    assert_eq!(
        parsed.colors.tertiary,
        Some(Rgb {
            red: 0,
            green: 0,
            blue: 0
        })
    );
    assert_eq!(parsed.ui.h_scale, 0.75);
    assert_eq!(parsed.ui.v_scale, 1.25);
    assert_eq!(parsed.ui.item_scale, 2.0);
    assert_eq!(parsed.ui.font_scale, 1.5);
}

#[test]
fn rejects_invalid_scales_and_colors_with_line_numbers() {
    for value in ["0", "-1", "NaN", "inf", "abc"] {
        let input = format!("[ui]\nfont_scale={value}");
        assert_eq!(config::parse(&input).unwrap_err().line, 2, "{value}");
    }
    for value in ["#fff", "#zz0000", "#ffffff #comment", "#é0000", "red"] {
        let input = format!("[colors]\nprimary={value}");
        assert_eq!(config::parse(&input).unwrap_err().line, 2, "{value}");
    }
}

#[test]
fn colors_must_appear_in_priority_order() {
    for input in [
        "[colors]\nsecondary=#000000",
        "[colors]\ntertiary=#000000",
        "[colors]\nprimary=#ffffff\ntertiary=#000000",
        "[colors]\nsecondary=#000000\nprimary=#ffffff",
    ] {
        assert!(config::parse(input).is_err(), "{input}");
    }
    assert!(config::parse("[colors]\nprimary=#ffffff\nsecondary=#000000").is_ok());
}

#[test]
fn rejects_unknown_duplicate_and_unsectioned_keys() {
    for input in [
        "h_scale=2",
        "[unknown]\nkey=value",
        "[ui]\nunknown=2",
        "[ui]\nh_scale=2\nh_scale=3",
        "[ui]\n[ui]",
        "[colors]\nprimary=#ffffff\n[colors]",
        "[ui",
        "[ui]\nh_scale",
    ] {
        assert!(config::parse(input).is_err(), "{input}");
    }
}

#[test]
fn xdg_candidates_put_user_before_system_directories() {
    let root = TempDir::new();
    let user = root.path().join("user");
    let system_a = root.path().join("system-a");
    let system_b = root.path().join("system-b");
    let paths = config::paths_for_dirs(Some(&user), &[system_a.clone(), system_b.clone()]);
    assert_eq!(
        paths,
        vec![
            user.join("dionysus/config.ini"),
            system_a.join("dionysus/config.ini"),
            system_b.join("dionysus/config.ini"),
        ]
    );
    assert!(config::load_from_paths(paths).unwrap().is_none());
}

#[test]
fn loads_first_existing_file_without_merging() {
    let root = TempDir::new();
    let user = root.write("user/dionysus/config.ini", "[colors]\nprimary=#fff000");
    let system = root.write("system/dionysus/config.ini", "[ui]\nh_scale=2");
    let loaded = config::load_from_paths([user.clone(), system.clone()])
        .unwrap()
        .unwrap();
    assert_eq!(loaded.path, user);
    assert_eq!(
        loaded.config.colors.primary,
        Some(Rgb {
            red: 255,
            green: 240,
            blue: 0
        })
    );
    assert_eq!(loaded.config.ui, Config::default().ui);

    let loaded = config::load_from_paths([root.path().join("missing"), system.clone()])
        .unwrap()
        .unwrap();
    assert_eq!(loaded.path, system);
    assert_eq!(loaded.config.ui.h_scale, 2.0);
}

#[test]
fn config_command_warns_and_uses_defaults_for_invalid_user_config() {
    let root = TempDir::new();
    let invalid = root.write("user/dionysus/config.ini", "[ui]\nh_scale=0");
    root.write("system/dionysus/config.ini", "[colors]\nprimary=#fff000");
    let output = Command::new(env!("CARGO_BIN_EXE_dionysus"))
        .arg("config")
        .env("XDG_CONFIG_HOME", root.path().join("user"))
        .env("XDG_CONFIG_DIRS", root.path().join("system"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(&format!("{}: line 2:", invalid.display())));
    assert!(stdout.contains("Config source: defaults (invalid config)"));
    assert!(stdout.contains("h_scale=1"));
    assert!(stdout.contains("primary=default"));
}

#[test]
fn invalid_or_unreadable_first_file_does_not_try_lower_priority() {
    let root = TempDir::new();
    let invalid = root.write("user/dionysus/config.ini", "[ui]\nh_scale=0");
    let system = root.write("system/dionysus/config.ini", "[ui]\nh_scale=2");
    match config::load_from_paths([invalid.clone(), system.clone()]).unwrap_err() {
        ConfigError::Parse { path, source } => {
            assert_eq!(path, invalid);
            assert_eq!(source.line, 2);
        }
        other => panic!("expected parse error, got {other}"),
    }
    let unreadable = root.path().join("directory-instead-of-file");
    fs::create_dir(&unreadable).unwrap();
    match config::load_from_paths([unreadable.clone(), system]).unwrap_err() {
        ConfigError::Io { path, .. } => assert_eq!(path, unreadable),
        other => panic!("expected I/O error, got {other}"),
    }
}
