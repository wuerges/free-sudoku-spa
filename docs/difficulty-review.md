# Sudoku difficulty and generation review

Historical baseline; see [the implemented v1 policy](difficulty-policy.md).

Reviewed 2026-10-09 against PR #3 (`107ad240`). This PR adds measurements and
invariant tests; it preserves generation, labels, assistance settings, and saves.
The release is `0.10.0` under the compatible-change version policy.

## Findings in this app

The generator fills a complete grid with randomized backtracking, then tries
removing shuffled 180°-symmetric pairs. It retains each removal only when a
solution counter finds exactly one solution, stopping its search after two.
That is a sound uniqueness-preserving construction, but it does not rate human
solving difficulty.

| Selected setting | Requested clues | Engine's clue-count category |
| --- | --- | --- |
| Fácil / Easy | 40–45 | Easy: 40 or more |
| Médio / Medium | 32–38 | Medium: 32–39 |
| Difícil / Hard | 26–31 | Hard: 26–31 |
| Expert | 20–25 | Expert: 20–25 |
| Mestre / Master | 17–19 | Master: fewer than 20 |

Concrete defects and limitations:

1. `generate` reads the range's lower bound but never its upper bound. It checks
   the stopping condition after removing a pair, so it can cross the lower bound.
2. The center cell is excluded from removal. Every generated puzzle has an odd
   number of clues and a filled center. Thus 41 can become 39 for Easy, and 33
   can become 31 for Medium, crossing the engine's own category boundaries.
3. A single pass can exhaust removable pairs above the requested range. Expert
   and Master can then produce the same density as Hard. There is no retry or
   reported failure when a range cannot be reached.
4. `AppState::new_game` stores the requested difficulty instead of
   `Board.difficulty`, hiding the mismatch. Master also disables hints based on
   that requested label, irrespective of the generated puzzle's actual rating.
5. The stored seed belongs only to the removal RNG. Full-grid generation uses a
   separate RNG, so that seed cannot recreate the puzzle. An RNG state of zero
   also remains zero in the current xorshift implementation.
6. Uniqueness, few clues, and symmetry do not establish a human difficulty level.
   Pair-removal irreducibility also does not establish individual-clue minimality.
   The resulting distribution is not established as uniform over Sudoku puzzles.

## Recorded measurements

Twenty puzzles per setting, 100 per audit. Native measurements use Rust 1.98.1,
release mode, Linux x86_64 on an Intel Core i9-12900HK. Timings exclude invariant
checks and compilation. All 100 native puzzles passed solution validity,
clue/solution agreement, unique-solution counting, and rotational clue symmetry.

| Selected | Native clues | Within requested range | Native median / max ms | Engine categories |
| --- | --- | --- | --- | --- |
| Easy | 39 | 0/20 | 0.088 / 0.443 | 20 Medium |
| Medium | 31 | 0/20 | 0.486 / 0.915 | 20 Hard |
| Hard | 25–31 | 18/20 | 1.967 / 12.870 | 18 Hard, 2 Expert |
| Expert | 25–29 | 6/20 | 3.655 / 10.630 | 14 Hard, 6 Expert |
| Master | 25–31 | 0/20 | 3.649 / 17.904 | 18 Hard, 2 Expert |

[Native CSV](difficulty-native.csv) includes the full puzzles and solutions so
those specific boards remain inspectable. Seeds are recorded for provenance,
not as a replay API. Rerunning the audit produces new random puzzles.

The browser audit exercised the release-WASM app from PR #3 in headless desktop
Chromium at 390×844. Its generation code is unchanged in this PR. Handler timing
includes generation and synchronous state/UI work; it excludes automation,
menu-opening, and later persistence/rendering work. Every selection displayed
and saved the requested label and initialized `givens` to the generated board.
No browser exceptions occurred.

| Selected | Browser clues | Within requested range | Handler median / max ms |
| --- | --- | --- | --- |
| Easy | 39 | 0/20 | 0.350 / 0.400 |
| Medium | 31 | 0/20 | 0.650 / 2.300 |
| Hard | 25–29 | 19/20 | 4.200 / 26.700 |
| Expert | 25–31 | 1/20 | 4.400 / 29.000 |
| Master | 25–31 | 0/20 | 6.150 / 28.300 |

[Browser CSV](difficulty-browser.csv) records these selections. A narrow viewport
on a desktop CPU is not Android performance evidence. These small random samples
do not establish population distributions, worst-case latency, or human effort.

## How established engines grade puzzles

There is no universal mapping from techniques to labels. Fix the grader version,
technique ordering, and score configuration before comparing results. Sudoku
Explainer's own documentation explains the limitations of its greedy solve path.

