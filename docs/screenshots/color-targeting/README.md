# Coloring and Drop highlight review

Production Chromium captures show selection-first coloring, Drop painting with
an independent highlight digit, and Drop notes with a stable selection anchor.
The deterministic board contains clues, hints, errors and all four manual colors.

| Flow | Phone 384×700, light | Phone 384×700, dark | Tablet 800×1280, light | Tablet 800×1280, dark |
| --- | --- | --- | --- | --- |
| Select a cell, then apply green | [Light](selection-light-384x700.png) | [Dark](selection-dark-384x700.png) | [Light](selection-light-800x1280.png) | [Dark](selection-dark-800x1280.png) |
| Drop painting, digit 5 selected | [Light](painting-light-384x700.png) | [Dark](painting-dark-384x700.png) | [Light](painting-light-800x1280.png) | [Dark](painting-dark-800x1280.png) |
| Drop notes, digit 3 selected | [Light](notes-light-384x700.png) | [Dark](notes-dark-384x700.png) | [Light](notes-light-800x1280.png) | [Dark](notes-dark-800x1280.png) |

The outlined cell in row 1, column 3 remains the anchor while other cells are
painted or receive notes. Painting keeps the chosen digit; changing that digit
changes matching/placement highlights without leaving the color tool. Tapping
the active color returns to number/note entry. Without Drop, palette clicks
apply immediately to the selected cell and never activate a tool.

Reproduce with `just build`, then `node tests/cell-colors-browser.mjs` using the
optional `PLAYWRIGHT_MODULE` and `CHROMIUM_PATH` documented in README.
`COLOR_SCREENSHOTS` chooses the output directory (default `/tmp/sudoku-cell-colors`).
The audit also covers 320×844, 384×824, 412×915 and 960×1280 in both themes,
legacy history, same-color removal, undo/redo, completed-digit inspection,
reload/offline resume, resizing and enlarged text. These are viewport emulations;
physical Galaxy S23 Ultra and 11-inch tablet acceptance remains pending.
