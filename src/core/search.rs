//! Pure application-search and ranking functions.

use super::AppEntry;
use std::collections::HashMap;

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
    let trimmed = query.trim();
    if trimmed.is_empty() {
        let mut ranked = apps.to_vec();
        ranked.sort_by(|a, b| {
            let a_frequency = frequencies.get(&a.id).copied().unwrap_or(0);
            let b_frequency = frequencies.get(&b.id).copied().unwrap_or(0);
            b_frequency.cmp(&a_frequency).then_with(|| a.id.cmp(&b.id))
        });
        ranked.truncate(limit);
        return ranked;
    }

    let mut scored: Vec<(AppEntry, f64, u64)> = apps
        .iter()
        .filter_map(|app| {
            fuzzy_score(trimmed, &app.name).map(|score| {
                let frequency = frequencies.get(&app.id).copied().unwrap_or(0);
                (app.clone(), score, frequency)
            })
        })
        .collect();

    scored.sort_by(
        |(app_a, score_a, frequency_a), (app_b, score_b, frequency_b)| {
            score_b
                .total_cmp(score_a)
                .then_with(|| frequency_b.cmp(frequency_a))
                .then_with(|| app_a.id.cmp(&app_b.id))
        },
    );

    scored
        .into_iter()
        .take(limit)
        .map(|(app, _, _)| app)
        .collect()
}

/// Determines how well a search query matches an application name.
fn fuzzy_score(query: &str, candidate: &str) -> Option<f64> {
    let query = query.to_lowercase();
    let candidate = candidate.to_lowercase();

    if candidate.contains(&query) {
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
