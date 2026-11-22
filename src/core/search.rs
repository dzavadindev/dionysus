//! Pure application-search and ranking functions.

use super::AppEntry;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// How applications are matched and ordered for a non-empty query.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchMode {
    #[default]
    Freq,
    Fuzz,
}

impl SearchMode {
    pub fn toggle(self) -> Self {
        match self {
            Self::Freq => Self::Fuzz,
            Self::Fuzz => Self::Freq,
        }
    }
}

/// Ranks applications by fuzzy match quality, launch frequency, and ID.
///
/// Empty queries return applications ordered by launch frequency. Non-empty
/// queries match either case-insensitive substrings or ordered fuzzy character
/// matches. Results are deterministic when all other ranking values match.
pub fn rank_apps(
    apps: &[AppEntry],
    frequencies: &HashMap<String, u64>,
    query: &str,
    limit: usize,
) -> Vec<AppEntry> {
    rank_apps_with_mode(apps, frequencies, query, limit, SearchMode::Fuzz)
}

/// Returns up to `limit` matches, ranked according to the selected mode.
/// FREQ requires a whitespace-insensitive substring and sorts by launch count;
/// FUZZ also accepts ordered character matches and sorts by match quality.
pub fn rank_apps_with_mode(
    apps: &[AppEntry],
    frequencies: &HashMap<String, u64>,
    query: &str,
    limit: usize,
    mode: SearchMode,
) -> Vec<AppEntry> {
    let normalized_query = normalize(query);
    if normalized_query.is_empty() {
        let mut ranked = apps.to_vec();
        ranked.sort_by(|a, b| {
            let a_frequency = frequencies.get(&a.id).copied().unwrap_or(0);
            let b_frequency = frequencies.get(&b.id).copied().unwrap_or(0);
            b_frequency.cmp(&a_frequency).then_with(|| a.id.cmp(&b.id))
        });
        ranked.truncate(limit);
        return ranked;
    }

    let mut scored: Vec<(&AppEntry, f64, u64)> = apps
        .iter()
        .filter_map(|app| {
            let name = normalize(&app.name);
            let score = match mode {
                SearchMode::Freq => name
                    .contains(&normalized_query)
                    .then(|| 1.0 + normalized_query.len() as f64 / name.len() as f64),
                SearchMode::Fuzz => fuzzy_score(&normalized_query, &name),
            }?;
            let frequency = frequencies.get(&app.id).copied().unwrap_or(0);
            Some((app, score, frequency))
        })
        .collect();

    scored.sort_by(|(a, score_a, freq_a), (b, score_b, freq_b)| {
        let quality = score_b.total_cmp(score_a);
        let frequency = freq_b.cmp(freq_a);
        match mode {
            SearchMode::Freq => frequency.then(quality),
            SearchMode::Fuzz => quality.then(frequency),
        }
        .then_with(|| a.id.cmp(&b.id))
    });

    scored
        .into_iter()
        .take(limit)
        .map(|(app, _, _)| app.clone())
        .collect()
}

fn normalize(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Determines how well a normalized search query matches a normalized name.
fn fuzzy_score(query: &str, candidate: &str) -> Option<f64> {
    if candidate.contains(query) {
        return Some(1.0 + (query.len() as f64 / candidate.len() as f64));
    }

    let query_chars: Vec<char> = query.chars().collect();
    if query_chars.is_empty() {
        return Some(0.0);
    }

    let mut query_index = 0;
    let mut matches = 0;
    for character in candidate.chars() {
        if query_index < query_chars.len() && character == query_chars[query_index] {
            query_index += 1;
            matches += 1;
        }
    }

    (matches == query_chars.len()).then(|| matches as f64 / candidate.chars().count() as f64)
}
