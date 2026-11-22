use dionysus::core::{
    AppEntry,
    search::{SearchMode, rank_apps, rank_apps_with_mode},
};
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
fn freq_mode_prioritizes_count_but_only_accepts_normalized_substrings() {
    let apps = vec![
        app("exact", "Terminal"),
        app("frequent", "Super Terminal App"),
        app("fuzzy", "Tremolo"),
    ];
    let frequencies = HashMap::from([(String::from("frequent"), 30), (String::from("fuzzy"), 100)]);

    let ranked = rank_apps_with_mode(&apps, &frequencies, "Te rM", 15, SearchMode::Freq);
    let ids: Vec<&str> = ranked.iter().map(|entry| entry.id.as_str()).collect();
    assert_eq!(ids, vec!["frequent", "exact"]);
}

#[test]
fn fuzz_mode_finds_subsequences_and_prioritizes_match_quality() {
    let apps = vec![
        app("exact", "Terminal"),
        app("frequent", "Team Rocket Manager"),
    ];
    let frequencies = HashMap::from([(String::from("frequent"), 100)]);

    let ranked = rank_apps_with_mode(&apps, &frequencies, "term", 15, SearchMode::Fuzz);
    let ids: Vec<&str> = ranked.iter().map(|entry| entry.id.as_str()).collect();
    assert_eq!(ids, vec!["exact", "frequent"]);
}

#[test]
fn result_limit_applies_after_ranking() {
    let apps: Vec<_> = (0..25)
        .map(|index| app(&format!("app-{index:02}"), "Terminal"))
        .collect();
    let frequencies = HashMap::from([(String::from("app-24"), 100)]);
    let ranked = rank_apps_with_mode(&apps, &frequencies, "", 15, SearchMode::Freq);

    assert_eq!(ranked.len(), 15);
    assert_eq!(ranked[0].id, "app-24");
}

#[test]
fn id_breaks_equal_score_and_frequency() {
    let apps = vec![app("b", "terminal"), app("a", "terminal")];

    let ranked = rank_apps(&apps, &HashMap::new(), "term", 10);
    let ids: Vec<&str> = ranked.iter().map(|entry| entry.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b"]);
}
