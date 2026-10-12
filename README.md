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

Run `node tests/effects-browser.mjs` after `just build` to check native Web Audio
recovery, saved volume/balloon controls, and phone/tablet victory effects in both
themes. Physical Android lifecycle and audible loudness acceptance is tracked in
`ISSUES.md` (ISSUE-012).

## Project

```
src/
├── main.rs               # WASM entry
├── i18n.rs               # interface language, labels, and named messages
├── app.rs                # root component
├── sudoku_engine.rs      # generation, solving, validation
├── game_history.rs       # local archive policy and safe game replacement
├── audio.rs              # Rust synthesis and shared Web Audio lifecycle
├── celebration.rs        # Rust canvas fireworks and balloons
├── effects.rs            # saved volume and balloon preferences
├── state.rs              # reactive game state (RwSignal)
├── serde_helpers.rs      # serde for large arrays
└── components/
    ├── cell.rs           # single grid cell
    ├── sudoku_grid.rs    # 9×9 grid
    ├── number_pad.rs     # 1-9 + undo/redo + note
    ├── game_controls.rs  # status bar, actions menu
    ├── game_page.rs      # game route layout
    ├── config_page.rs    # config toggles (undo, auto-notes, hint, domino, sound)
    ├── help_page.rs      # help / instructions
    ├── header.rs         # dark mode, install button
    └── icon.rs           # shared local line icons
```

## Interface languages

The entire interface supports 🇦🇷 Español (Argentina), 🇵🇾 Avañe’ẽ (Paraguái),
🇧🇷 Português (Brasil), and 🇺🇸 English (United States). On first launch, the
app selects the first supported language in browser preference order, including
base-language matching (for example, `es-MX` selects `es-AR`). If browser settings
are missing or unsupported, it selects English. The resolved choice is immediately
saved to the separate `sudoku_locale` localStorage key.

Settings offers four country-flag language buttons with native language names.
Your saved selection takes precedence on later launches; browser language changes
do not override it. Settings reset preserves your language. Games, history and
shared links do not store or replace it. If storage is unavailable, a localized
notice explains that the choice applies only to the current session.

All catalogs are bundled for offline play and APK staging. History dates use the
selected locale; browsers without Guarani date support use Spanish Paraguay for
dates. Technical documentation stays in English. Native-speaker review of new
translations remains open in `ISSUES.md`.

Run `npm run test:i18n` for catalog, selection, persistence and offline asset
regressions. After `just build`, run `node tests/localization-browser.mjs` using
`PLAYWRIGHT_MODULE` and `CHROMIUM_PATH` as described below. It checks all four
languages on phones/tablets in both themes, fallback and storage failures, game
preservation, sharing, and offline routes. Screenshots go to
`/tmp/sudoku-localization` (override with `I18N_SCREENSHOTS`).
See the [localization screenshot gallery](docs/screenshots/localization/README.md)
for flag selectors, gameplay and complete Settings pages in both themes.

## Features

- 5 difficulty levels (Easy → Master) with unique-solution puzzles generated client-side in Rust/WASM
- **Efeito Dominó**: after a correct normal or Drop entry, auto-fills empty cells with exactly one candidate from a simple row/column/3×3 box scan. Saved settings control initial delay, acceleration, minimum delay (defaults: 600ms → *0.8 → min 100ms), and an empty-cell threshold (enabled by default at 10 empty cells; 0: no limit; positive: at most that many empties after the entry). Cascades stop on pause, game replacement, settings changes, or further board/note edits
- Sound on correct guess: beep, explosion (default), or off
- Pencil-mark notes mode with per-cell candidates
- **🎯 Drop mode**: select a number, then tap cells to place or note it in bulk
- In normal Drop (outside Notes/painting), tap a correctly filled cell to pick its number without editing it. Disable “Selecionar número pelas células resolvidas” in Settings to keep the previous behavior. The preference defaults to enabled and survives reloads/new games.
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
uses cool slate and blue; dark uses stepped navy backgrounds. Without manual marks, error and hint
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

Local SVG line icons accompany localized control labels. The compact header
combines time, errors, pause and utility icons, with the title/difficulty below.
Header icons use 36px targets; entry and action controls retain at least 44px.
Number buttons and the five color tools have the same height: 44px on phones,
48px on tablets. Digits use two rows on phones and one nine-column row above
640px; the color palette sits directly below them.

