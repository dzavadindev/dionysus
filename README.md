# dionysus

A Rust GTK4 application launcher for Wayland.

_"dionysus" is currently a working title._

## Installing 

### From source

```bash
cd dionysus
cargo install --path . --root /usr/local
```

### AUR (to come)
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

## TODO before official release

- Pick a license and add a `LICENSE` file.
- Set real repository/homepage/docs URLs in `Cargo.toml`.
- Replace placeholder metadata in `PKGBUILD`.
- Add desktop entry/icon install lines in `PKGBUILD` once those assets exist.
- CI/CD pipeline to build and create releases from main.
