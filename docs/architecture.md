# Architecture

## Runtime

`index.html` is the Trunk entry point. It links the generated CSS, copies the PWA assets, supplies the loading screen, and registers the service worker and install prompt bridge.

`src/main.rs` mounts the Leptos app. `src/app.rs` provides shared game and theme signals, runs the timer and persistence effects, and defines the game (`/`), help (`/help`), and configuration (`/config`) routes.

## Source map

| Location | Responsibility |
| --- | --- |
| `src/sudoku_engine.rs` | Puzzle generation, solving, candidates, and validation; engine unit tests |
| `src/state.rs` | Game state, user actions, undo/redo, hints, domino behavior, localStorage persistence, and state tests |
| `src/serde_helpers.rs` | Serialization helpers for 81-element arrays |
| `src/components/cell.rs` | Cell rendering and interaction |
| `src/components/sudoku_grid.rs` | Board layout |
| `src/components/number_pad.rs` | Number, delete, notes, and drop-mode controls |
| `src/components/game_controls.rs` | Game actions and timer controls |
| `src/components/game_page.rs` | Main game layout |
| `src/components/config_page.rs` | Player settings |
| `src/components/help_page.rs` | Player instructions |
| `src/components/header.rs` | Header, theme, and installation controls |
| `style/input.css` | Tailwind input and custom styles |
| `manifest.json`, `sw.js`, `public/icons/` | PWA metadata, caching, and icons |

## Build and hosting

The npm lockfile supplies Tailwind's CLI. `just css` produces `style/output.css`; Trunk bundles Rust/WASM and the linked assets into `dist/`. These outputs are ignored by version control.

`Trunk.toml` configures the bundle. `vercel.json` invokes `build.sh` and configures static hosting with SPA rewrites. The deployment script currently installs its own standalone Tailwind CLI; local builds use the npm dependency. There is no GitHub Actions workflow in this repository.

## Persistence and change boundaries

Game progress is serialized into the `sudoku_state` localStorage key by `src/state.rs`. Check the load/save functions and serde helpers when altering the state format. Browser APIs belong at the state/UI boundary; keep engine algorithms independent of DOM and Leptos.

`VISION.md` describes product goals, current capabilities, and known gaps. In particular, the current service worker precaches only the manifest and does not cache fetched app-shell assets; full offline support remains a goal.
