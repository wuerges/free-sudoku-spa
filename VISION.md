# Product vision: Sudoku PWA

## Purpose

Build a free, open-source Sudoku game with no ads or trackers, designed first for phones and tablets. Players should be able to start quickly, resume their game, and play offline after a successful online app installation.

The primary interface language is Brazilian Portuguese. Rust and Leptos provide the client-side game and UI, compiled to WebAssembly. Static hosting serves the app; game generation and play do not require a backend.

This document defines product direction and acceptance criteria. It is not a claim that every goal has shipped. See [README.md](README.md) for setup, [docs/architecture.md](docs/architecture.md) for the current implementation, and [AGENTS.md](AGENTS.md) for Codex working instructions.

## Existing implementation

The repository contains:

- A 9×9 board with five selectable difficulty labels: Fácil, Médio, Difícil, Expert, and Mestre.
- A custom Rust generator and backtracking solver. Generation removes rotationally paired cells and checks that removals preserve a unique solution.
- A button-based number pad, pencil notes, drop mode, conflict highlighting, undo/redo, timer and pause, hints, and victory feedback.
- Optional domino cascades with saved timing and empty-cell activation settings, plus sound settings. Cascades use only single candidates from a simple row/column/3×3 box scan and start after correct normal or Drop entries.
- Game, configuration, and help routes; automatic and manually selectable light/dark themes with shared semantic color tokens.
- Game-state serialization to the `sudoku_state` localStorage key.
- A manifest, icons, installation-prompt bridge, and service worker.
- Engine and state unit tests, Trunk build configuration, and Vercel static-hosting configuration.

These statements describe code present in the repository. Browser behavior, performance, and device compatibility still require verification; test presence does not imply complete coverage.

## Product principles

- Make the board and controls comfortable to use by touch without depending on hover.
- Use the custom number pad rather than text input fields for game entry, avoiding unwanted OS keyboard activation.
- Keep puzzles valid and uniquely solvable. Difficulty labels should eventually reflect solving effort rather than clue count alone.
- Preserve saved progress and settings when changing the persisted state format.
- Keep Sudoku algorithms separate from DOM/UI code.
- Keep the app simple, responsive, and usable without an account.
- Treat accessibility, offline reliability, and update behavior as acceptance criteria to verify.

## Known gaps and priorities

### 1. Offline play and updates

**Current behavior:** Trunk generates a content-versioned worker that precaches the complete app shell and assets. Controlled navigation serves the cached HTML for every SPA route, while assets come from the same bundle. Updates wait until all app tabs close; failed installations preserve the previous bundle.

**Verified:** Local Chromium checks at a phone-sized viewport cover offline reload on `/`, `/help`, and `/config`, query-string navigation, saved-game resume, new-game generation and completion, all precached assets, multi-tab updates, and failed-update recovery. The production build and worker regression tests pass.

**Remaining validation:** Verify deployed cache headers and returning-player updates on a Vercel preview, and installation behavior on target Android devices. Offline operation requires successful installation and retained browser storage.

### 2. Difficulty quality

**Current behavior:** Grader v1 records a deterministic logical solve trace and rates its strongest technique. New selections accept only matching measured levels, with bounded fresh generation and a bundled verified fallback bank. Unsupported puzzles stay unrated. Seed replay uses one nonzero RNG stream and is deterministic for the same generator/grader/corpus version. Clue bounds are secondary density constraints; failure is explicit.

**Verified:** Fixed puzzle fixtures and trace soundness checks cover all five levels. Native and offline WASM audits each exercised 100 selections, with actual ratings matching every selection. The bank was independently compared with Sukaku Explainer 1.18.1. See [the policy and measurements](docs/difficulty-policy.md). Existing saves retain progress and their previous labels; new rating metadata has compatible defaults.

**Remaining validation:** Check generation/grading latency on representative Android hardware and collect player feedback to refine the local technique bands. Ratings describe this solver's chosen path; they are not a universal or minimal-difficulty guarantee. Daily-puzzle semantics and cross-version seed replay remain future work.

### 3. Mobile usability and accessibility

**Goal:** Make board selection, notes, drop mode, settings, pause, and resume clear on narrow screens. Support keyboard navigation and accessible descriptions of cells and controls.

**Acceptance:** Verify the affected flows at phone and tablet sizes, check touch target sizes and contrast, and review keyboard and screen-reader behavior. Do not declare compliance based on markup alone.

Color-token tests cover text and indicator contrast in both themes, including a 5.5:1 board-text target and stronger separation of highlight fills. Original clues, player entries, hints, errors, selection, matching numbers, and row/column/box peers have distinct treatments. Selecting a filled cell or Drop digit previews legal empty cells with a brighter fill/dot and uses opposite diagonal stripe directions for empties blocked by the selected occurrence or other matching numbers, using current rules rather than the solution. Three shading sliders (selection/peers, matching blockers, available cells) and dot/stripe checkboxes persist as compatible settings and survive new games. A full keyboard/screen-reader and target-device audit remains open.

The control review adds consistent local line icons, larger action targets,
above-board game status, and a two-column assistance group. Settings groups
highlights, assistances, and sound, with timing behind a native disclosure and
explicit sound choices. Tablet layouts enlarge the board with controls below
it; the installed PWA stays locked to portrait. Chromium audits cover
320–1280px layouts, including tablet portrait and browser resizing, in both themes,
control sizing, keyboard focus/tab order, radio/slider/disclosure operation,
mode states, settings persistence/reset, and installation placement. The
[screenshot gallery](docs/screenshots/ui-ux/README.md) records the production UI.
These checks do not replace a full board keyboard, screen-reader, or physical
phone/tablet audit.

### 4. Persistence and reliability

**Goal:** Resume progress safely and make storage failures understandable without breaking gameplay.

**Acceptance:** Test reload/resume, malformed saved data, and state-format changes. Avoid assuming that storage is always available or that saved state has a fixed small size; undo history affects serialized size.

### 5. Build and deployment consistency

**Current behavior:** Local and Vercel builds use the npm lockfile for Tailwind. Vercel has separate installation and build phases, with pinned project-local Rust/Trunk tooling and locked Cargo dependencies. GitHub Actions checks PRs and publishes version tags/releases after merged PRs on main pass validation.

**Goal:** Use a reproducible toolchain across development and hosting, with suitable compilation, lint, test, and production-build checks.

**Acceptance:** Verify a clean dependency install and production bundle. Verify required checks and Vercel preview integration; documentation must describe the workflow actually present.

## Performance goals

Keep puzzle generation responsive, animation smooth, and download size small on representative mobile devices. Establish measurements before choosing numerical budgets. The previous generation-time, frame-rate, and compressed-WASM-size figures were aspirations, not measured results.

## Future ideas

These are candidates, not committed scope:

- Local statistics and streaks.
- Explanations of hints using human solving techniques.
- A daily puzzle backed by deterministic generation.
- Puzzle import/export and result sharing.
- Additional interface languages.

Prioritize correctness, saved-game compatibility, offline reliability, and usability before expanding features. No implementation deadline or delivery estimate is set by this document.

## Maintaining this vision

Update this document when product decisions change. Move verified behavior into the existing-implementation section and retain remaining work as explicit goals. Keep setup commands and file maps in their linked documents rather than duplicating scaffolding examples here.
