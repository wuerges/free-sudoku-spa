# Sudoku PWA

Sudoku Progressive Web App built with Rust, Leptos, and WebAssembly. Supports offline reload after a successful online installation and targets installation on Android.

## Tech

- **Rust** → WASM via `wasm-bindgen`
- **Leptos** (CSR) → reactive UI
- **Trunk** → build & bundle
- **Tailwind CSS v4** → styling
- **Vercel** → deploy (static)

## Setup

Use Node.js 24 and npm. With nvm, the checked-in `.nvmrc` selects Node 24:

```bash
nvm install
nvm use
just setup
just dev
```

`just setup` installs the pinned Rust 1.98.1 compiler, WASM target, rustfmt,
Clippy and checksum-verified Trunk 0.21.14 under `.build-tools/`, then uses
locked npm dependencies. Setup requires network access and supports Linux
x86_64/aarch64. `just` itself must be installed. If you do not use nvm, select
Node 24 with your preferred version manager first.

## Run

`just dev` is the local development target at http://localhost:8080. It uses
debug builds and rebuilds Tailwind CSS on Rust/CSS changes before Trunk reloads
the browser. No separate CSS watcher is needed. `just serve` is an alias for
this same command. Both accept Trunk options, for example:

```bash
just dev --port 8081
just dev --address 0.0.0.0 --port 8081  # access from a tablet on your LAN
```

Development assets go to `.dev-dist/`; `just build` produces the release PWA
in `dist/`. The development worker clears older Sudoku caches on that local
origin and lets requests use the network, so cached production assets cannot
hide edits. It preserves saved games. If you previously installed a release
on the same localhost origin, reload once after the development worker activates.
Use a different port when previewing a release alongside the development app.

All Just Rust targets use the project-local toolchain installed by `just setup`.
For platforms unsupported by setup, install the pinned Rust/Trunk tools and
Node 24 manually, then use `npm ci` and `trunk serve --config Trunk.dev.toml --locked`.

## Commands

```bash
just setup        # install project-local tools and locked npm dependencies
just dev          # debug server + Rust/CSS reload → .dev-dist/
just serve        # alias for dev
just build        # release PWA build → dist/
just fmt          # check Rust formatting
just test         # engine/state tests
just check        # WASM compilation + Clippy
just test-tooling # build/dev/offline/contrast/release regression tests
just ci           # all validation above + release build
just css          # standalone CSS build
just css-watch    # optional standalone CSS watcher
just clean        # remove dist/, .dev-dist/ and target/; preserve tools/dependencies
```

With Playwright available, run `node tests/dev-browser.mjs` after `just build`
to check offline release routes, development cache takeover, saved progress,
mobile/tablet layouts and automatic Rust/CSS reload. The audit temporarily edits
two source files and restores them; run it without concurrent source edits.
Set `PLAYWRIGHT_MODULE` to the module path if Playwright is installed elsewhere.

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
    ├── game_controls.rs  # status bar, undo, hint, new game
    ├── game_page.rs      # game route layout
    ├── config_page.rs    # config toggles (undo, auto-notes, hint, domino, sound)
    ├── help_page.rs      # help / instructions
    ├── header.rs         # dark mode, install button
    └── icon.rs           # shared local line icons
