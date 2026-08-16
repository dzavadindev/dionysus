use dionysus::core::{AppEntry, search::rank_apps};
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
fn score_takes_priority_over_frequency() {
    let apps = vec![app("high", "almanac"), app("low", "atlas")];
    let frequencies = HashMap::from([(String::from("low"), 500)]);

    let ranked = rank_apps(&apps, &frequencies, "alma", 10);
    assert_eq!(ranked.first().map(|entry| entry.id.as_str()), Some("high"));
}

#[test]
fn frequency_breaks_equal_scores() {
    let apps = vec![app("a", "terminal"), app("b", "terminal")];
    let frequencies = HashMap::from([(String::from("a"), 2), (String::from("b"), 10)]);

    let ranked = rank_apps(&apps, &frequencies, "term", 10);
    assert_eq!(ranked.first().map(|entry| entry.id.as_str()), Some("b"));
}

#[test]
fn id_breaks_equal_score_and_frequency() {
    let apps = vec![app("b", "terminal"), app("a", "terminal")];

    let ranked = rank_apps(&apps, &HashMap::new(), "term", 10);
    let ids: Vec<&str> = ranked.iter().map(|entry| entry.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b"]);
}
