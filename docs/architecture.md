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

Both setup and `build.sh` source `scripts/build-env.sh` for identical project-local tool paths. The build phase runs offline regression tests, uses npm's locked Tailwind CLI, and runs Trunk with `--release --locked`. `just build` invokes the same build script. Standard development commands still use shell-installed Rust/Trunk; the compiler is pinned by `rust-toolchain.toml`. Static hosting retains SPA rewrites and the versioned offline hook. `.github/workflows/ci.yml` validates PRs and gates releases on tests and the production build. `scripts/release-policy.mjs` checks synchronized versions, changelog entries, and the declared saved-game compatibility against the PR base. After a merged PR reaches main, a separate job publishes a version tag and release with narrowly scoped write permissions. Reruns verify that any existing tag targets the same commit.

## Persistence and change boundaries

Game progress is serialized into the `sudoku_state` localStorage key by `src/state.rs`. Check the load/save functions and serde helpers when altering the state format. Browser APIs belong at the state/UI boundary; keep engine algorithms independent of DOM and Leptos.

## Offline bundles

Trunk runs `scripts/offline-bundle.mjs` after asset generation and before publishing its staging directory. The script enumerates the staged files (excluding the worker), hashes their paths/contents and the worker template, and injects the asset list and version into the deployed worker. Node.js is required for all Trunk builds, including the deployment script.

Each worker installs its complete bundle into a separate `sudoku-offline-<hash>` cache. A failed precache rejects installation and deletes only that incomplete cache. Navigation always uses that version's cached HTML; listed assets use the same cache, keeping HTML and hashed assets consistent across releases. Other requests use the browser's network behavior.

Updates wait for all controlled tabs to close. Activation removes only obsolete Sudoku caches, including legacy `sudoku-v1`; unrelated caches and localStorage remain untouched. Initial installation does not claim the already-open page, so reload after installation to enter worker control.

Hosting defaults to revalidation, with immutable caching reserved for hashed Trunk JS/WASM/CSS. Verify actual Vercel response headers on a preview before marking the hosting-cache issue resolved.

`VISION.md` describes product goals, current capabilities, and remaining validation work.

## Theme and clue provenance

`style/input.css` defines light and dark `--ui-*` tokens exposed through Tailwind
v4 semantic utilities. Cell state precedence is error, hint, selected, matching,
available/blocked/matching-blocked (empty cells with an active digit or empty selection), peer, secondary, default. Selection has a separate 3px outline, including on errors
and hints. The early theme script in `index.html` applies system preference
before WASM renders.

`GameState.givens` stores the original puzzle cells separately from mutable
board entries; undo snapshots do not change genuine original clues. Hints remain separately
locked. The loader migrates saves without `givens` by preserving their old
correct-number locks, without changing board values, notes, history, or settings. If a historical snapshot changes a
migrated locked entry, it proves the entry was editable and releases that lock
so undo cannot leave an empty cell permanently locked.

`GameState.active_number` derives the inspected digit from Drop selection or
the selected filled cell. `placement_available` uses the existing engine
`is_valid_move` on empty cells and the current board only. With no digit in normal
mode, it instead classifies selected-unit peers versus outside empty cells. Preview classifications are derived rather than serialized. Matching digits/notes use the same active digit. A selected empty
cell retains its selection fill/outline, with placement metadata and an available
dot when inspecting a Drop digit; errors and hints retain precedence.

`placement_blocker` classifies blocked empty cells as selected or matching.
Selected-source stripes win overlaps; a keypad digit with no selected matching
occurrence classifies all blockers as matching. Source metadata and Portuguese accessible
labels expose the distinction without persisting the classifications. The board has
no visible legend: `/` hatching denotes selected-source blockers and `\` denotes
matching-source blockers.

`GameState.highlights` persists a serde-defaulted `HighlightSettings` object.
Selection/matching shading default to 100; available shading defaults to zero.
Dots/stripes default to enabled.
Partial settings use the same defaults, loading clamps percentages to 0–100,
new games preserve preferences, and reset restores defaults without changing
the board. Board CSS variables independently blend selection/peer, matching/blocker, and
available fills from the normal cell background to strong theme shades.
Zero restores the normal fill and disables that source’s stripes; selection
outline, matching underline, errors/hints, and optional dots remain independent.
Scoped data attributes control dots and each source’s stripe visibility.
