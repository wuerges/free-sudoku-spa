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

if ! node -e 'if (Number(process.versions.node.split(".")[0]) !== 24) process.exit(1)' ; then
    echo 'Node.js 24 is required; run nvm install && nvm use (or select Node 24 with your version manager)' >&2
    exit 1
fi

sudoku_require_build_tools() {
    if [ ! -x "$CARGO_HOME/bin/rustup" ] || [ ! -x "$SUDOKU_TOOLS/bin/trunk" ]; then
        echo 'Missing build tools; run just setup (or sh scripts/setup-build.sh) first' >&2
        exit 1
    fi
    if [ "$("$SUDOKU_TOOLS/bin/trunk" --version)" != "trunk $SUDOKU_TRUNK_VERSION" ]; then
        echo 'Unexpected Trunk version; rerun just setup' >&2
        exit 1
    fi
}
