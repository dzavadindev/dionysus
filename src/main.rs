mod app;
mod core;
mod ui;

fn main() {
    let state = app::build_context();
    app::start(state);
}
