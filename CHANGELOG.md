# Changelog

Entries are pending until the matching GitHub release is published. GitHub
records publication dates. Versions follow saved-game compatibility: compatible
changes and state additions bump minor; incompatible state changes bump major.
The initial shared version is 0.9.0.

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