```

## Features

- 5 difficulty levels (Easy → Master) with unique-solution puzzles generated client-side in Rust/WASM
- **Efeito Dominó**: after a correct normal or Drop entry, auto-fills empty cells with exactly one candidate from a simple row/column/3×3 box scan. Saved settings control initial delay, acceleration, minimum delay (defaults: 600ms → *0.8 → min 100ms), and an empty-cell threshold (enabled by default at 10 empty cells; 0: no limit; positive: at most that many empties after the entry). Cascades stop on pause, game replacement, settings changes, or further board/note edits
- Sound on correct guess: beep, explosion (default), or off
- Pencil-mark notes mode with per-cell candidates
- **🎯 Drop mode**: select a number, then tap cells to place or note it in bulk
- In Drop mode, tap a correctly filled cell to pick its number without editing it. Disable “Selecionar número pelas células resolvidas” in Settings to keep the previous behavior. The preference defaults to enabled and survives reloads/new games.
- Completed numbers (nine correct occurrences) are disabled for Drop placement, but remain available in Drop notes mode. Incorrect duplicates do not count toward completion.
- Real-time conflict highlighting
- Undo/Redo with full history (cleared on hint)
- Timer with pause
- Hint system available at every level according to player settings
- Win detection with fireworks + balloons animation
- Dark/light mode (auto + manual toggle)
- PWA manifest, install prompt, and versioned offline app-shell caching


## PWA

- `manifest.json` — installable, standalone, portrait orientation
- `sw.js` — build-time template for versioned app-shell and asset caching
- Install prompt via `beforeinstallprompt` (Android Chrome/Edge)

## Deploy

Vercel uses the Other framework preset (`framework: null`) and Node.js 24.
Its install phase runs `sh scripts/setup-build.sh`; its build phase runs
`sh build.sh`, which checks offline regressions, builds locked Tailwind CSS,
and runs `trunk build --release --locked`. The output directory is `dist/`.
Build tooling lives in `.build-tools/`; no privileged install path is needed.
GitHub Actions checks PRs with formatting, tests, compilation, lint, color contrast,
version policy, and a production build. After a merged PR reaches `main` and
checks pass, CI publishes its `v<version>` tag and GitHub release. Direct pushes
and merges into feature branches do not publish releases.

Run `npm run test:build` to verify setup/build failure handling and environment
consistency. A successful local build does not establish deployed cache headers;
check the PR's Vercel preview before merging.


## Working with Codex

Repository instructions live in [AGENTS.md](AGENTS.md), following the
[official Codex instructions guide](https://developers.openai.com/codex/guides/agents-md).
Use `jj` for version control. See [the architecture guide](docs/architecture.md)
for source responsibilities and [VISION.md](VISION.md) for product direction and known gaps.
Track unresolved work in [ISSUES.md](ISSUES.md); remove issues when they are solved and verified.

`just dev` rebuilds CSS automatically on relevant source changes. Run `just fmt`,
`just test`, and `just check` for Rust changes, and `just build` to verify the
production bundle.

## Offline reload and updates

Trunk's post-build hook requires Node.js and generates `dist/sw.js` from the
completed bundle. It precaches HTML, JS, WASM, CSS, manifest, and icons. After
installation completes online, reload once to enter service-worker control;
`/`, `/help`, and `/config` then reload offline using the same cached bundle.
Offline support depends on the browser retaining its cache.

Updates download a complete new bundle and wait until all app tabs close.
An incomplete download leaves the previous version usable. Saved-game storage
is unchanged. Serve production builds over HTTPS or localhost; opening
`dist/index.html` directly does not install a service worker.

Run `npm run test:offline` for generator and worker regression tests. For browser
verification, serve `dist/`, wait for `navigator.serviceWorker.ready`, reload,
then enable offline mode and check all three routes, saved-game resume, and
new-game generation. For updates, keep two tabs open, serve a second build,
verify its worker waits, close both tabs, and reopen the app.

Verified locally in Chromium at a 390×844 viewport: offline routes (including a
query string), saved-game resume, new games, puzzle completion, all precached
assets, and safe multi-tab updates. A
missing required asset rejected the update while preserving the working bundle.
Vercel response headers still need verification on a deployed preview.

## Themes and releases

Tailwind v4 consumes semantic CSS custom properties in `style/input.css`. Light
uses cool slate and blue; dark uses stepped navy backgrounds. Error and hint
backgrounds take precedence over selection, while an inset blue outline keeps
the active cell visible. Matching numbers also have an underline; errors have
an exclamation mark. Original clues use bold text, player entries use blue,
and notes use a muted color. Same-box peers are highlighted alongside row and
column peers. Selection has a strong blue fill and 3px outline; neutral peers and blue
matching highlights separate location from digit scanning. Board text/notes
meet a 5.5:1 product contrast target, above the 4.5:1 WCAG minimum.
Dark surfaces use layered navy rather than black for long playing sessions.
Selecting a filled cell or a Drop number previews every empty cell: brighter
with a dot means legal by current row/column/box rules; diagonal `/` stripes mean
blocked by the selected number, and `\` stripes by another matching number.
Both blocker sources retain nearby blue/slate fills beneath the 2px stripes,
and matching filled numbers have a stronger blue highlight and underline.
Selected-number shading takes precedence when both block a cell.
Settings has three 0–100% shading sliders: selection and its peers/blockers,
matching-number blockers, and available empty cells. Zero uses the normal cell
background without stripes; 100 uses a strong shade. An empty selection also
highlights its row/column/box and available empties outside those groups.
Dots/stripes have independent toggles. Preferences survive reloads/new games;
defaults and reset use 20% selection/matching, 100% available shading, and both toggles.
Explicit saved preferences remain unchanged.
This preview does not reveal the solution or prevent entering a number.

`npm run test:theme` checks text and indicator contrast; this is a color check,
not a complete accessibility audit. New saves retain original clue provenance.
Older saves retain the correct-number locks they already had because original
clues cannot reliably be reconstructed.

See [the contrast review](docs/theme-contrast-review.md) for comparisons and
measured ratios, and [CHANGELOG.md](CHANGELOG.md). The initial shared version is `0.9.0`. Future
PRs bump minor for compatible changes (including added state), or major for
breaking saved-game changes. Keep both manifest/lockfile pairs synchronized.
Declare `Game-state compatibility: compatible` or `breaking` in the PR body;
`npm run test:release` checks the validator and tag protection. Existing release
tags are never moved. GitHub required checks and Vercel preview acceptance still
need repository-level verification.

For the optional Chromium audits, install Playwright outside the repository and
run `node tests/theme-browser.mjs` or `node tests/domino-browser.mjs` with
`PLAYWRIGHT_MODULE` pointing to its module
and `CHROMIUM_PATH` to the browser executable. It writes screenshots under
`/tmp/sudoku-themes` (override with `THEME_SCREENSHOTS`); the domino audit writes
settings screenshots under `/tmp/sudoku-domino` and checks timing, activation,
cancellation, undo/redo, and persistence at phone and desktop widths.

`node tests/ui-browser.mjs` uses the same optional Playwright environment and
checks phone layouts and tablet portrait layouts and browser resizing
from 320 to 1280 pixels in both themes. It writes ten production screenshots
to `docs/screenshots/ui-ux/`
(override with `UI_SCREENSHOTS` to keep ad hoc outputs outside the repository).
`node tests/drop-selection-browser.mjs` checks solved-cell Drop number picking,
completed-number placement/notes, the configuration toggle, old-save loading,
persistence and reset at
phone/tablet sizes in both themes. It writes review captures under
`/tmp/sudoku-drop-selection`.

See [the screenshot gallery](docs/screenshots/ui-ux/README.md).

## Controls and settings layout

Local SVG line icons accompany Portuguese control labels. Header utilities have
44px targets; installation occupies its own row. Time, errors, and pause/resume
sit above the board. Number entry uses two rows on phones and one nine-column
row in tablet portrait, with 52px minimum height; Apagar, Notas, and Drop follow
below, then the two-column assistance buttons and a separate Novo jogo action.
Active modes use pressed states and a stable label, with a separate selected-number indicator for Drop.

On tablets, the board grows to 720px when space allows, with larger digits,
notes, and 64px number buttons. Controls stay below the board at every width.
The installed PWA is locked to portrait; ordinary browser tabs retain the same
stacked layout when resized.

Settings groups highlights, assistances, and sound in a single column. Domino
activation and its empty-cell threshold remain visible; “Ajustar velocidade”
expands the timing controls. Sound offers Desligado, Bip, and Explosão directly.
“Restaurar padrões” restores preferences without changing game progress.

## Difficulty

Levels are graded from a deterministic logical solve trace: singles (Fácil),
locked candidates (Médio), pairs/triples (Difícil), fish/wings/coloring/short
chains (Expert), and longer alternating chains (Mestre). A limited grader that
stalls reports unrated. These are app-specific bands, not SE rating numbers.

New games accept only a matching measured level. Fresh generation is bounded;
verified variations from the bundled bank keep all levels available offline.
The header shows the strongest technique. Older games keep their saved level
and display “jogo anterior”; their progress is preserved.

See [the implemented policy and measurements](docs/difficulty-policy.md) and
[the historical review](docs/difficulty-review.md). Run
`cargo test --release audit_difficulty_settings -- --ignored --nocapture` for
native measurements, or `tests/difficulty-browser.mjs` with the optional
Playwright environment above for offline WASM measurements.

## Android APK distribution

See [Android setup and release acceptance](docs/android.md) for bundled offline APKs,
`just android-*` commands, private signing secrets, saves and GitHub release retries.
Web development does not require an Android SDK. Build support is implemented;
signed publication and physical-device acceptance remain pending.