| Engine | Rating approach | Implication for this app |
| --- | --- | --- |
| [Sudoku / Sukaku Explainer](https://github.com/SudokuMonster/SukakuExplainer/wiki/SE121---FAQ) | Rates the hardest technique encountered by its solver, with a detailed technique scale. | A puzzle's solving path matters; the rating is tied to the solver's rules and order. |
| [HoDoKu](https://hodoku.sourceforge.net/en/docs_cre.php) | Each technique has a level and score. Step scores accumulate; the hardest technique sets a minimum level, and total score can raise it. Configuration is adjustable. | Track both the strongest step and the amount of work. |
| [QQWing source](https://qqwing.com/qqwing.cpp.html) | Singles produce Simple/Easy, pairs or box-line eliminations produce Intermediate, and guesses produce Expert. | “Expert” here means this solver needed search; a more capable logical solver might avoid it. |

Human-study research finds that individual step complexity and dependencies
between steps both matter. A clue-count threshold or computer backtracking time
alone is therefore an insufficient product policy. See
[Pelánek's evaluation](https://arxiv.org/abs/1403.7373).

## Established generation approaches

| Approach | How it works | Suitability |
| --- | --- | --- |
| Full grid, then remove clues | Generate a valid solution; remove clues while counting solutions up to two. Optionally preserve symmetry. Rate and accept/reject the result. | Our existing foundation. [QQWing also uses full-grid generation and uniqueness-tested clue removal](https://qqwing.com/qqwing.cpp.html). |
| Exact-cover or constraint solver | Encode Sudoku constraints; use a solver for completion and uniqueness checks. Knuth supplies [Algorithm X / Dancing Links and Sudoku programs](https://www-cs-faculty.stanford.edu/~knuth/programs.html). | A faster verification backend, not a human difficulty rating by itself. |
| Generate and grade offline | Build a large corpus, grade each puzzle, and ship samples from known rating bands. [Sudoku Exchange uses QQWing generation and Sukaku Explainer grading](https://sudokuexchange.com/puzzle-bank/). | A bundled corpus can give stable high difficulty and instant offline selection; corpus provenance and size need review. |
| Strategy-constrained construction | Search for clues whose puzzle can be solved with a defined technique set. [Nishikawa and Toda describe an exact method](https://arxiv.org/abs/2005.14098). | More specialized and potentially expensive; not the first change for a browser PWA. |

Permuting digits, rows within bands, columns within stacks, bands, stacks, or
transposing a known puzzle yields equivalent valid puzzles. This is useful for
variation but cannot produce the full range of puzzle structures from one
fixed template. Minimality means no further clue can be removed while preserving
uniqueness; it does not mean minimum clue count or maximum difficulty.

## Recommended next implementation

Keep the existing uniqueness-preserving foundation and add a deterministic
logical grader with an auditable solve trace. Store the strongest technique,
step counts, and grader version separately from the requested selection. A
stalled limited grader means **unrated**, not automatically Master or guess-only.

A proposed local policy, to calibrate against fixtures and external grading:

| Label | Strongest supported technique on the chosen solve path |
| --- | --- |
| Fácil | Naked/hidden singles |
| Médio | Locked candidates: pointing/claiming |
| Difícil | Naked/hidden pairs or triples |
| Expert | Supported fish/wings, such as X-Wing and XY-Wing |
| Mestre | Supported advanced chains; introduce this band only with verified coverage |

These are proposed app bands, not an SE-compatible scale or an industry standard.
Select new puzzles by their measured band, with bounded retries and a clear
failure/fallback policy. Fix clue-range bounds and center handling, and use one
nonzero seed stream for complete replay. Measure budgets on target Android
hardware; move expensive generation/grading off the UI thread if necessary.
A bundled pregraded corpus is a practical alternative for rare advanced bands.

Validate with fixed technique fixtures, invariant tests, cross-grader comparisons,
and generation success/latency distributions. Keep hint availability as an
explicit assistance policy. Preserve existing saved puzzles and difficulty enum
values; new rating metadata should have compatible defaults. ISSUE-006 remains
open until the new policy is implemented and calibrated.

## Repeat the audits

After installing the pinned toolchain, collect the native audit:

```bash
cargo test --release audit_difficulty_settings -- --ignored --nocapture > /tmp/difficulty-native.log
rg '^AUDIT,' /tmp/difficulty-native.log
```

The normal suite checks generated puzzles at every setting. The manual audit is
ignored by default because it emits measurements rather than a timing gate.

For the browser audit, build with `just build`, install Playwright outside the
repository, and provide `PLAYWRIGHT_MODULE` and `CHROMIUM_PATH` as described in
README's optional browser audit instructions:

```bash
node tests/difficulty-browser.mjs > /tmp/difficulty-browser.log
rg '^AUDIT,' /tmp/difficulty-browser.log
```

Remove the leading `AUDIT,` marker when saving CSV. The raw files in this review
record one run each; their numeric results are not universal performance budgets.
