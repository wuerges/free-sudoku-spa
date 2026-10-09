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

The 3px outline is the primary active-cell cue in both themes: selection and
its directly related cells share the same shading slider. At maximum, selection
has 1.80:1 contrast against the normal cell in light mode and 1.87:1 in dark.
Matching fills have 1.81:1 and 1.63:1 respectively. Notes on the maximum selected
fill have 8.11:1 (light) / 6.82:1 (dark); player entries have 7.24:1 / 5.92:1.

`npm run test:theme` checks board text at ≥5.5:1 across every maximum fill and
stripe color, other text at ≥4.5:1, and grid/selection indicators at ≥3:1.
Maximum selection, matching, and availability shades versus normal require
≥1.5:1; stripes versus maximum blocker fills require ≥1.3:1. These are product
targets, not WCAG requirements. Tests also sample all three sliders’ blends
at 0/25/50/75/100% to retain text/grid contrast.
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
retain their original/hint/error/matching treatments. Selecting an empty cell in normal mode shades its row/column/box with the
selection slider and outside empty cells with the availability slider, without
claiming a number is valid. Clearing a Drop digit clears the digit preview.
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

Settings has exactly three 0–100% sliders:

1. Selection shading covers the selected cell/number and its row/column/box
   peers, including when an empty cell is selected.
2. Matching shading covers other matching filled numbers and empty cells they
   block; selected-source shading wins overlaps.
3. Available shading covers empty cells not blocked by the inspected digit,
   or outside selected units when inspecting an empty cell.

Each slider starts from the normal cell background at zero and reaches a strong
blue/slate fill at 100. Zero also suppresses that source’s stripes, even with
stripes enabled. Selection outline, matching underlines, errors/hints, and the
dot toggle remain independent. Dots/stripes retain their separate checkboxes.
Default selection/matching values are 100; availability defaults to zero.
The new availability field defaults to zero in existing two-slider saves.

Preferences survive saving/loading and new games. Older or partial settings use
compatible defaults; reset preserves the current game. Browser checks verify
all three sliders independently, entirely unshaded zero values, empty-cell
selection, checkbox effects, reload persistence, reset, and mobile layout.
Board and settings screenshots are in `docs/themes/`.
