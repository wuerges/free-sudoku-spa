# Drop, Notes and highlight controls

Production Chromium captures for version 0.25.0. Notes is 50% wider than its
neighbors, with the same height. Selecting a color enables Notes; leaving
painting keeps Notes on. Normal selection can be cleared with a second tap.
Drop highlights follow only the chosen digit and retain no cell anchor.

| Viewport | Light | Dark |
| --- | --- | --- |
| 320×844 phone, English, unselected | [Phone](phone-light.png) | [Phone](phone-dark.png) |
| 320×844 phone, English, Drop painting | [Painting](phone-painting-light.png) | [Painting](phone-painting-dark.png) |
| 320×844 phone, English, saved No Drop setting | [Settings](drop-settings-light.png) | [Settings](drop-settings-dark.png) |
| 800×1280 tablet, Portuguese, unselected | [Tablet](tablet-light.png) | [Tablet](tablet-dark.png) |

Captures use production Chromium with transitions completed. Regression coverage
comes from `tests/localization-browser.mjs` and `tests/ui-browser.mjs`.
Use the optional Playwright setup in the root README and temporary screenshot
output directories. `UI_WIDTHS=320,800` limits the layout audit to phones/tablets;
the audit checks Notes proportions with Undo/Redo visible and hidden.
Automated checks do not establish physical-device or screen-reader acceptance.

Settings offers Drop only, No Drop and Both (the default). Fixed choices remove
the Drop button. `tests/drop-availability-browser.mjs` covers all four languages,
phone/tablet viewports, both themes, all hidden-button layout combinations,
keyboard choices, legacy saves and offline persistence.
