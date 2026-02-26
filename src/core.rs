pub mod desktop;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

// ----------- STATE STRUCTS

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub id: String,
    pub desktop_path: PathBuf,
    pub name: String,
    pub exec: String,
    pub icon: Option<IconRef>,
    pub terminal: bool,
    pub nodisplay: bool,
    pub hidden: bool,
}

#[derive(Debug)]
pub struct AppState {
    pub frequency: HashMap<String, u64>,
}

#[derive(Clone)]
pub struct SharedState(Arc<Mutex<AppState>>);

#[derive(Debug, Clone)]
pub enum IconRef {
    ThemedName(String),
    FilePath(PathBuf),
}

// ----------- STRUCT IMPLS

impl AppState {
    pub fn new() -> Self {
        Self {
            frequency: Default::default(),
        }
    }

    pub fn record_launch(&mut self, id: &str) {
        *self.frequency.entry(id.to_string()).or_insert(0) += 1;
    }
}

impl SharedState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(AppState::new())))
    }

    pub fn lock(&self) -> MutexGuard<'_, AppState> {
        // TODO: :) maybe don't just unwrap
        self.0.lock().unwrap()
    }
}
