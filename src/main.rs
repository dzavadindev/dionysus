mod app;
mod core;
mod ui;

fn main() {
    let state = core::SharedState::new();
    app::start(state);
}
