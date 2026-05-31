# dionysus

A Rust GTK4 application launcher for Wayland.

## About (WIP)

simple app laucnher build with Rust and GTK4
compatible with any wayland compositor

## Installing 

### From source

```bash
cd dionysus
cargo install --path . --root /usr/local
```

### AUR
```bash
paru -S dionysus-git
```

## Local development

Build and run manually:

```bash
cargo run --release -- init
dionysus toggle
```

To use with Hyprland
```lua
hl.exec_cmd("dionysus init &")
hl.bind(mainMod .. " + W", hl.dsp.exec_cmd("dionysus toggle"))
```

Licensed under GPL-3.0-or-later
