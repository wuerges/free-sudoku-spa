# Offline puzzle bank

The 5,000 puzzles in `src/puzzle_bank.csv` come from
[Sudoku Exchange](https://github.com/grantm/sudoku-exchange-puzzle-bank), pinned to
revision `d8c8ebaee0c08c412cfba96af1923dfa61c83317`.
The dataset is dedicated to the public domain; its original license is retained
in [puzzle-bank-LICENSE.txt](puzzle-bank-LICENSE.txt).
Upstream generated candidates using QQWing and rated them with Sukaku Explainer.
This is a generated, graded collection, not a claim of individual human curation.

Reproduce from an extracted archive of that revision:

```sh
sh scripts/with-build-env.sh cargo run --release --features puzzle-bank-tools --bin import_puzzle_bank -- /path/to/extracted/source /tmp/puzzle_bank.csv
cmp /tmp/puzzle_bank.csv src/puzzle_bank.csv
```

The Rust importer reads easy, medium, hard and diabolical files in that order
and preserves their record order. It accepts the first 1,000 candidates in each
of the app's five local technique bands, checks uniqueness and stores the logical
solver's completed grid. Unsupported candidates are skipped. A graph fingerprint
filters equivalents under all standard Sudoku symmetries, including legal
row/column permutations and digit relabeling. Fingerprint collisions can reject
additional candidates; they cannot cause equivalent puzzles to be accepted.
The output is deterministic for this importer, grader and source revision.

All 5,000 puzzles are checked again by the engine test suite, including logical
trace soundness. Grading describes this solver's chosen path. Enjoyment and
physical-device performance still need player feedback; the historical external
calibration CSV describes the previous 30-puzzle bank, not this collection.

Runtime selection walks a seeded shuffle per level and saves its position
separately from games. Each base is used once per cycle, including across
reloads; cycle boundaries avoid immediate repetition. Clearing browser storage
resets the cycle. Unavailable storage and simultaneous selections in multiple
tabs cannot guarantee repeat avoidance. New games reserve a selection before the
archive transition completes, so canceled/failed transitions can skip entries.
Existing saves and shared links retain their explicit boards and solutions.
