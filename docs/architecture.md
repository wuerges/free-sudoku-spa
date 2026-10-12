# Architecture

## Runtime

`index.html` is the Trunk entry point. It links the generated CSS, copies the PWA assets, supplies the loading screen, and registers the service worker and install prompt bridge.

`src/main.rs` mounts the Leptos app. `src/app.rs` provides shared game and theme signals, runs the timer and persistence effects, and defines the game (`/`), help (`/help`), and configuration (`/config`) routes.

## Source map

| Location | Responsibility |
| --- | --- |
| `src/sudoku_engine.rs` | Puzzle generation, solving, candidates, and validation; engine unit tests |
| `src/game_history.rs` | Archive DTO and retention policy |
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

`GameStatus` renders time, errors and pause inside the compact header; the
caption below contains title and difficulty. Header utility targets are 36px.
NumberPad renders the digits, four color choices and clear-all, followed by
Apagar, Notas, Drop and GameControls' Ações trigger. Digits and color tools share
one CSS height variable: 44px on phones and 48px on tablets.

GameControls owns a transient open signal and a dismissible action panel with
undo/redo, auto-notes, hint and new-game difficulty choices. The anchored panel
uses a fixed outside-click layer and height-limited scrolling; it overlays the
board instead of reserving layout space. Escape/close return focus to the
trigger; starting a game also closes the panel. Victory status stays visible
outside the panel. These UI signals are not persisted.

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
In normal Drop mode (outside Notes/painting), `select_cell` picks a nonzero
correct value before attempting an entry, without creating history or editing
the board. It intentionally changes the highlight digit and anchor. Normal
selection and disabled-preference Drop entries keep their existing behavior. New games
preserve the preference; configuration reset enables it.

`number_is_solved` counts nine board/solution matches for a digit. Drop keypad
buttons disable completed digits outside Notes/painting, and `enter_number` also
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

## Manual cell colors and viewport sizing

`GameState.cell_colors` and `Snapshot.cell_colors` use the existing 81-byte
serde helper: 0 means unmarked, 1 green, 2 purple, 3 pink, and 4 orange. Missing
arrays default to zeros; unknown IDs normalize to zero on load in the current
board and both history stacks. All snapshot capture/restore paths include
colors. Entry, notes, hints and domino fills retain marks. New games clear them;
configuration reset preserves them. `paint_tool` is serde-skipped, with 1–4
selecting a color and `None` normal input; zero is not a selectable tool.

Without Drop, the palette applies immediately to `selected` and never activates
a tool; no selection means no action. With Drop, `paint_tool` selects repeated
painting without clearing `drop_number`. Digit choices keep painting active and
change the highlight digit. Tapping the active color exits painting. Toggling
Drop clears its digit/tool, and Notes/erase leave painting.

Drop painting, note entry and number placement use their target coordinates
without changing `selected` or expanded highlight state. Normal solved-cell
picking deliberately sets the digit/anchor and clears expanded highlights;
it is bypassed for painting and Notes. Painting can mark clues/hints without
editing values, cancels pending domino work, and rejects paused/won states.
Same-color removal and clear-all use existing snapshots.

The palette sits immediately after the digits. Cell exposes `data-color` and
a Portuguese color description. Manual background colors override highlight
fills, preserving selection/focus outlines, matching underlines, placement
stripes/dots and error/hint text. There is no cell-edge marker. Theme tests cover
board text, error/hint text, grid and selection contrast on each manual fill.

The game is a vertical flex layout: compact header, square board, controls.
Layout wrappers use `display: contents`. Dynamic viewport height (`dvh`) and
safe-area insets reserve control height without JavaScript measurement. The
board flexes into remaining height, capped by available width and 840px on
tablets. Its minimum height is 252px; insufficient height falls back to document
scrolling. Controls stay next to the board rather than being separated by unused
space. The collapsed action panel has no permanent height cost.

`tests/cell-colors-browser.mjs` checks legacy history, same-color removal,
undo/redo, reload/offline, equal button heights, larger boards and action-panel
keyboard/outside dismissal. Browser audit helpers in `tests/game-actions.mjs`
open the panel for the existing game-action flows. Physical S23 Ultra/11-inch
tablet acceptance remains separate from emulation.

## Shared links

`src/sharing.rs` defines an independent v1 DTO and URL-safe unpadded Base64 JSON
codec, capped at 8192 encoded bytes. It validates exact array sizes, ranges,
solution units and clue/hint consistency before preview; wrong player entries
are allowed. It never passes untrusted data to the trusted local-save loader.
`ShareDialogs` handles initial and subsequent hash navigation with native modal
dialogs (focus containment, Escape, scrollable phone layout). Links are captured
on opening (default: current progress) and on mode changes, using untracked
game reads so timer updates do not change the displayed snapshot. Recipients
select the received snapshot or its initial position; preview and
confirmation use the same derived fields without changing the incoming payload.
`initial_position` clears progress while retaining recorded clues and metadata.
`import_shared` replaces only game fields, invalidates domino callbacks and
clears transient input
and history. Completed imports suppress celebration. Save serialization is unchanged.
The build-resolved public origin also applies inside the APK; APK link import is out of scope.


`build.rs` invokes the shared Node resolver in `scripts/public-url.mjs` and
embeds the validated origin into Rust, with Cargo environment-change tracking.
Trunk hooks apply the same resolver to marked social metadata in staged HTML,
before release offline hashing or development-worker staging. Source HTML is
never rewritten. Previews use VERCEL_URL; only production considers
VERCEL_PROJECT_PRODUCTION_URL. Explicit SUDOKU_PUBLIC_URL overrides both.


## Local game archive

`GameState.game_id` is a serde-defaulted stable local identity. Missing IDs are
assigned on load without changing the board or preferences. Shared DTOs do not
include identity. `/history` has an active card plus pinned/recent past cards;
read-only HTML/CSS miniature boards reuse the import preview renderer.

`GameHistory` in `src/state.rs` coordinates all user-triggered replacements. It freezes gameplay
and invalidates domino callbacks, awaits an outgoing snapshot transaction, checks
that the candidate can be persisted to localStorage, then switches the active
signal. Resume restores identity and undo/redo while merging current settings;
restart creates another identity and marks the source attempt recently used.
Shared-link confirmation uses the same archive-first transition and removes the
fragment only on success. Failure restores the prior pause state and keeps progress.

`public/history-store.js` owns IndexedDB transactions for the version-1
`sudoku_game_history` database (`games` and `meta` stores). It preserves pin flags
when snapshots are updated and assigns monotonically increasing activation order
plus a wall-clock display date. Pinning and archive updates preserve recency.
Rust selects eviction candidates: only unpinned past records after the newest ten,
excluding the active ID. The deletion transaction checks pins/active identity again.
Pruning runs after the new active save is persisted; failures retain extras for
retry. Invalid records are reported and kept in storage. No server is involved.
The active record's archive copy may lag ongoing gameplay; its card and reload use
the current localStorage-backed state, and replacement captures the latest snapshot.

The new bridge is precached with the app shell; generic SPA offline navigation
also covers `/history`. APK staging retains the bridge and its separate WebView
storage. Browser/device data retention remains a platform constraint.
