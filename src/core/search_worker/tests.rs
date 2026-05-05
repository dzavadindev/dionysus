use super::*;
use std::path::PathBuf;

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
fn rank_apps_prioritizes_score_over_frequency() {
    let apps = vec![app("high", "almanac"), app("low", "atlas")];
    let mut freq = HashMap::new();
    freq.insert("high".to_string(), 1);
    freq.insert("low".to_string(), 500);

    let ranked = rank_apps(&apps, &freq, "alma", 10);
    assert_eq!(ranked.first().map(|a| a.id.as_str()), Some("high"));
}

#[test]
fn rank_apps_uses_frequency_for_equal_scores() {
    let apps = vec![app("a", "terminal"), app("b", "terminal")];
    let mut freq = HashMap::new();
    freq.insert("a".to_string(), 2);
    freq.insert("b".to_string(), 10);

    let ranked = rank_apps(&apps, &freq, "term", 10);
    assert_eq!(ranked.first().map(|a| a.id.as_str()), Some("b"));
}

#[test]
fn rank_apps_is_deterministic_when_score_and_frequency_match() {
    let apps = vec![app("b", "terminal"), app("a", "terminal")];
    let freq = HashMap::new();

    let ranked = rank_apps(&apps, &freq, "term", 10);
    let ids: Vec<&str> = ranked.iter().map(|a| a.id.as_str()).collect();
    assert_eq!(ids, vec!["a", "b"]);
}