Undo, Redo, Notes, Drop and Actions share one row below the color tools.
Undo/Redo use icon buttons with localized accessible names and respect their
Settings toggle. Clear colors uses a slashed-palette icon and only removes manual
marks. Actions opens a dismissible panel over the board with Auto notes, Hint,
History, Share and New Game. Close it with
its close button, Escape, or a tap outside. The panel keeps keyboard focus
returning to Ações and does not shrink the board. Active entry modes expose
pressed states, with a separate selected-number indicator for Drop. Notes is
50% wider than the other buttons, with the same height. Tap a selected cell
again in normal mode to deselect it and remove analytical highlights; errors,
hints and manual colors remain visible. Without selection, entry and direct
coloring have no target.

Settings → Assistances → Drop modes offers Drop only (always enabled, button
hidden), No Drop (always disabled, button hidden), and Both (default, toggle
button available). The choice survives reloads and game changes; restoring
Settings defaults selects Both. With Drop hidden, the action row closes the gap
and keeps Notes 50% wider than the remaining buttons.

Drop on/off and its chosen digit survive reloads, including offline reloads.
The active painting color is transient. New games, imports and history restores
keep Notes/Drop on/off but clear the digit, color and cell selection. Switching
Drop on or off also clears selection; Drop previews follow only its digit.

The square board uses the space left by the compact controls, growing up to
840px on tablets. Dynamic viewport height and safe-area insets support phone
browser bars and installed use. Controls stay immediately below the board.
Normal phone/tablet play fits at the verified viewports; short screens and
enlarged text may scroll to keep controls reachable. The installed PWA remains
locked to portrait; ordinary browser tabs can resize.

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

## Completed keypad digits

A keypad digit is marked with a check when all nine occurrences are correct.
Settings → “Números concluídos” offers “Contraste dos números concluídos” from
0–100% (default 100%), adjusting its shading and outline in both themes. At zero,
the check still identifies completion. Selected Drop digits retain their selected
appearance, and completed digits remain usable in Drop notes mode. The setting
survives reloads and new games; restoring defaults preserves board progress.
Older saves receive the default without changing notes, history or settings.

With the optional Playwright environment described above, run
`node tests/completed-contrast-browser.mjs` after `just build` to check legacy
saves, slider endpoints/midpoint, notes/Drop, persistence, reset and phone/tablet
layouts in both themes. Captures go to `/tmp/sudoku-completed-contrast`.

## Manual cell colors

With Drop off, select a cell and tap Verde, Roxo, Rosa or Laranja to color it.
With Drop on, choose a color and tap cells to paint repeatedly. The palette
avoids blue, which already identifies game highlights. Choosing a color keeps
the Drop digit. Number buttons change that digit while
painting stays active, including completed digits. Tap the active color again
to resume Notes entry. Selecting any color enables Notes; leaving painting keeps
Notes on. The first Notes tap while painting exits painting with Notes on;
a subsequent tap disables Notes. Toggling Drop also exits painting and clears
cell selection. Drop highlights follow only the selected digit, without an anchor.
Normal Drop still supports picking a solved cell when enabled in Settings;
painting and Notes never pick a digit from a target cell.
Applying the same color again removes that cell’s mark; “Limpar cores” clears
all marks in one undoable action. Painting includes original clues and hints without changing
values or notes. It is unavailable while paused or after victory.

Marks survive reloads, offline resume and configuration reset; a new game clears
them. Undo/redo restores colors alongside numbers and notes. Existing saves and
history load with no marks. Manual colors use the cell background without an edge marker. Selection/focus
outlines, matching underlines, placement patterns, and error/hint text remain
visible over the fill.

Run `node tests/cell-colors-browser.mjs` with the optional Playwright environment
described above after `just build`. It checks painting, legacy saves, history,
mode changes, equal button heights, larger boards, collapsible actions, resizing,
enlarged text and offline reload in both
themes. `COLOR_SCREENSHOTS` overrides the default `/tmp/sudoku-cell-colors`.
See [the targeting review captures](docs/screenshots/color-targeting/README.md) and
[the earlier layout review](docs/screenshots/cell-colors/README.md).

## Sharing Sudoku

Ações → Compartilhar automatically creates a fixed current-progress snapshot.
Selecting Estado atual or Sudoku inicial immediately regenerates the link from
the game at that moment. Timer ticks do not change the displayed link.
Web and APK links target the public origin resolved at build time (see below).
No backend or account is involved. The fragment includes the solution and excludes
preferences/history. Recipients choose Estado atual to continue the received
progress or Sudoku inicial
to restart from its recorded clues, see a validated read-only preview and confirm
replacement; settings stay local, history is cleared and play starts paused.
Cancel leaves progress intact. Confirmation/cancellation removes the fragment.
Older saves use their recorded clue set, which may include previously locked
correct entries. Social cards use `public/social-sudoku.png`, a generic board;
fragments are unavailable to server-side preview crawlers.

