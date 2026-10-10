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
| `src/components/game_controls.rs` | Above-board status bar and below-keypad game actions |
| `src/components/game_page.rs` | Main game layout |
| `src/components/config_page.rs` | Player settings |
| `src/components/help_page.rs` | Player instructions |
| `src/components/header.rs` | Header, theme, and installation controls |
| `src/components/icon.rs` | Local decorative SVG line icons |
| `style/input.css` | Tailwind input and custom styles |
| `manifest.json`, `sw.js`, `public/icons/` | PWA metadata, caching, and icons |

## Build and hosting

The npm lockfile supplies Tailwind's CLI. `just css` produces `style/output.css`; Trunk bundles Rust/WASM and the linked assets into `dist/`. These outputs are ignored by version control.

`Trunk.toml` configures the bundle. Vercel explicitly uses the Other framework preset and Node.js 24. Its install phase runs `scripts/setup-build.sh` to install locked npm dependencies, Rust 1.98.1 with the WASM target, and checksum-verified Trunk 0.21.14. Rust and Trunk live under ignored `.build-tools/`; the bootstrap supports Linux x86_64 and aarch64 and requires network access on a clean install.

Setup, `build.sh`, and the Just command wrapper source `scripts/build-env.sh` for identical project-local tool paths. The build phase runs offline regression tests, uses npm's locked Tailwind CLI, and runs Trunk with `--release --locked`. `just build` invokes the same build script. Just development and validation commands use the same project-local tools; setup includes rustfmt and Clippy. `.nvmrc`, npm engine metadata and CI select Node 24, and scripts reject other Node majors. Static hosting retains SPA rewrites and the versioned offline hook. `.github/workflows/ci.yml` validates PRs and gates releases on tests and the production build. `scripts/release-policy.mjs` checks synchronized versions, changelog entries, and the declared saved-game compatibility against the PR base. After a merged PR reaches main, a separate job publishes a version tag and release with narrowly scoped write permissions. Reruns verify that any existing tag targets the same commit.

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
Selection/matching shading default to 20; available shading defaults to 100.
Dots/stripes default to enabled.
Partial settings use the same defaults, loading clamps percentages to 0–100,
new games preserve preferences, and reset restores defaults without changing
the board. Board CSS variables independently blend selection/peer, matching/blocker, and
available fills from the normal cell background to strong theme shades.
Zero restores the normal fill and disables that source’s stripes; selection
outline, matching underline, errors/hints, and optional dots remain independent.
Scoped data attributes control dots and each source’s stripe visibility.

## Difficulty audit

Engine tests cover all-setting puzzle invariants. The ignored
`audit_difficulty_settings` test emits native release-mode CSV;
`tests/difficulty-browser.mjs` measures WASM new-game selections.
[The review](difficulty-review.md) records current bounds/label limitations,
primary-source comparisons, raw datasets, and a proposed grading policy.

## Logical rating and generation

`src/sudoku_engine.rs` owns grader v1, deterministic deduction traces, bounded
MRV verification, density-constrained clue removal, and seeded selection.
`src/puzzle_bank.csv` contains self-generated verified fallback puzzles compiled
into WASM. `grade` never reads a solution; an unsupported/stalled puzzle has no
rating. Only matching ratings enter a new game.

`GameState.requested_difficulty` and `rating` are optional, serde-defaulted
metadata. New games store the actual engine difficulty and immutable original
puzzle rating. Loading old saves preserves their existing difficulty and state;
it does not regrade a partially played board. Undo, hints, and entries do not
change the puzzle's rating. The header distinguishes older ungraded games.
See [the implemented policy](difficulty-policy.md) for bounds and calibration.

## Domino cascades

`GameState.domino` stores serde-defaulted `DominoSettings`. Defaults use
600ms initial delay, 20% acceleration, 100ms minimum, and a 10-empty-cell limit,
with domino enabled. Explicit saved settings remain unchanged; missing settings
and configuration reset use these defaults.
Settings are normalized on load and through the setter and survive new games.
Normal and Drop entries share one transition. Only correct entries start a
cascade; enabling, loading, or changing settings does not auto-start it.
Each step recomputes naked singles with the engine's row/column/box candidate
scan, ignores notes and advanced deductions, and skips solution-inconsistent
candidates. The triggering entry and cascade share one undo snapshot.
A transient generation counter invalidates callbacks after board/note edits,
undo/redo, hints, pause, settings changes, or game replacement. Legacy saved
counters are ignored. Browser scheduling wraps native-testable state steps.

