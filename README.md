# dionysus

A UI-independent Rust application-launcher library.

The library discovers freedesktop `.desktop` entries, ranks applications using
fuzzy search and launch frequency, starts applications, and persists launch
frequencies. A future GTK/Relm4 interface can use `dionysus::core::Launcher`
as its application behavior and keep all rendering state in its own Relm4
model.

## Library usage

```rust,no_run
use dionysus::core::{desktop, Launcher};

let catalog = desktop::load_app_entries();
let mut launcher = Launcher::from_catalog(catalog, 5)?;
launcher.set_query("terminal");

if let Some(app) = launcher.selected() {
    println!("{}", app.name);
}

# Ok::<(), Box<dyn std::error::Error>>(())
```

The core contains no GTK, Relm4, or other UI dependencies. Query changes,
selection movement, and launching can be connected to Relm4 input messages.

Licensed under GPL-3.0-or-later
