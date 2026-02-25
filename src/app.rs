use gtk4::Application;
use gtk4::prelude::*;

use crate::core::desktop;
use crate::ui::window;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct AppState {
    pub frequency: HashMap<String, u64>,
}

impl AppState {
    pub fn record_launch(&mut self, id: &str) {
        *self.frequency.entry(id.to_string()).or_insert(0) += 1;
    }
}

pub type SharedState = Arc<Mutex<AppState>>;

pub fn build_context() -> SharedState {
    Arc::new(Mutex::new(AppState {
        frequency: Default::default(),
    }))
}

fn init_window() {
    let app = Application::builder()
        .application_id("com.dzavadindev.dionysus")
        .build();

    app.connect_activate(|app| {
        window::build_launcher_window(&app);
    });

    app.run();
}

pub fn start(_state: Arc<Mutex<AppState>>) {
    // init_window();
    println!("{:?}", _state.lock().unwrap());
}
