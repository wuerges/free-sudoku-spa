# Changelog

Entries are pending until the matching GitHub release is published. GitHub
records publication dates. Versions follow saved-game compatibility: compatible
changes and state additions bump minor; incompatible state changes bump major.
The initial shared version is 0.9.0.

## [0.24.0]

### Changed

- Replace Erase with directly accessible Undo and Redo below the color tools; Notes, Drop and Actions remain beside them.

### Fixed

- Identify Clear colors with a slashed palette instead of the New Game reset icon.

## [0.23.0]

### Added

- Complete interface catalogs for Argentine Spanish, Paraguayan Guarani, Brazilian Portuguese and American English, including help, accessibility labels, sharing and history messages.
- Country-flag language choices in Settings. First launch selects a supported browser language or English and saves that choice; manual selections persist independently of games.
- Localized loading screen, document language/title and history dates. All translations are bundled for offline and Android use.
- Feedback when the language preference cannot be saved; the selected language remains usable for the session.

### Changed

- Settings reset preserves the interface language and game progress. Shared links and archived games retain the recipient's local language.

## [0.22.0]

### Added

- Histórico with read-only board miniatures, recent-game details and pin controls.
- Retain the active game, ten unpinned past games and all pinned games locally, including the game replaced by a confirmed shared-link import.
- Resume saved progress with undo/redo or start a separate attempt from original clues, keeping current settings and preserving the previous attempt.
- IndexedDB archives with guarded replacement, storage-error feedback and offline history navigation. Older active saves gain an identity without losing progress.

## [0.21.0]

### Added

- Self-contained Sudoku links from Ações → Compartilhar, with automatically generated current-progress and initial-puzzle snapshots, selectable/copyable URLs and system sharing where available.
- Validated read-only import preview with current-progress or initial-puzzle choices and confirmation, preserving recipient settings and starting paused with empty history.
- Generic Sudoku social preview image; social crawlers do not receive the puzzle fragment.
- Build-time public URL configuration for shared links and social metadata, with automatic Vercel preview/production resolution and explicit overrides for web/APK hosting.

## [0.20.0]

### Changed

- Cell colors follow number targeting: select a cell then apply a color, or use Drop for repeated painting while independently choosing a highlight digit.

### Fixed

- Drop painting, notes and number entry preserve the highlight anchor; notes no longer pick digits from solved cells.

## [0.19.0]

### Added

- Manual green, purple, pink and orange cell marks, tap-again removal, clear-all and undo/redo, saved compatibly with older games.

### Changed

- Larger boards, a compact header, equally sized number/color buttons, and collapsible game actions preserve playing space on phones and 11-inch tablets.

## [0.18.0]

### Added

- Saved 0–100% contrast slider for completed keypad digits, with stronger default shading, outlines and checkmarks in both themes. Completion counts nine correct occurrences; notes remain available.

## [0.17.0]

### Added

- Bundled offline Android APK packaging, portrait Sudoku app, debug builds, lint and emulator CI.
- Signed GitHub release APK and checksum publication with immutable-commit retries and private signing credentials.

### Changed

- Native installations suppress browser PWA prompts and service workers. Android saves use the existing state format in separate app storage.

## [0.16.0]

### Added

- `just dev` provides a single-command debug server with Rust/CSS reload, isolated development output, and a network-only development worker.
- Setup, formatting and tooling-validation targets use the pinned project-local tools.

### Changed

- Local setup, npm engine metadata and CI use Node.js 24.
- `just serve` aliases `just dev`; the local CI target includes formatting and tooling regressions, and cleanup includes development output.

## [0.15.0]

### Added

- Optional selection of the Drop number by tapping a correctly filled cell, enabled by default and saved across reloads and new games. Picking a number leaves progress, notes, errors, and undo/redo history intact.

### Changed

- Completed numbers (nine correct occurrences) are disabled for Drop placement while remaining available for notes. Incorrect duplicates do not count toward completion.

## [0.14.0]

### Changed

