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
fn standalone_parse_reports_duplicate_key_on_second_line() {
    let error = config::parse("[ui]\nh_scale=1\nh_scale=2").unwrap_err();
    assert_eq!(error.line, 3);
    assert_eq!(error.message, "duplicate key");
}

#[test]
fn colors_require_prerequisites_but_not_textual_order() {
    for (input, line) in [
        ("[colors]\nsecondary=#000000", 2),
        ("[colors]\ntertiary=#000000", 2),
        ("[colors]\nprimary=#ffffff\ntertiary=#000000", 3),
    ] {
        assert_eq!(config::parse(input).unwrap_err().line, line, "{input}");
    }
    let parsed =
        config::parse("[colors]\ntertiary=#000000\nsecondary=#123456\nprimary=#ffffff").unwrap();
    assert_eq!(
        parsed.colors.primary,
        Some(Rgb {
            red: 255,
            green: 255,
            blue: 255
        })
    );
    assert_eq!(
        parsed.colors.secondary,
        Some(Rgb {
            red: 18,
            green: 52,
            blue: 86
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
    root.write("user/dionysus/z.ini", "[ui]\nh_scale=2");
    root.write("user/dionysus/a.ini", "[ui]\nv_scale=2");
    root.write("user/dionysus/config.ini", "[ui]\nitem_scale=2");
    root.write("user/dionysus/ignored.txt", "not ini");
    root.write("system-a/dionysus/z.ini", "[ui]\nh_scale=3");
    root.write("system-a/dionysus/config.ini", "[ui]\nfont_scale=2");
    root.write("system-b/dionysus/a.ini", "[ui]\nh_scale=4");
    root.write("system-b/dionysus/config.ini", "[ui]\nfont_scale=3");
    let paths = config::paths_for_dirs(Some(&user), &[system_a.clone(), system_b.clone()]).unwrap();
    assert_eq!(
        paths,
        vec![
            user.join("dionysus/config.ini"),
            user.join("dionysus/a.ini"),
            user.join("dionysus/z.ini"),
            system_a.join("dionysus/config.ini"),
            system_b.join("dionysus/config.ini"),
            system_a.join("dionysus/z.ini"),
            system_b.join("dionysus/a.ini"),
        ]
    );
}

#[test]
fn merges_all_existing_files_and_warns_about_overrides() {
    let root = TempDir::new();
    let user = root.write("user/dionysus/config.ini", "[ui]\nh_scale=3");
    let other = root.write("user/dionysus/other.ini", "[ui]\nh_scale=2\nitem_scale=1.5");
    let system = root.write(
        "system/dionysus/config.ini",
        "[ui]\nh_scale=1.25\nfont_scale=2",
    );
    let loaded = config::load_from_paths([user.clone(), other.clone(), system.clone()])
        .unwrap()
        .unwrap();
    assert_eq!(
        loaded.paths,
        vec![user.clone(), other.clone(), system.clone()]
    );
    assert_eq!(loaded.config.ui.h_scale, 3.0);
    assert_eq!(loaded.config.ui.item_scale, 1.5);
    assert_eq!(loaded.config.ui.font_scale, 2.0);
    assert_eq!(loaded.warnings.len(), 2);
    assert!(loaded.warnings[0].contains("value '1.25' overwritten by"));
    assert!(loaded.warnings[0].contains(&other.display().to_string()));
    assert!(loaded.warnings[1].contains("value '2' overwritten by"));
    assert!(loaded.warnings[1].contains(&user.display().to_string()));

    let loaded = config::load_from_paths([root.path().join("missing"), system.clone()])
        .unwrap()
        .unwrap();
    assert_eq!(loaded.paths, vec![system]);
    assert_eq!(loaded.config.ui.h_scale, 1.25);
    assert!(
        config::load_from_paths([root.path().join("missing")])
            .unwrap()
            .is_none()
    );
}

#[test]
fn merges_colors_across_files_before_validating_dependencies() {
    let root = TempDir::new();
    let user = root.write(
        "user/dionysus/colors.ini",
        "[colors]\nsecondary=#000000\ntertiary=#123ABC",
    );
    let system = root.write("system/dionysus/config.ini", "[colors]\nprimary=#fff000");
    let loaded = config::load_from_paths([user, system]).unwrap().unwrap();
    assert_eq!(
        loaded.config.colors.primary,
        Some(Rgb {
            red: 255,
            green: 240,
            blue: 0
        })
    );
    assert_eq!(
        loaded.config.colors.secondary,
        Some(Rgb {
            red: 0,
            green: 0,
            blue: 0
        })
    );
    assert_eq!(
        loaded.config.colors.tertiary,
        Some(Rgb {
            red: 18,
            green: 58,
            blue: 188
        })
    );
}

#[test]
fn missing_color_prerequisite_reports_winning_file_and_line() {
    let root = TempDir::new();
    let user = root.write("user/dionysus/colors.ini", "[colors]\ntertiary=#123ABC");
    let system = root.write("system/dionysus/config.ini", "[ui]\nh_scale=2");
    match config::load_from_paths([user.clone(), system]).unwrap_err() {
        ConfigError::Parse { path, source } => {
            assert_eq!(path, user);
            assert_eq!(source.line, 2);
            assert!(source.message.contains("requires primary and secondary"));
        }
        other => panic!("expected color dependency error, got {other}"),
    }
}

#[test]
fn duplicate_keys_within_one_file_warn_and_last_value_wins() {
    let root = TempDir::new();
    let file = root.write("user/dionysus/config.ini", "[ui]\nh_scale=1\nh_scale=2");
    let loaded = config::load_from_paths([file.clone()]).unwrap().unwrap();
    assert_eq!(loaded.config.ui.h_scale, 2.0);
    assert_eq!(loaded.warnings.len(), 1);
    assert!(loaded.warnings[0].contains(&format!(
        "{}:2 value '1' overwritten by {}:3 value '2'",
        file.display(),
        file.display()
    )));
}

#[test]
fn replaced_invalid_value_is_not_applied_but_malformed_file_still_fails() {
    let root = TempDir::new();
    let user = root.write("user/dionysus/config.ini", "[ui]\nh_scale=2");
    let system = root.write("system/dionysus/config.ini", "[ui]\nh_scale=invalid");
    let loaded = config::load_from_paths([user.clone(), system])
        .unwrap()
        .unwrap();
    assert_eq!(loaded.config.ui.h_scale, 2.0);
    let malformed = root.write("system/dionysus/bad.ini", "[bad]\nkey=value");
    assert!(matches!(
        config::load_from_paths([user, malformed]),
        Err(ConfigError::Parse { .. })
    ));
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
fn config_command_reports_merged_values_and_duplicate_sources() {
    let root = TempDir::new();
    let user = root.write("user/dionysus/config.ini", "[colors]\nprimary=#fff000");
    let other = root.write("user/dionysus/theme.ini", "[colors]\nprimary=#ff0000");
    let system = root.write(
        "system/dionysus/config.ini",
        "[colors]\nprimary=#0000ff\nsecondary=#ffffff",
    );
    let output = Command::new(env!("CARGO_BIN_EXE_dionysus"))
        .arg("config")
        .env("XDG_CONFIG_HOME", root.path().join("user"))
        .env("XDG_CONFIG_DIRS", root.path().join("system"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stdout.contains("primary=#fff000 secondary=#ffffff"));
    assert!(stdout.contains(&user.display().to_string()));
    assert!(stdout.contains(&other.display().to_string()));
    assert!(stdout.contains(&system.display().to_string()));
    assert!(stderr.contains(&format!(
        "{}:2 value '#0000ff' overwritten by {}:2 value '#ff0000'",
        system.display(),
        other.display()
    )));
    assert!(stderr.contains(&format!(
        "{}:2 value '#ff0000' overwritten by {}:2 value '#fff000'",
        other.display(),
        user.display()
    )));
}

#[test]
fn invalid_or_unreadable_file_rejects_entire_merge() {
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
