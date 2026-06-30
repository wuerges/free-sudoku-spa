#!/bin/sh
set -e
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/download/v4.1.9/tailwindcss-linux-x64
chmod +x tailwindcss-linux-x64
mv tailwindcss-linux-x64 /usr/local/bin/tailwindcss
tailwindcss -i style/input.css -o style/output.css --minify
trunk build --release