## UI controls

`GameStatus` renders the timer, labeled error count, and accessible pause/resume
control above the board. `GameControls` groups assistance actions below the
keypad, followed by the difficulty disclosure. Header utilities and other
non-board actions use shared 44px targets in `style/input.css`. Number buttons
use five columns on phones and nine above 640px, with 52px minimum height.
`GamePage` wraps the board/status and controls in separate panels. Tablet
styles grow the board up to 720px within the viewport and increase number
buttons to 64px height. Panels stay stacked at all viewport widths. Board
digits and notes scale up, and the manifest locks the installed PWA to portrait.

`Icon` uses a small `IconName` enum and local paths, with `aria-hidden` so control
labels provide accessible names. Active notes/Drop modes expose `aria-pressed`
and keep stable names. Settings uses labeled native checkboxes, radios, ranges,
and a native details/summary disclosure for timing. Its components call existing
state transitions; `set_sound` selects the existing `SoundType` directly.
Layout changes do not alter serialization or saved preferences.

`tests/ui-browser.mjs` covers responsive layouts, control sizing, keyboard
operation, disclosure, sound, mode states, installation, reset, and reload.
Screenshots under `docs/screenshots/ui-ux/` document the verified production UI.

`GameState.drop_pick_solved` is a saved preference with a true serde default.
In Drop mode, `select_cell` picks a nonzero correct value before attempting
an entry, without creating history or editing the board. Normal selection
and disabled-preference Drop entries keep their existing behavior. New games
preserve the preference; configuration reset enables it.

`number_is_solved` counts nine board/solution matches for a digit. Drop keypad
buttons disable completed digits outside notes mode, and `enter_number` also
blocks placement so picking a completed digit from a cell cannot bypass the
rule. Notes remain editable; incorrect duplicates do not count as solved.

## Local development

`just dev` (also `just serve`) invokes `scripts/with-build-env.sh` and
`trunk serve --config Trunk.dev.toml --locked`. The development configuration
uses debug builds in `.dev-dist/` and watches source inputs rather than
installed tools, generated CSS, or build outputs. A pre-build npm hook rebuilds
Tailwind when Rust/CSS input changes. The post-build development hook stages a
network-only worker that takes over older local Sudoku workers and clears only
Sudoku caches. Production `Trunk.toml` keeps the content-versioned offline hook
and `dist/` output. Saved-game storage is unchanged in both modes.

## Android wrapper

`capacitor.config.json` fixes the native origin to `https://localhost`. The tracked
`android/` template supplies the Gradle wrapper and native activity.
`scripts/android.mjs` builds release assets into `.android-dist/`, stages a native
marker and removes the worker in `.android-web/`, then syncs Capacitor.
`index.html` gates browser installation/worker registration on that marker.
The activity handles system-bar insets, portrait and Back navigation; Capacitor
opens external origins through Android intents. Gradle derives version metadata
from npm and requires private credentials for release signing. State serialization
is unchanged. Android checks and publication use separate workflows; see
[Android documentation](android.md) for acceptance limits and signing setup.

## Completed keypad contrast

`HighlightSettings.completed_contrast` is a serde-defaulted 0–100 percentage,
with a default of 100 and load-time clamping. Existing saves gain only this
preference; board, notes and history are unchanged. New games retain the setting
and configuration reset restores its default. `NumberPad` exposes it as the
`--completed-contrast` CSS property. `NumberBtn` uses `number_is_solved` for its
completion marker and accessible description independently of disabling/Drop
selection, so incorrect duplicates never gain the completed style. A dedicated
settings slider controls the fill and outline; text remains readable throughout
the range, and the checkmark remains present at zero. Selected Drop buttons use
the existing selected colors. Styling does not change entry rules.
