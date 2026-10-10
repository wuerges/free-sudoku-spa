# Local development and validation commands (Node 24; run setup once)

default:
    @just --list

# Install the pinned Rust/Trunk tools and locked npm dependencies
setup:
    sh scripts/setup-build.sh

# Build CSS using the locked npm tooling
css:
    npm run css

# Standalone CSS watcher; dev already rebuilds CSS automatically
css-watch:
    npm run css:watch

# Debug server with Rust/CSS reload at http://localhost:8080; accepts Trunk options
dev *args:
    sh scripts/with-build-env.sh trunk serve --config Trunk.dev.toml --locked {{args}}

# Backward-compatible name for the local development server
alias serve := dev

# Release build with offline caching → dist/
build:
    sh build.sh

# Check Rust formatting
fmt:
    sh scripts/with-build-env.sh cargo fmt --check

# Check WASM compilation and lint
check:
    sh scripts/with-build-env.sh cargo check --locked --target wasm32-unknown-unknown
    sh scripts/with-build-env.sh cargo clippy --locked -- -D warnings

# Run engine/state tests
test:
    sh scripts/with-build-env.sh cargo test --locked

# Validate build/dev scripts, offline caching, contrast and release policy
test-tooling:
    npm run test:build
    npm run test:dev
    npm run test:offline
    npm run test:theme
    npm run test:release

# Full local validation pipeline
ci: fmt test check test-tooling build

# Remove build outputs; keep installed tools and dependencies
clean:
    rm -rf dist/ .dev-dist/ target/
