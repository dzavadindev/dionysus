use crate::core::AppEntry;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;

const SCORE_EPSILON: f64 = 0.12;

#[derive(Debug)]
pub enum SearchCommand {
    ReloadApps(Vec<AppEntry>),
    UpdateFreq(HashMap<String, u64>),
    Query(String),
    Stop,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub query: String,
    pub items: Vec<AppEntry>,
}

pub struct SearchWorkerHandle {
    pub command_tx: mpsc::Sender<SearchCommand>,
    pub result_rx: mpsc::Receiver<SearchResult>,
}

pub fn spawn_search_worker(limit: usize) -> SearchWorkerHandle {
    let (command_tx, command_rx) = mpsc::channel::<SearchCommand>();
    let (result_tx, result_rx) = mpsc::channel::<SearchResult>();

    thread::spawn(move || {
        let mut apps: Vec<AppEntry> = Vec::new();
        let mut freq: HashMap<String, u64> = HashMap::new();

        while let Ok(command) = command_rx.recv() {
            match command {
                SearchCommand::ReloadApps(next_apps) => {
                    apps = next_apps;
                }
                SearchCommand::UpdateFreq(next_freq) => {
                    freq = next_freq;
                }
                SearchCommand::Query(query) => {
                    let items = rank_apps(&apps, &freq, &query, limit);
                    let _ = result_tx.send(SearchResult { query, items });
                }
                SearchCommand::Stop => break,
            }
        }
    });

    SearchWorkerHandle {
        command_tx,
        result_rx,
    }
}

fn rank_apps(
    apps: &[AppEntry],
    freq: &HashMap<String, u64>,
    query: &str,
    limit: usize,
) -> Vec<AppEntry> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        let mut ranked = apps.to_vec();
        ranked.sort_by(|a, b| {
            let af = freq.get(&a.id).copied().unwrap_or(0);
            let bf = freq.get(&b.id).copied().unwrap_or(0);
            bf.cmp(&af)
        });
        ranked.truncate(limit);
        return ranked;
    }

    let mut scored: Vec<(AppEntry, f64, u64)> = apps
        .iter()
        .filter_map(|app| {
            fuzzy_score(trimmed, &app.name).map(|score| {
                let launches = freq.get(&app.id).copied().unwrap_or(0);
                (app.clone(), score, launches)
            })
        })
        .collect();

    scored.sort_by(|(_, score_a, freq_a), (_, score_b, freq_b)| {
        let score_diff = (score_b - score_a).abs();
        if score_diff <= SCORE_EPSILON {
            return freq_b.cmp(freq_a);
        }

        score_b.partial_cmp(score_a).unwrap_or(Ordering::Equal)
    });

    scored
        .into_iter()
        .take(limit)
        .map(|(app, _, _)| app)
        .collect()
}

fn fuzzy_score(query: &str, candidate: &str) -> Option<f64> {
    let q = query.to_lowercase();
    let c = candidate.to_lowercase();

    if c.contains(&q) {
        return Some(1.0 + (q.len() as f64 / c.len() as f64));
    }

    let mut qi = 0usize;
    let qchars: Vec<char> = q.chars().collect();
    if qchars.is_empty() {
        return Some(0.0);
    }

    let mut matches = 0usize;
    for ch in c.chars() {
        if qi < qchars.len() && ch == qchars[qi] {
            qi += 1;
            matches += 1;
        }
    }

    if matches == qchars.len() {
        Some(matches as f64 / c.chars().count() as f64)
    } else {
        None
    }
}
