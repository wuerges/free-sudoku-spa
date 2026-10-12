# Game control placement

Production Chromium captures for version 0.24.0. Digits sit above colors;
Undo, Redo, Notes, Drop and Actions share the bottom row. Clear colors uses
a slashed palette. Undo/Redo retain localized accessible names and tooltips.

| Viewport | Light | Dark |
| --- | --- | --- |
| 320×844 phone, English | [Phone](phone-light.png) | [Phone](phone-dark.png) |
| 800×1280 tablet, Portuguese | [Tablet](tablet-light.png) | [Tablet](tablet-dark.png) |

Generate phone captures with `node tests/localization-browser.mjs` and tablet
captures with `node tests/ui-browser.mjs`, using the optional Playwright setup
in the root README. The audits check controls, persistence and responsive
layouts; these captures do not establish physical-device or screen-reader acceptance.
