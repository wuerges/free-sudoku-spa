#!/bin/sh
set -eu
SUDOKU_PROJECT_ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$SUDOKU_PROJECT_ROOT"
. "$SUDOKU_PROJECT_ROOT/scripts/build-env.sh"

case "$(uname -s)/$(uname -m)" in
    Linux/x86_64)
        SUDOKU_HOST=x86_64-unknown-linux-gnu
        SUDOKU_TRUNK_ARCH=x86_64-unknown-linux-musl
        SUDOKU_TRUNK_SHA=a67f4054b249fe9acc5fabc25de1aebf19783aca3ad6ff64bf34d7da44d0ea20
        ;;
    Linux/aarch64)
        SUDOKU_HOST=aarch64-unknown-linux-gnu
        SUDOKU_TRUNK_ARCH=aarch64-unknown-linux-musl
        SUDOKU_TRUNK_SHA=e8e2a2bb423ce6702ab9f4f02f8c9ae99d790f0301f7634e986b2dd8706019cc
        ;;
    *) echo 'Build setup supports Linux x86_64 and aarch64 only' >&2; exit 1 ;;
esac

npm ci
mkdir -p "$SUDOKU_TOOLS/bin"
SUDOKU_DOWNLOAD_DIR=$(mktemp -d "$SUDOKU_TOOLS/download.XXXXXX")
trap 'rm -rf "$SUDOKU_DOWNLOAD_DIR"' EXIT HUP INT TERM

if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
    SUDOKU_RUSTUP_URL="https://static.rust-lang.org/rustup/dist/$SUDOKU_HOST/rustup-init"
    curl --fail --silent --show-error --location --retry 3 "$SUDOKU_RUSTUP_URL" -o "$SUDOKU_DOWNLOAD_DIR/rustup-init"
    curl --fail --silent --show-error --location --retry 3 "$SUDOKU_RUSTUP_URL.sha256" -o "$SUDOKU_DOWNLOAD_DIR/rustup-init.sha256"
    (cd "$SUDOKU_DOWNLOAD_DIR" && sha256sum --check rustup-init.sha256)
    chmod +x "$SUDOKU_DOWNLOAD_DIR/rustup-init"
    "$SUDOKU_DOWNLOAD_DIR/rustup-init" -y --no-modify-path --profile minimal --default-toolchain none
fi
"$CARGO_HOME/bin/rustup" toolchain install "$SUDOKU_RUST_VERSION" --profile minimal --no-self-update
"$CARGO_HOME/bin/rustup" target add --toolchain "$SUDOKU_RUST_VERSION" wasm32-unknown-unknown
"$CARGO_HOME/bin/rustup" component add --toolchain "$SUDOKU_RUST_VERSION" rustfmt clippy

if [ ! -x "$SUDOKU_TOOLS/bin/trunk" ] || [ "$("$SUDOKU_TOOLS/bin/trunk" --version)" != "trunk $SUDOKU_TRUNK_VERSION" ]; then
    SUDOKU_TRUNK_URL="https://github.com/trunk-rs/trunk/releases/download/v$SUDOKU_TRUNK_VERSION/trunk-$SUDOKU_TRUNK_ARCH.tar.gz"
    curl --fail --silent --show-error --location --retry 3 "$SUDOKU_TRUNK_URL" -o "$SUDOKU_DOWNLOAD_DIR/trunk.tar.gz"
    echo "$SUDOKU_TRUNK_SHA  $SUDOKU_DOWNLOAD_DIR/trunk.tar.gz" | sha256sum --check -
    tar -xzf "$SUDOKU_DOWNLOAD_DIR/trunk.tar.gz" -C "$SUDOKU_DOWNLOAD_DIR"
    install -m 755 "$SUDOKU_DOWNLOAD_DIR/trunk" "$SUDOKU_TOOLS/bin/trunk"
fi
"$CARGO_HOME/bin/rustc" --version
"$SUDOKU_TOOLS/bin/trunk" --version