Run `node tests/sharing-browser.mjs` after `just build` with the optional
Playwright environment described above. Review captures live in
[the sharing gallery](docs/screenshots/sharing/README.md).


### Public URL at build time

Set an explicit public origin for web or APK builds:

```bash
SUDOKU_PUBLIC_URL=https://sudoku.example.com just build
SUDOKU_PUBLIC_URL=https://sudoku.example.com just android-release
```

Resolution order is an explicit `SUDOKU_PUBLIC_URL`, then
`https://VERCEL_PROJECT_PRODUCTION_URL` **only when `VERCEL_ENV=production`**,
then `https://VERCEL_URL`, then `https://free-sudoku-spa.vercel.app`.
Vercel sets its production URL even on previews; the environment condition keeps
preview/staging links on their own deployment. Enable **Automatically expose
System Environment Variables** in Vercel Settings → Environment Variables
([Vercel documentation](https://vercel.com/docs/environment-variables/system-environment-variables)).
An explicit override applies to whichever environment it is configured for;
set it for Production only if previews should keep their deployment URLs.
Preview links inherit deployment access restrictions.

The setting accepts an absolute HTTP/HTTPS origin with an optional trailing slash.
Credentials, subdirectory paths, query strings, fragments and empty overrides
fail the build. Hosting at a subdirectory remains unsupported. It is public
configuration, not a secret. Both compiled sharing links and staged Open Graph/X
metadata use the same resolver. Changing the environment requires rebuilding;
Cargo tracks these values so no clean is necessary. Node.js is required for
Rust builds as well as Trunk tooling. Restart `just dev` after changing variables.
Outside Vercel, APK builds use the explicit override or the production fallback,
never the native `https://localhost` origin.

For a configured build, run the sharing browser audit with
`SHARING_EXPECTED_ORIGIN=https://sudoku.example.com`. Set `SHARING_SCREENSHOTS`
to a temporary directory to retain the checked-in default captures.
`tests/public-url-browser.mjs` checks configured links/metadata in both web and
Android-staged assets; it requires `SHARING_EXPECTED_ORIGIN` and optionally
`SHARING_BUILD_DIR` (default `dist`), with the same Playwright environment.


## Local game history

Ações → Histórico opens `/history` with miniature boards, difficulty, elapsed
time, errors, completion and last-played dates. Jogo atual is kept separately;
Recentes retains ten unpinned past games, and Fixados retains every pinned game
without automatic expiration. Pin the current game or any saved game. Unpinning
an older game requires confirmation when it would immediately leave the ten-game
window. Browsing or pinning does not change its recency.

Starting a new game, confirming a shared-link import or opening a saved game
archives the outgoing progress first. Previewing/cancelling a link leaves the
current game unchanged. Estado atual resumes the same identity with undo/redo;
Sudoku inicial creates a separate attempt and retains the previous one (and its
pin). Restarting marks the source as recently used so it is not immediately
removed at the ten-game limit. Both history load modes preserve current settings,
clear temporary inputs/selection and start paused. Completed games remain completed.

History is versioned in the `sudoku_game_history` IndexedDB database. The active
`sudoku_state` save remains in localStorage, with a defaulted `game_id` assigned
to older saves without changing progress. Archive failures block replacement and
show an error. Failed cleanup retains extra games and retries on reopening
history. Invalid archive records are reported and retained rather than deleted.
Pinned games have no application expiration, but browser data clearing, device
loss and browser storage limits still apply. Web/PWA and APK storage are separate;
there is no account or synchronization.

Run `node tests/history-browser.mjs` after `just build` using the optional
Playwright environment above. `HISTORY_SCREENSHOTS` overrides the default
`docs/screenshots/history`. The audit checks retention, pins, identity/undo resume,
separate restarts, safe shared-link imports, save failures and offline `/history`.

`node tests/drop-availability-browser.mjs` checks all three Drop policies in all
four languages, both themes and phone/tablet layouts, including older saves,
Notes width with hidden controls, keyboard radios, reset, new games and offline
reload. It uses the optional Playwright setup above and writes captures under
`/tmp/sudoku-drop-availability` (override with `DROP_AVAILABILITY_SCREENSHOTS`).
