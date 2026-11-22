use dionysus::core::{freq_store::LauncherState, search::SearchMode};
use std::collections::HashMap;

#[test]
fn state_round_trips_frequency_counts_and_ranking_mode() {
    let state = LauncherState {
        frequencies: HashMap::from([(String::from("app.desktop"), 12)]),
        mode: SearchMode::Fuzz,
    };

    let json = serde_json::to_string(&state).unwrap();
    let restored: LauncherState = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.mode, SearchMode::Fuzz);
    assert_eq!(restored.frequencies, state.frequencies);
}

#[test]
fn state_requires_both_fields() {
    assert!(serde_json::from_str::<LauncherState>(r#"{"app.desktop":7}"#).is_err());
    assert!(serde_json::from_str::<LauncherState>(r#"{"frequencies":{}}"#).is_err());
    assert_eq!(LauncherState::default().mode, SearchMode::Freq);
}
