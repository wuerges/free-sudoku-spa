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

`Trunk.toml` configures the bundle. Vercel explicitly uses the Other framework preset and Node.js 22. Its install phase runs `scripts/setup-build.sh` to install locked npm dependencies, Rust 1.98.1 with the WASM target, and checksum-verified Trunk 0.21.14. Rust and Trunk live under ignored `.build-tools/`; the bootstrap supports Linux x86_64 and aarch64 and requires network access on a clean install.

Both setup and `build.sh` source `scripts/build-env.sh` for identical project-local tool paths. The build phase runs offline regression tests, uses npm's locked Tailwind CLI, and runs Trunk with `--release --locked`. `just build` invokes the same build script. Standard development commands still use shell-installed Rust/Trunk; the compiler is pinned by `rust-toolchain.toml`. Static hosting retains SPA rewrites and the versioned offline hook. There is no GitHub Actions workflow in this repository.

## Persistence and change boundaries

Game progress is serialized into the `sudoku_state` localStorage key by `src/state.rs`. Check the load/save functions and serde helpers when altering the state format. Browser APIs belong at the state/UI boundary; keep engine algorithms independent of DOM and Leptos.

## Offline bundles

Trunk runs `scripts/offline-bundle.mjs` after asset generation and before publishing its staging directory. The script enumerates the staged files (excluding the worker), hashes their paths/contents and the worker template, and injects the asset list and version into the deployed worker. Node.js is required for all Trunk builds, including the deployment script.

Each worker installs its complete bundle into a separate `sudoku-offline-<hash>` cache. A failed precache rejects installation and deletes only that incomplete cache. Navigation always uses that version's cached HTML; listed assets use the same cache, keeping HTML and hashed assets consistent across releases. Other requests use the browser's network behavior.

Updates wait for all controlled tabs to close. Activation removes only obsolete Sudoku caches, including legacy `sudoku-v1`; unrelated caches and localStorage remain untouched. Initial installation does not claim the already-open page, so reload after installation to enter worker control.

Hosting defaults to revalidation, with immutable caching reserved for hashed Trunk JS/WASM/CSS. Verify actual Vercel response headers on a preview before marking the hosting-cache issue resolved.

`VISION.md` describes product goals, current capabilities, and remaining validation work.
