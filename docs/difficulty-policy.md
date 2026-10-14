# Difficulty policy v1

The app uses its own deterministic technique order and bands. The strongest
technique used on the chosen logical solve path determines the level; this
is not a claim about every possible solve path or equivalence to SE scores.

| Level | Techniques |
| --- | --- |
| Fácil | Naked and hidden singles |
| Médio | Locked candidates: pointing/claiming |
| Difícil | Naked/hidden pairs and triples |
| Expert | X-Wing, XY-Wing, simple coloring, three-link alternating chains |
| Mestre | Five/seven-link alternating single-digit chains |

`grade` initializes candidates from the original clues, retains eliminations,
and records placements/eliminations and technique counts. It never reads the
solution or guesses. Stalled, invalid, or exhausted puzzles remain unrated.
Fixed trace tests check that every placement matches the known solution and
no elimination removes a solution candidate. The same checks cover the
historical 100-puzzle corpus and every bundled fixture.

## Generation and responsiveness

- Full-grid completion and uniqueness use MRV search, not a difficulty score.
- Each search has a 20,000-node budget. An inconclusive uniqueness check rejects
  the removal; it is never accepted as evidence of uniqueness.
- Clue removal includes the center, honors both bounds, and returns failure if
  it cannot reach the requested density range. Density is secondary to rating.
- All five levels select from the bundled bank, 1,000 puzzles per level.
- Each level walks a seeded shuffled permutation, persisted separately in
  `sudoku_bank_v2`. A cycle contains no repeated base puzzle; the next cycle's
  first puzzle cannot equal the previous cycle's last. Storage failure falls
  back to playable games without a durable repeat guarantee. Tabs share the
  cursor but concurrent selections are not transactional.
- Digit/rotation/reflection variations get two attempts and are regraded.
  The verified original is the final fallback.
- A logical trace is bounded to 810 progress steps. Chain searches are capped
  at seven links and 10,000 visits per digit; exhausting these limits is unrated.

The bank has 5,000 puzzles imported from the public-domain Sudoku Exchange
collection. See [provenance and reproduction](puzzle-bank.md). The importer
checks uniqueness and local rating and filters equivalents using a graph
fingerprint invariant under digit relabeling, transpose, legal row/column and
band/stack permutations. Hash collisions may reject extra candidates.
Normal tests verify all shipped puzzles' solutions, uniqueness and logical traces.
No network or external engine is needed in the app. Clue patterns need not be symmetric.

Seed zero is normalized to a nonzero constant. Recorded seeds reproduce new
selections with this implementation/corpus; cross-version replay is not promised.
Existing saves store full boards/solutions and never regenerate from seeds.

## Independent calibration

The former 30-puzzle self-generated bank was compared with the official
[Sukaku Explainer 1.18.1 release](https://github.com/SudokuMonster/SukakuExplainer/releases/tag/v1.18.1).
[Calibration CSV](difficulty-calibration.csv) records those 30 historical puzzles,
SE ratings, and the external solver's strongest technique. Historical Hard bank puzzles
were filtered to SE ≥3.0 and Master to SE ≥5.0; the app still requires its own
matching logical rating. Java/the external grader are development tools only.

Simple coloring overlapped Expert on the external scale, so v1 treats coloring
as Expert and reserves Master for longer alternating chains. SE and local
ratings can differ because rule sets and solve ordering differ; the lower
bands' SE ranges overlap. This calibration is a useful cross-check, not a
universal human-difficulty measurement. Technique definitions were checked
against HoDoKu's [coloring](https://hodoku.sourceforge.net/en/tech_col.php),
[fish](https://hodoku.sourceforge.net/en/tech_fishb.php), and
[wings](https://hodoku.sourceforge.net/en/tech_wings.php) documentation.

Reproduce the external check with the pinned JAR outside the repository:

```bash
java -cp /tmp/SukakuExplainer-1.18.1.jar diuf.sudoku.test.serate \
  --input=/tmp/puzzles.txt --output=/tmp/ratings.txt --format='%g,%r,%R' --threads=2
```

`puzzles.txt` contains the bank's puzzle column, one 81-digit grid per line.
The ignored `collect_grading_fixtures` test searches original puzzles and checks
all deductions before printing candidates; external calibration happens before
adding them to the production bank.

## Persistence and assistance

New saves add optional `requested_difficulty` and `rating` metadata. The rating
includes grader version, measured level, strongest technique, and step counts.
The header shows that technique. Old saves load these fields as absent, retain
all progress/settings/history and their old difficulty, and show “jogo anterior”.
Moves, undo, and hints keep the original puzzle's rating. Hints follow the
player's setting at every level, including Master.

## Recorded verification

[Native audit](difficulty-v1-native.csv) and
[offline WASM audit](difficulty-v1-browser.csv) each contain 20 selections per
level. Every selection matched the requested measured rating. Native fixtures
passed rule, solution, uniqueness, and symmetry checks. Browser selections
verified rating metadata, clue provenance, level text, and Master hint access
without network access. Timing summaries are recorded in the PR.

Measurements use Rust 1.98.1 release mode on Linux x86_64 / Intel i9-12900HK and
headless desktop Chromium at 390×844. Browser handler timings include synchronous
state/UI work and exclude automation and later persistence/rendering. A narrow
viewport is not Android performance evidence. Target-device latency and player
feedback remain validation work; no universal timing budget is claimed.