- Settings groups highlights, assistances, and sound in readable sections, with domino timing under “Ajustar velocidade” and explicit sound choices.
- Consistent local line icons, labeled controls, larger touch targets, stable notes/Drop labels, and grouped game actions improve mobile navigation.
- Tablet layouts enlarge the board, digits, notes, and number controls; controls remain below the board, and installed apps stay locked to portrait.
- Number buttons use larger digits and 52px minimum height, arranged in two rows on phones for wider touch targets.
- Timer, error count, and pause/resume sit above the board; installation has its own header row and reset uses a neutral preference action.
- Help matches the refreshed controls; saved preferences and game progress remain unchanged.

## [0.13.0]

### Changed

- Default selection and matching shading to 20%, and available-cell shading to 100%.
- Enable domino by default with an activation threshold of 10 empty cells.
- Configuration reset and missing saved settings use these defaults; explicit saved preferences remain unchanged.

## [0.12.0]

### Added

- Saved domino preferences for initial delay, acceleration, minimum delay, and the empty-cell activation threshold (zero means no limit).

### Changed

- Correct normal and Drop-mode entries trigger the same domino behavior, using only single candidates from row, column, and 3×3 box constraints.

### Fixed

- Queued cascades stop after disabling/resetting/changing settings, pausing, starting a new game, undo/redo, hints, or further board/note edits.
- Domino skips candidates inconsistent with the solution instead of propagating incorrect player entries.
- Settings and help describe the actual configurable timing; existing saves retain progress and receive compatible defaults.

## [0.11.0]

### Added

- Versioned logical difficulty ratings and deduction traces, from singles through alternating chains.
- A self-generated, independently calibrated offline puzzle bank for reliable level selection.
- Deterministic seed replay and regression coverage for ratings, deductions, bounds, budgets, and saved games.

### Changed

- Difficulty selection accepts only puzzles with the requested measured rating; the header shows the strongest solving technique.
- Hints are available at every level according to the player's preference.

### Fixed

- Clue removal respects both range bounds, can remove the center, and explicitly rejects unreachable density targets.
- Bounded search and generation attempts prevent endless retries; verified fallback puzzles keep every level available offline.
- Existing saves load with their progress and previous labels intact; new rating metadata defaults to absent for older saves.

## [0.10.0]

### Added

- Difficulty/generation review comparing established solver-based rating systems and outlining a technique-based policy.
- All-setting generation invariants and a manual release-mode audit with clue counts, classifications, and timing data.

### Fixed

- Release-policy unit tests use independent fixtures so future version bumps do not invalidate the initial-release tests.

## [0.9.0]

### Added

- Saved board-highlight preferences: three shading sliders for selection/peers, matching-number blockers, and available cells, plus independent dot and stripe toggles.

- Number-placement preview from filled cells and the Drop keypad: brighter legal empty cells with a dot, solid blue/slate fills with stronger opposite diagonal stripes for selected-number and matching-number blockers.

- Initial client-side Sudoku PWA with notes, hints, undo/redo, difficulty selection, settings, and saved progress.
- Semantic light and navy-dark color tokens, visible selection/error cues, and consistent colors across game, settings, help, and loading screens.
- CI checks for versions, contrast, game-state regressions, and builds; automatic version tags and GitHub releases after qualifying merges into main.

### Changed

- Stronger board contrast in both themes: neutral peers, blue matching numbers, a 3px active outline, clearer grid boundaries, and brighter dark-mode entries/notes.

- Original clues, player entries, and hints have distinct typography and colors; peers include the selected cell's 3×3 box.
- Package versions now share the 0.9.0 baseline, with explicit saved-game compatibility rules for future PRs.
- Reproducible project-local Rust/Trunk tooling and separate Vercel install/build phases ([#2](https://github.com/wuerges/free-sudoku-spa/pull/2)).

### Fixed

- Zero shading now restores the normal cell background and removes that source’s stripes. Selection shading applies to empty-cell row/column/box highlights as well.

- Legacy undo history can erase migrated player entries without leaving their cells locked.

- Offline reload now uses a complete versioned bundle; updates wait for open tabs and preserve the previous bundle on failed installation ([#1](https://github.com/wuerges/free-sudoku-spa/pull/1)).
- Correct player entries remain editable instead of being misclassified as original clues. Legacy saves preserve their existing locked cells when original provenance is unavailable.
- Conflict backgrounds remain visible when a cell is selected or highlighted; notes and control labels use tested contrast pairs.
