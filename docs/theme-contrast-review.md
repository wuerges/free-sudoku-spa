# Theme contrast review

Reviewed 2026-10-09 for the existing theme PR (#3). Styling remains Tailwind v4
with semantic CSS custom properties in `style/input.css`. Compatible saved highlight preferences are described below.

## Comparisons

- [Sudoku.com](https://sudoku.com/): direct browser inspection of the light board
  showed dark digits, heavier 3×3 boundaries, neutral peer shading, stronger
  matching shading, and a blue selected cell. This supports a clear hierarchy;
  it does not establish that their palette meets accessibility requirements.
- [Sudoku Maven's own guide](https://sudokumaven.com/how-to-play-sudoku/) distinguishes
  softly highlighted row/column/box peers from matching numbers and matching
  pencil marks. Its documentation supports keeping these roles distinct.
- [Apple News Sudoku's guide](https://support.apple.com/en-ph/guide/iphone/iph9b53d2906/ios)
  documents matching-number highlighting from either the keypad or a filled
  square. Digit scanning is a separate task from tracking the active cell.

These are interaction/design references, not palettes copied from competitors.
Only Sudoku.com's light board was visually compared; no claim is made about
competitor dark-theme contrast.

## Adjustments

Neutral slate peers, blue matching digits, and blue selection now distinguish
location from digit scanning. Matching digits retain their underline; selection
has a 3px inset outline that survives error/hint backgrounds. Hints remain amber
and errors red with an exclamation marker. This keeps color from carrying every
meaning alone. Dark cells use deep navy, with brighter player entries/notes and
higher-contrast box boundaries; light entries/notes and boundaries are darker.

The outline is the primary active-cell cue in dark mode. Its selected-fill
contrast is 10.39:1 (light: 7.24:1). Tinted fills intentionally remain softer than
text and outlines so a whole highlighted row does not overwhelm the puzzle.

| Contrast pair | Light | Dark |
| --- | ---: | ---: |
| Peer / default cell | 1.23:1 | 1.57:1 |
| Matching / default cell | 1.54:1 | 1.69:1 |
| Selected / peer | 1.46:1 | 1.47:1 |
| Selected / matching | 1.17:1 | 1.59:1 |
| Given / selected | 9.90:1 | 14.11:1 |
| Player entry / selected | 7.24:1 | 10.39:1 |
| Notes / selected | 8.11:1 | 11.97:1 |

`npm run test:theme` checks board text at ≥5.5:1 across every cell background,
other text at ≥4.5:1, and grid/selection indicators at ≥3:1. New hierarchy tests
also require peers versus default ≥1.2:1, matching digits versus default
≥1.5:1, selection versus peers ≥1.4:1 and versus matching digits ≥1.1:1.
The stronger matching-digit fill intentionally sits closer to the selected fill;
the high-contrast 3px outline identifies selection independently. These are
product targets, not WCAG requirements.
[WCAG text contrast](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html)
and [non-text contrast](https://www.w3.org/WAI/WCAG21/understanding/non-text-contrast.html)
remain the accessibility references. Token checks do not constitute a full
accessibility audit or guarantee individual perception on every display.

Production browser verification covers both themes at 390×844 and 1280×900,
all routes, highlights, selected errors/hints, editable entries, undo, legacy
saves, and horizontal overflow. Screenshots are in `docs/themes/`.

## Number-placement preview

A filled-cell selection or active Drop digit previews all empty cells against
every existing occurrence of that digit, including box restrictions. Available
cells are brighter and carry a small dot; blocked cells use `/` diagonal stripes for selected-occurrence peers and `\`
stripes for other matching-number peers, in the same blue/slate palette.
Selected-source stripes win overlaps. Drop digits without a selected matching
occurrence use `\` stripes for all blockers. Filled cells
retain their original/hint/error/matching treatments. Selecting an empty cell
in normal mode clears the preview; clearing the Drop digit also clears it.
Matching notes and matching filled cells use the same inspected digit.

There is no visible shading legend. Per-cell accessible labels describe availability and blocker source.
Availability means legal by current row/column/box rules, not a guaranteed
answer; neither the preview nor its tests consult the solution. Notes and
number-entry behavior remain unchanged. Contrast tests include both new fills
and require at least 1.5:1 separation between available and each blocked fill as a product target.
State regression tests check distant occurrences, box-only restrictions, no
solution dependency, Drop precedence/reset, and blocker-source overlap; browser tests check all 81
cells for normal selection and multiple Drop digits in both themes.

The final preview replaces violet/green with blue/slate and adds opposite
diagonal hatch directions rather than relying on hue to distinguish blockers.
Stripe colors are included in text/grid contrast tests. Browser checks verify
both gradient directions and the absence of the visible shading legend.

Both blocker sources retain solid fills in nearby blue/slate shades underneath
their patterns. Matching filled numbers have a stronger blue fill and underline.
Hatch strokes are 2px wide (12px repeat); contrast tests require at least 1.3:1
between each blocker fill and its stripe while preserving readable notes.

## Configurable highlights

Settings exposes separate 0–100% sliders for selected-source and matching-source
shading, plus independent dot and stripe checkboxes. At 0% the corresponding
fill blends fully into the available-cell fill; at 100% it uses the palette
above. Stripes are controlled independently. Defaults retain full shading,
dots, and stripes. Lower fill separation is an intentional player preference.
Tests sample text/grid contrast at 0/25/50/75/100% in both themes.

Preferences survive saving/loading and new games. Older or partial settings
use compatible defaults; reset preserves the current game. Browser checks
verify slider independence, checkbox effects, reload persistence, reset, and
mobile layout. Board and settings screenshots are in `docs/themes/`.
