#!/bin/sh
set -eu
SUDOKU_PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$SUDOKU_PROJECT_ROOT"
. "$SUDOKU_PROJECT_ROOT/scripts/build-env.sh"

sudoku_require_build_tools

npm run test:offline
npm run css
"$SUDOKU_TOOLS/bin/trunk" build --release --locked
