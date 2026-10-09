# Sudoku PWA

Sudoku Progressive Web App built with Rust, Leptos, and WebAssembly. Targets offline play and installation on Android; see [known gaps](VISION.md#1-offline-play-and-updates).

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

# CSS tooling (versions locked in package-lock.json)
npm ci
```

## Run

```bash
just serve     # dev server + hot reload
# or manually:
npm run css:watch &
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
└── components/
    ├── cell.rs           # single grid cell
    ├── sudoku_grid.rs    # 9×9 grid
    ├── number_pad.rs     # 1-9 + delete + note
    ├── game_controls.rs  # timer, undo, hint, new game
    ├── game_page.rs      # game route layout
    ├── config_page.rs    # config toggles (undo, auto-notes, hint, domino, sound)
    ├── help_page.rs      # help / instructions
    └── header.rs         # dark mode, install button
```

## Features

- 5 difficulty levels (Easy → Master) with unique-solution puzzles generated client-side in Rust/WASM
- **Efeito Dominó**: after a correct guess, auto-fills cells with a single candidate in a timed cascade (600ms → *0.8 → min 100ms)
- Sound on correct guess: beep, explosion (default), or off
- Pencil-mark notes mode with per-cell candidates
- **🎯 Drop mode**: select a number, then tap cells to place or note it in bulk
- Real-time conflict highlighting
- Undo/Redo with full history (cleared on hint)
- Timer with pause
- Hint system (disables on Master difficulty)
- Win detection with fireworks + balloons animation
- Dark/light mode (auto + manual toggle)
- PWA manifest and install prompt; complete offline caching remains unfinished


## PWA

- `manifest.json` — installable, standalone, portrait
- `sw.js` — network-first navigation, cache lookup for assets; currently precaches only the manifest
- Install prompt via `beforeinstallprompt` (Android Chrome/Edge)

## Deploy

Vercel configuration is in `vercel.json`; its install command runs `build.sh`
and publishes `dist/`. No GitHub Actions workflow is currently checked in.


## Working with Codex

Repository instructions live in [AGENTS.md](AGENTS.md), following the
[official Codex instructions guide](https://developers.openai.com/codex/guides/agents-md).
Use `jj` for version control. See [the architecture guide](docs/architecture.md)
for source responsibilities and [VISION.md](VISION.md) for product direction and known gaps.
Track unresolved work in [ISSUES.md](ISSUES.md); remove issues when they are solved and verified.

Run `just css-watch` in a second terminal when changing Tailwind styles during
`just serve`. Run `cargo fmt --check`, `just test`, and `just check` for Rust
changes, and `just build` to verify the production bundle.
