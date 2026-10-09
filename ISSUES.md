# Open issues

Track only unresolved issues here. Remove an issue when its acceptance criteria
have been verified; keep resolution history in the PR or change description.
Use stable IDs and do not renumber existing issues.

## ISSUE-001: Complete offline caching

- Evidence: `sw.js` precaches only the manifest and does not cache fetched app-shell assets.
- Impact: Offline reload and navigation are not reliable.
- Acceptance: After an online load, verify offline reload, navigation, new-game generation, and saved-game resume with all required HTML, WASM, JS, CSS, and icons available.

## ISSUE-002: Correct hosting cache policy

- Evidence: `vercel.json` applies one-year immutable caching broadly, with an exception only for `/index.html`.
- Impact: Unhashed files and navigation routes may receive unsuitable cache headers.
- Acceptance: Use long-lived immutable caching only for hashed assets. Verify deployed response headers for `/`, `/help`, `/config`, `/index.html`, `/sw.js`, and `/manifest.json`, and verify updates reach returning players.

## ISSUE-003: Make deployment builds reproducible

- Evidence: `build.sh` assumes Rust is available, installs an unpinned Trunk version, downloads Tailwind 4.1.9 without a checksum, and writes it to `/usr/local/bin`. Local CSS uses the npm lockfile instead.
- Impact: Deployment depends on environment permissions and can differ from local builds.
- Acceptance: Use pinned build tooling and consistent CSS dependencies, avoid privileged install paths, and verify a clean Vercel preview build.

## ISSUE-004: Put the build in Vercel's build phase

- Evidence: `vercel.json` runs `sh build.sh` as its install command; its build command is only `echo built`.
- Impact: Dependency installation and compilation are conflated, making build failures harder to inspect.
- Acceptance: Separate dependency setup from compilation and verify that Vercel's build phase produces a complete `dist/` bundle.

## ISSUE-005: Establish PR deployment checks

- Evidence: No CI workflow is checked in, and `build.sh` does not run formatting, lint, or test checks. Vercel repository connection and GitHub required checks have not been verified.
- Impact: A deployable bundle can pass without correctness checks, and preview automation is unconfirmed.
- Acceptance: Verify Vercel repository connection and production branch, run appropriate Rust checks for PRs, require applicable checks before merging, and verify a preview for the latest PR commit. Smoke-test gameplay, mobile layout, asset loading, and direct route navigation.

## ISSUE-006: Validate difficulty classification

- Evidence: `src/sudoku_engine.rs` grades by clue count and may stop removal before reaching the requested range. It does not grade human solving techniques.
- Impact: Difficulty labels do not guarantee consistent solving effort or clue ranges.
- Acceptance: Test generated puzzles across all levels for uniqueness, record achieved clue counts and generation times, and define and validate a consistent difficulty policy.

## ISSUE-007: Verify persistence failure handling

- Evidence: `src/state.rs` uses localStorage with errors largely handled silently; compatibility and storage failure scenarios need validation.
- Impact: Players may lose progress without clear feedback when saved data is malformed or storage fails.
- Acceptance: Verify reload/resume, malformed data, unavailable storage, and state-format compatibility; provide understandable feedback for failures while keeping gameplay usable.

## ISSUE-008: Verify mobile accessibility

- Evidence: Keyboard, screen-reader, touch target, and contrast acceptance checks in `VISION.md` have not been established by a browser audit.
- Impact: Usability and accessibility on target devices remain unverified.
- Acceptance: Audit phone/tablet layouts, keyboard navigation, cell/control labels, contrast, and touch targets; resolve findings and record verification in the PR.
