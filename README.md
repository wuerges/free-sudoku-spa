# Sudoku PWA

Sudoku Progressive Web App built with Rust, Leptos, and WebAssembly. Supports offline reload after a successful online installation and targets installation on Android.

## Tech

- **Rust** → WASM via `wasm-bindgen`
- **Leptos** (CSR) → reactive UI
- **Trunk** → build & bundle
- **Tailwind CSS v4** → styling
- **Vercel** → deploy (static)

## Setup

Use Node.js 22 and npm. On Linux x86_64 or aarch64, install the pinned build
stack and locked CSS dependencies without global Rust or Trunk installations:

```bash
sh scripts/setup-build.sh
just build
```

Setup installs Rust 1.98.1 with the WASM target and checksum-verified Trunk
0.21.14 under the ignored `.build-tools/` directory. It runs `npm ci`; both
Vercel and `just build` use these project-local tools and the same npm lockfile.
Network access is required for a clean setup. Rerunning setup reuses installed
build tools while reinstalling the locked npm dependencies.

For development commands (`just serve`, `just test`, `just check`), install
Rust with rustup and Trunk 0.21.14 in your normal shell. `rust-toolchain.toml`
pins the compiler and WASM target; Node/npm supply CSS tooling. The project-local
bootstrap supports Linux only; normal development tooling can be used on other
platforms.

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
- Hint system available at every level according to player settings
- Win detection with fireworks + balloons animation
- Dark/light mode (auto + manual toggle)
- PWA manifest, install prompt, and versioned offline app-shell caching


## PWA

- `manifest.json` — installable, standalone, portrait
- `sw.js` — build-time template for versioned app-shell and asset caching
- Install prompt via `beforeinstallprompt` (Android Chrome/Edge)

## Deploy

Vercel uses the Other framework preset (`framework: null`) and Node.js 22.
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

Run `just css-watch` in a second terminal when changing Tailwind styles during
`just serve`. Run `cargo fmt --check`, `just test`, and `just check` for Rust
changes, and `just build` to verify the production bundle.

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
reset restores 100% selection/matching, 0% available shading, and both toggles.
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

For the optional Chromium audit, install Playwright outside the repository and
run `node tests/theme-browser.mjs` with `PLAYWRIGHT_MODULE` pointing to its module
and `CHROMIUM_PATH` to the browser executable. It writes screenshots under
`/tmp/sudoku-themes` (override with `THEME_SCREENSHOTS`).

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
