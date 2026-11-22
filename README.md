# dionysus

A Rust application launcher with a GTK/Relm4 interface and a UI-independent core.

The core discovers freedesktop `.desktop` entries, ranks applications using
fuzzy search and launch frequency, starts applications, and sorts based on launch frequencies. Also, the list of entries refreshes when desktop files change! (One of the things I missed in other app launchers I used)

## Launcher

Start the long-running instance with `dionysus init`, then show or hide it with `dionysus toggle`. Type to filter applications (up to 15 results). Use Up/Down to select an app, Enter to launch it and hide the launcher, or Escape to hide it without launching. Ctrl+Tab switches between FREQ (frequent substring matches) and FUZZ (fuzzy relevance); the mode persists across restarts.

## State and styling

The state file at `$XDG_STATE_HOME/dionysus/freq.json` (or
`~/.local/state/dionysus/freq.json`) now requires both fields:

```json
{"frequencies": {"example.desktop": 4}, "mode": "freq"}
```

Older frequency-only files must be converted or removed before starting the
launcher. Add UI styling to `src/ui/style.css`, using Dionysus-specific CSS
classes for selectors; it is bundled and applied globally at startup.

Licensed under GPL-3.0-or-later
