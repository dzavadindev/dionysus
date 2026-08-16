//! Stateful, UI-independent launcher operations.

use super::AppEntry;
use super::desktop;
use super::freq_store;
use super::search;
use std::collections::HashMap;
use std::io;

/// Errors that can occur while launching an application through [`Launcher`].
#[derive(Debug)]
pub enum LaunchError {
    /// No application is currently selected.
    NoSelection,
    /// The desktop entry's executable could not be started.
    Execution(desktop::LaunchError),
    /// The launch succeeded, but the updated frequency data could not be saved.
    Frequency(io::Error),
}

impl std::fmt::Display for LaunchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSelection => formatter.write_str("no application is selected"),
            Self::Execution(error) => write!(formatter, "failed to launch application: {error}"),
            Self::Frequency(error) => write!(formatter, "failed to save launch frequency: {error}"),
        }
    }
}

impl std::error::Error for LaunchError {}

/// Stateful launcher behavior that can be driven by any user interface.
///
/// The launcher owns query, result, selection, and frequency state. A UI can
/// read [`Self::results`] and [`Self::selected`] after handling each user
/// action, then render those values however it wants.
#[derive(Debug)]
pub struct Launcher {
    catalog: Vec<AppEntry>,
    frequencies: HashMap<String, u64>,
    query: String,
    results: Vec<AppEntry>,
    selected: Option<usize>,
    result_limit: usize,
}

impl Launcher {
    /// Creates a launcher using the supplied catalog and persisted frequencies.
    ///
    /// The initial query is empty, so the initial results are ordered by launch
    /// frequency. `result_limit` controls the maximum number of results.
    pub fn new(
        catalog: Vec<AppEntry>,
        frequencies: HashMap<String, u64>,
        result_limit: usize,
    ) -> Self {
        let mut launcher = Self {
            catalog,
            frequencies,
            query: String::new(),
            results: Vec::new(),
            selected: None,
            result_limit,
        };
        launcher.refresh_results();
        launcher
    }

    /// Loads persisted frequencies and creates a launcher for `catalog`.
    pub fn from_catalog(catalog: Vec<AppEntry>, result_limit: usize) -> io::Result<Self> {
        let frequencies = freq_store::load_freq()?;
        Ok(Self::new(catalog, frequencies, result_limit))
    }

    /// Returns the complete catalog currently used for searching.
    pub fn catalog(&self) -> &[AppEntry] {
        &self.catalog
    }

    /// Returns the current search query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns the current ranked search results.
    pub fn results(&self) -> &[AppEntry] {
        &self.results
    }

    /// Returns the selected result index, if a result is selected.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    /// Returns the currently selected application.
    pub fn selected(&self) -> Option<&AppEntry> {
        self.selected.and_then(|index| self.results.get(index))
    }

    /// Returns the recorded launch count for an application.
    pub fn frequency(&self, id: &str) -> u64 {
        self.frequencies.get(id).copied().unwrap_or(0)
    }

    /// Replaces the query and recalculates ranked results.
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.refresh_results();
    }

    /// Moves the selection toward the first result.
    pub fn move_selection_up(&mut self) {
        self.selected = if self.results.is_empty() {
            None
        } else {
            Some(self.selected.unwrap_or(0).saturating_sub(1))
        };
    }

    /// Moves the selection toward the last result.
    pub fn move_selection_down(&mut self) {
        self.selected = if self.results.is_empty() {
            None
        } else {
            Some((self.selected.unwrap_or(0) + 1).min(self.results.len() - 1))
        };
    }

    /// Launches the selected application and persists its updated frequency.
    pub fn launch_selected(&mut self) -> Result<(), LaunchError> {
        let id = self.selected().ok_or(LaunchError::NoSelection)?.id.clone();
        self.launch_by_id(&id)
    }

    /// Finds, launches, and records an application by desktop-entry ID.
    pub fn launch_by_id(&mut self, id: &str) -> Result<(), LaunchError> {
        let app = self
            .catalog
            .iter()
            .find(|app| app.id == id)
            .ok_or(LaunchError::NoSelection)?;

        desktop::launch_exec(&app.exec).map_err(LaunchError::Execution)?;
        self.frequencies
            .entry(id.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        freq_store::save_freq(&self.frequencies).map_err(LaunchError::Frequency)?;
        self.refresh_results();
        Ok(())
    }

    /// Clears the query and selection, preparing the launcher to be shown again.
    pub fn reset(&mut self) {
        self.query.clear();
        self.refresh_results();
    }

    fn refresh_results(&mut self) {
        self.results = search::rank_apps(
            &self.catalog,
            &self.frequencies,
            &self.query,
            self.result_limit,
        );
        self.selected = (!self.results.is_empty()).then_some(0);
    }
}
