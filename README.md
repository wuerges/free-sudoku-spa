# Sudoku PWA

Sudoku Progressive Web App built with Rust, Leptos, and WebAssembly. Works offline, installable on Android.

## Tech

- **Rust** → WASM via `wasm-bindgen`
- **Leptos** (CSR) → reactive UI
- **Trunk** → build & bundle
- **Tailwind CSS v4** → styling
- **Vercel** → deploy (static)

## Setup

```bash
# Prerequisites
rustup target add wasm32-unknown-unknown
cargo install trunk

# Tailwind CSS CLI (standalone, no Node.js needed)
curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/download/v4.1.9/tailwindcss-linux-x64
chmod +x tailwindcss-linux-x64
sudo mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss
```

## Run

```bash
just serve     # dev server + hot reload
# or manually:
tailwindcss -i style/input.css -o style/output.css --watch &
trunk serve
```

## Commands

```bash
just build     # release build → dist/
just test      # cargo test
just check     # cargo check + clippy
just serve     # dev server
```

## Project

```
src/
├── main.rs               # WASM entry
├── app.rs                # root component
├── sudoku_engine.rs      # generation, solving, validation
├── state.rs              # reactive game state (RwSignal)
├── serde_helpers.rs      # serde for large arrays
├── utils.rs              # format_time
└── components/
    ├── cell.rs           # single grid cell
    ├── sudoku_grid.rs    # 9×9 grid
    ├── number_pad.rs     # 1-9 + delete + note
    ├── game_controls.rs  # timer, undo, hint, new game
    └── header.rs         # dark mode, install button
```

## PWA

- `manifest.json` — installable, standalone, portrait
- `sw.js` — cache-first, auto-update
- Install prompt via `beforeinstallprompt` (Android Chrome/Edge)

## Deploy

Push to `main` → GitHub Actions builds and deploys to Vercel.
Set `VERCEL_TOKEN` secret in repo settings.
