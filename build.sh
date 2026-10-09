#!/bin/sh
set -eu
SUDOKU_PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$SUDOKU_PROJECT_ROOT"
. "$SUDOKU_PROJECT_ROOT/scripts/build-env.sh"

if [ ! -x "$CARGO_HOME/bin/rustup" ] || [ ! -x "$SUDOKU_TOOLS/bin/trunk" ]; then
    echo 'Missing build tools; run sh scripts/setup-build.sh first' >&2
    exit 1
fi
if [ "$("$SUDOKU_TOOLS/bin/trunk" --version)" != "trunk $SUDOKU_TRUNK_VERSION" ]; then
    echo 'Unexpected Trunk version; rerun sh scripts/setup-build.sh' >&2
    exit 1
fi
npm run test:offline
npm run css
"$SUDOKU_TOOLS/bin/trunk" build --release --locked
