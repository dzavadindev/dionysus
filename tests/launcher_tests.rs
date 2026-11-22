use dionysus::core::{AppEntry, Launcher};
use std::{collections::HashMap, path::PathBuf};

fn app(id: &str, name: &str) -> AppEntry {
    AppEntry {
        id: id.to_string(),
        desktop_path: PathBuf::from(format!("/tmp/{id}.desktop")),
        name: name.to_string(),
        exec: name.to_string(),
        icon: None,
        terminal: false,
        nodisplay: false,
        hidden: false,
    }
}

#[test]
fn query_and_selection_are_updated_by_core_operations() {
    let apps = vec![app("one", "Terminal"), app("two", "Browser")];
    let mut launcher = Launcher::new(apps, HashMap::new(), 5);

    assert_eq!(launcher.selected_index(), Some(0));
    launcher.move_selection_down();
    assert_eq!(
        launcher.selected().map(|entry| entry.id.as_str()),
        Some("two")
    );

    launcher.set_query("term");
    assert_eq!(launcher.results().len(), 1);
    assert_eq!(
        launcher.selected().map(|entry| entry.id.as_str()),
        Some("one")
    );
}

#[test]
fn selection_and_catalog_refresh_follow_current_results() {
    let apps = vec![app("one", "Terminal"), app("two", "Browser")];
    let mut launcher = Launcher::new(apps, HashMap::new(), usize::MAX);

    launcher.select_index(1);
    assert_eq!(launcher.selected().map(|app| app.id.as_str()), Some("two"));
    launcher.select_index(42);
    assert_eq!(launcher.selected().map(|app| app.id.as_str()), Some("two"));

    launcher.set_query("terminal");
    launcher.replace_catalog(vec![app("three", "New Terminal"), app("four", "Editor")]);
    assert_eq!(launcher.results().len(), 1);
    assert_eq!(
        launcher.selected().map(|app| app.id.as_str()),
        Some("three")
    );
}

#[test]
fn reset_clears_query_and_selects_first_result() {
    let apps = vec![app("one", "Terminal"), app("two", "Browser")];
    let mut launcher = Launcher::new(apps, HashMap::new(), 5);

    launcher.set_query("browser");
    launcher.reset();

    assert_eq!(launcher.query(), "");
    assert_eq!(launcher.results().len(), 2);
    assert_eq!(launcher.selected_index(), Some(0));
}
