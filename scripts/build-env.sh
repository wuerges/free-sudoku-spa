# Sourced by setup/build scripts after SUDOKU_PROJECT_ROOT is set.
SUDOKU_TOOLS="$SUDOKU_PROJECT_ROOT/.build-tools"
CARGO_HOME="$SUDOKU_TOOLS/cargo"
RUSTUP_HOME="$SUDOKU_TOOLS/rustup"
PATH="$SUDOKU_TOOLS/bin:$CARGO_HOME/bin:$PATH"
SUDOKU_RUST_VERSION=$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' "$SUDOKU_PROJECT_ROOT/rust-toolchain.toml")
SUDOKU_TRUNK_VERSION=0.21.14
if [ -z "$SUDOKU_RUST_VERSION" ]; then
    echo 'Missing pinned Rust toolchain' >&2
    exit 1
fi
# Trunk accepts true/false, whereas terminal environments often set NO_COLOR=1.
if [ -n "${NO_COLOR:-}" ]; then NO_COLOR=true; export NO_COLOR; fi
export CARGO_HOME RUSTUP_HOME PATH
