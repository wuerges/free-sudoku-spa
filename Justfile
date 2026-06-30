# Sudoku PWA — ponytail: just serve / just build / just test. That's it.

default:
    @just --list

# Build CSS (Tailwind v4 standalone CLI)
css:
    tailwindcss -i style/input.css -o style/output.css --minify

# Dev CSS watcher
css-watch:
    tailwindcss -i style/input.css -o style/output.css --watch

# Dev server
serve:
    @echo "→ http://localhost:8080"
    trunk serve

# Release build → dist/
build: css
    trunk build --release

# Check compilation
check:
    cargo check --target wasm32-unknown-unknown
    cargo clippy -- -D warnings

# Run tests
test:
    cargo test

# Full CI pipeline
ci: check test build

# Clean build artifacts
clean:
    rm -rf dist/ target/
