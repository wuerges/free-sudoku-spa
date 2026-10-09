# Product vision: Sudoku PWA

## Purpose

Build a free, open-source Sudoku game with no ads or trackers, designed first for phones and tablets. Players should be able to start quickly, resume their game, and eventually play reliably offline after the app has loaded once.

The primary interface language is Brazilian Portuguese. Rust and Leptos provide the client-side game and UI, compiled to WebAssembly. Static hosting serves the app; game generation and play do not require a backend.

This document defines product direction and acceptance criteria. It is not a claim that every goal has shipped. See [README.md](README.md) for setup, [docs/architecture.md](docs/architecture.md) for the current implementation, and [AGENTS.md](AGENTS.md) for Codex working instructions.

## Existing implementation

The repository contains:

- A 9×9 board with five selectable difficulty labels: Fácil, Médio, Difícil, Expert, and Mestre.
- A custom Rust generator and backtracking solver. Generation removes rotationally paired cells and checks that removals preserve a unique solution.
- A button-based number pad, pencil notes, drop mode, conflict highlighting, undo/redo, timer and pause, hints, and victory feedback.
- Optional domino cascades and sound settings.
- Game, configuration, and help routes; automatic and manually selectable light/dark themes.
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

**Current behavior:** `sw.js` precaches only `/manifest.json`. It uses network-first navigation and cache-first asset lookup, but does not store fetched HTML, WASM, JS, CSS, or icons. Full offline play after an initial visit is therefore not established.

**Goal:** Cache a complete app shell and its required assets, provide an offline navigation fallback, and make deployed updates reach returning players without mixing incompatible bundle versions.

**Acceptance:** After a successful online load, reload and navigate the app offline, start and finish a puzzle, and resume saved progress. Then deploy a changed bundle and verify that returning players receive a consistent update. Check hosting cache headers alongside service-worker behavior.

### 2. Difficulty quality

**Current behavior:** Difficulty uses clue-count targets. The generator may stop before reaching the requested range when further removals would violate uniqueness; it does not grade human solving techniques. The stored seed is not a reproducible puzzle-generation API.

**Goal:** Keep unique solutions while making difficulty labels consistent with solving effort.

**Acceptance:** Validate generated puzzles across all levels, record actual clue counts and generation times, and introduce technique-based grading only with meaningful tests. Do not promise a guaranteed clue range or a reproducible daily puzzle with the current generator.

### 3. Mobile usability and accessibility

**Goal:** Make board selection, notes, drop mode, settings, pause, and resume clear on narrow screens. Support keyboard navigation and accessible descriptions of cells and controls.

**Acceptance:** Verify the affected flows at phone and tablet sizes, check touch target sizes and contrast, and review keyboard and screen-reader behavior. Do not declare compliance based on markup alone.

### 4. Persistence and reliability

**Goal:** Resume progress safely and make storage failures understandable without breaking gameplay.

**Acceptance:** Test reload/resume, malformed saved data, and state-format changes. Avoid assuming that storage is always available or that saved state has a fixed small size; undo history affects serialized size.

### 5. Build and deployment consistency

**Current behavior:** Local CSS builds use the npm lockfile. `build.sh` installs a standalone Tailwind CLI for Vercel. No GitHub Actions workflow is checked in.

**Goal:** Use a reproducible toolchain across development and hosting, with suitable compilation, lint, test, and production-build checks.

**Acceptance:** Verify a clean dependency install and production bundle. Add CI or change hosting only as an explicit implementation task; documentation must describe the workflow actually present.

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
