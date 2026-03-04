pub mod desktop;
pub mod search;

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
    pub apps: Vec<AppEntry>,
    pub freq: HashMap<String, u64>,
}

#[derive(Clone)]
pub struct UiHandle {
    pub window: gtk4::ApplicationWindow,
}

#[derive(Clone, Debug)]
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
            apps: Default::default(),
            freq: Default::default(),
        }
    }

    pub fn record_launch(&mut self, id: &str) {
        self.freq
            .entry(id.to_string())
            .and_modify(|e| *e += 1)
            .or_insert(0);
    }

    pub fn init_apps(&mut self, apps: &Vec<AppEntry>) {
        let clone = apps.clone();
        self.apps = clone;
    }
}

impl SharedState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(AppState::new())))
    }

    pub fn lock(&self) -> MutexGuard<'_, AppState> {
        self.0.lock().unwrap()
    }
}
