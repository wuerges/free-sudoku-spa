# Local development and validation commands

default:
    @just --list

# Build CSS using the locked npm tooling
css:
    npm run css

# Dev CSS watcher
css-watch:
    npm run css:watch

# Dev server
serve: css
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
