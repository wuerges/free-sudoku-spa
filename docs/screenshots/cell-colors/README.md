# Cell colors and compact layout review

Production Chromium captures use deterministic test puzzles to show all four
manual colors alongside selection, notes, clues, errors and hints. Manual marks are cell
backgrounds without edge bars; applying the same color twice removes it. These are
viewport emulations, not photographs or physical-device acceptance.

| Viewport | Light | Dark |
| --- | --- | --- |
| Phone, reduced browser height — 384×700 | [Light](painting-light-384x700.png) | [Dark](painting-dark-384x700.png) |
| Phone, taller app viewport — 384×824 | [Light](painting-light-384x824.png) | [Dark](painting-dark-384x824.png) |
| 11-inch tablet layout — 800×1280 | [Light](painting-light-800x1280.png) | [Dark](painting-dark-800x1280.png) |

The larger board, equal-height number/color buttons and entry modes fit inside
each capture. Numbers and colors are 44px high on phones and 48px on tablets;
header icons use 36px targets. Desfazer, Refazer, Auto notas, Dica and Novo jogo
are inside Ações, which opens over the board without shrinking it.

| Expanded actions | Light | Dark |
| --- | --- | --- |
| Phone — 384×700 | [Light](actions-light-384x700.png) | [Dark](actions-dark-384x700.png) |
| Tablet — 800×1280 | [Light](actions-light-800x1280.png) | [Dark](actions-dark-800x1280.png) |

Browser regression also covers 320×844, 412×915 and 960×1280, viewport height
changes, installation in the compact header, legacy saves and history, painting,
same-color removal, clear-all, undo/redo, action-panel dismissal, mode transitions, settings reset, new games,
reload and offline resume. Enlarged text may scroll without hiding controls.
The existing UI audit covers additional desktop/landscape widths.

Reproduce with `just build`, then `node tests/cell-colors-browser.mjs` using the
optional `PLAYWRIGHT_MODULE` and `CHROMIUM_PATH` described in the repository
README. Captures default to `/tmp/sudoku-cell-colors`; `COLOR_SCREENSHOTS` changes
the destination. Copy only reviewed captures into this directory.

Physical Galaxy S23 Ultra and 11-inch tablet verification remains pending.
