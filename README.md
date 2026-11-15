# Dionysus

Dionysus is a lightweight GTK4 application launcher for Linux Wayland
desktops. It reads freedesktop `.desktop` entries, lets you search for an
application by name, and launches the selected result from a keyboard-friendly
overlay.

## Features

- Finds applications in the standard freedesktop application directories and
  uses localized names when available. Hidden entries, entries marked
  `NoDisplay`, and entries without an `Exec` command are skipped.
- Filters the application list as you type. Search supports case-insensitive
  substrings and ordered fuzzy character matches. Results are ranked by match
  quality, then by how often you have launched each application.
- Shows up to five results in a GTK4 layer-shell overlay. The window adjusts
  its height to the result list and scrolls when needed.
- Keeps one application process running so a separate `toggle` command can
  show or hide the window. Escape hides it; the arrow keys change selection;
  Enter launches the selected application.
- Saves launch counts between sessions so frequently used applications appear
  first when the search field is empty.

## Requirements

- A Linux Wayland session with support for GTK4 Layer Shell.
- A Rust toolchain supporting the 2024 edition and Cargo.
- GTK4, GTK4 Layer Shell, and `pkg-config` development files to build from
  source. On Arch Linux, install `gtk4`, `gtk4-layer-shell`, and `pkgconf`.

Cargo will download the Rust dependencies from `Cargo.toml` during the first
build.

## Install

From a checkout of this repository:

```sh
cargo install --path . --locked
```

This puts `dionysus` in Cargo's binary directory, normally `~/.cargo/bin`.
Make sure that directory is on your `PATH`.

On Arch Linux, the separate `dionysus-git` AUR package is another option:

```sh
paru -S dionysus-git
```

## Run

Start the long-running application process once per desktop session:

```sh
dionysus init
```

The window starts hidden. From another terminal or a compositor key binding,
show or hide it with:

```sh
dionysus toggle
```

`toggle` reports when there is no running `init` instance. Running `dionysus`
without a command, or with `help`, prints the available commands.

For a quick source-tree run without installing the binary, use
`cargo run --locked -- init` in one terminal and
`cargo run --locked -- toggle` in another.

For example, a Hyprland configuration can start Dionysus with the session
and bind a key to the toggle command:

```ini
exec-once = dionysus init
bind = SUPER, Space, exec, dionysus toggle
```

Once the window is open, type to search, use Up and Down to select a result,
press Enter to launch it, or press Escape to hide the window. Launching an
application also hides the window.

## Launch history

Successful launches increment a counter for that desktop-entry ID. Dionysus
stores the counters in `$XDG_STATE_HOME/dionysus/freq.json`, or in
`~/.local/state/dionysus/freq.json` if `XDG_STATE_HOME` is unset. If neither
`XDG_STATE_HOME` nor `HOME` is set, the code falls back to
`/tmp/dionysus/freq.json`.

With an empty query, results are ordered by launch count. For a non-empty
query, match quality comes first and launch count breaks ties.

## Develop and test

```sh
cargo build --locked
cargo test --locked
```

The current tests cover search ranking and its frequency tie-breaker. They do
not open the GTK window or exercise the full desktop interaction.

## Known limitations

- Desktop entries are loaded when `init` starts. Restart the process to pick
  up applications installed afterward.
- `Exec` parsing is simple: Dionysus drops the command from the first `%`
  field code onward and splits the remainder on whitespace. Quoted arguments,
  later arguments, and terminal-specific behavior in desktop entries are not
  fully supported.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
