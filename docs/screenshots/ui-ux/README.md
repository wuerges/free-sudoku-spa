# UI/UX screenshots

Production screenshots for the 0.14.0 settings and control review. Both themes
use the same generated puzzle and saved preferences. Timing controls are
collapsed in settings; the browser audit also verifies their expanded state.

| Page | Mobile light (390×844) | Mobile dark (390×844) | Desktop light (1280×900) | Desktop dark (1280×900) |
| --- | --- | --- | --- | --- |
| Game | [View](game-light-390.png) | [View](game-dark-390.png) | [View](game-light-1280.png) | [View](game-dark-1280.png) |
| Settings | [View](settings-light-390.png) | [View](settings-dark-390.png) | [View](settings-light-1280.png) | [View](settings-dark-1280.png) |

Screenshots include the full page, so scrollable content can exceed the viewport
height. Number buttons use two rows on phones, one in tablet portrait, and
the same stacked layout in wide browser tabs. Minimum height is 52px on phones
and 64px on tablets. No horizontal scrolling or control overlap is expected at any audited
width (320, 390, 768, 800, 1024, and 1280 pixels).

Tablet captures use CSS viewports representative of an 11-inch tablet;
physical Xiaomi hardware still needs validation. The installed PWA is locked
to portrait. The audit resizes browser tabs and verifies that progress survives and controls remain below the board.

| Tablet game | Light | Dark |
| --- | --- | --- |
| Portrait (800×1280) | [View](game-light-800.png) | [View](game-dark-800.png) |

To regenerate after `just build`, provide the optional Playwright environment
described in the repository README and run:

```sh
node tests/ui-browser.mjs
```

The audit writes these screenshots. For checks without modifying this gallery,
set `UI_SCREENSHOTS=/tmp/sudoku-ui-review`.
