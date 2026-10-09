# Theme contrast review

Reviewed 2026-10-09 for the existing theme PR (#3). Styling remains Tailwind v4
with semantic CSS custom properties in `style/input.css`. No saved-state change.

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

Neutral slate peers, green matching digits, and blue selection now distinguish
location from digit scanning. Matching digits retain their underline; selection
has a 3px inset outline that survives error/hint backgrounds. Hints remain amber
and errors red with an exclamation marker. This keeps color from carrying every
meaning alone. Dark cells use deep navy, with brighter player entries/notes and
higher-contrast box boundaries; light entries/notes and boundaries are darker.

The outline is the primary active-cell cue in dark mode. Its selected-fill
contrast is 10.39:1 (light: 5.74:1). Tinted fills intentionally remain softer than
text and outlines so a whole highlighted row does not overwhelm the puzzle.

| Contrast pair | Light | Dark |
| --- | ---: | ---: |
| Peer / default cell | 1.23:1 | 1.57:1 |
| Matching / default cell | 1.21:1 | 1.56:1 |
| Selected / peer | 1.46:1 | 1.47:1 |
| Selected / matching | 1.49:1 | 1.46:1 |
| Given / selected | 9.90:1 | 14.11:1 |
| Player entry / selected | 5.74:1 | 10.39:1 |
| Notes / selected | 5.74:1 | 11.97:1 |

`npm run test:theme` checks board text at ≥5.5:1 across every cell background,
other text at ≥4.5:1, and grid/selection indicators at ≥3:1. New hierarchy tests
also require peer/matching fills versus default ≥1.2:1 and selection versus
those fills ≥1.4:1. The latter are product targets, not WCAG requirements.
[WCAG text contrast](https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html)
and [non-text contrast](https://www.w3.org/WAI/WCAG21/understanding/non-text-contrast.html)
remain the accessibility references. Token checks do not constitute a full
accessibility audit or guarantee individual perception on every display.

Production browser verification covers both themes at 390×844 and 1280×900,
all routes, highlights, selected errors/hints, editable entries, undo, legacy
saves, and horizontal overflow. Screenshots are in `docs/themes/`.
