mod app;
mod core;
mod ipc;
mod ui;

fn main() {
    let state = core::SharedState::new();

    let mut args = std::env::args().skip(1);

    if let Some(cmd) = args.next() {
        if cmd == ipc::TOGGLE_COMMAND {
            if let Err(err) = ipc::send_toggle() {
                eprintln!("Failed to send toggle: {err}");
            }
            return;
        }
    }

    app::start(state);
}
