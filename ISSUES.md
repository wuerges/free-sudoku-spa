# Open issues

Track only unresolved issues here. Remove an issue when its acceptance criteria
have been verified; keep resolution history in the PR or change description.
Use stable IDs and do not renumber existing issues.

## ISSUE-002: Correct hosting cache policy

- Status: Configuration now defaults to revalidation and reserves immutable caching for hashed Trunk JS/WASM/CSS. Deployed behavior remains unverified.
- Impact: Actual Vercel response headers and returning-player updates still need a preview deployment check.
- Acceptance: Use long-lived immutable caching only for hashed assets. Verify deployed response headers for `/`, `/help`, `/config`, `/index.html`, `/sw.js`, and `/manifest.json`, and verify updates reach returning players.

## ISSUE-003: Make deployment builds reproducible

- Status: Setup now installs pinned project-local Rust/Trunk, verifies Trunk checksums, and uses the npm lockfile for CSS. PR #2’s Vercel build check passed; clean-install build-log verification remains pending.
- Impact: The clean setup/build must still be verified on Vercel's build image.
- Acceptance: Use pinned build tooling and consistent CSS dependencies, avoid privileged install paths, and verify a clean Vercel preview build.

## ISSUE-004: Put the build in Vercel's build phase

- Status: Vercel now has separate setup and build commands. PR #2’s Vercel build check passed; preview runtime acceptance remains pending.
- Impact: Successful output from Vercel's actual build phase still needs verification.
- Acceptance: Separate dependency setup from compilation and verify that Vercel's build phase produces a complete `dist/` bundle.

## ISSUE-005: Establish PR deployment checks

- Evidence: Vercel GitHub integration is confirmed by PR #1. CI now supplies Rust formatting/lint/test, color/version, and production-build checks. GitHub required checks and latest preview acceptance remain unverified.
- Impact: A deployable bundle can pass without correctness checks, and preview automation is unconfirmed.
- Acceptance: Verify Vercel repository connection and production branch, run appropriate Rust checks for PRs, require applicable checks before merging, and verify a preview for the latest PR commit. Smoke-test gameplay, mobile layout, asset loading, and direct route navigation.

## ISSUE-007: Verify persistence failure handling

- Evidence: Browser checks verified reload/resume, legacy saves without givens/highlight/rating metadata, undo history, and highlight preference persistence in both themes. `src/state.rs` still handles storage errors largely silently; malformed data and unavailable storage need validation.
- Impact: Players may lose progress without clear feedback when saved data is malformed or storage fails.
- Acceptance: Verify reload/resume, malformed data, unavailable storage, and state-format compatibility; provide understandable feedback for failures while keeping gameplay usable.

## ISSUE-008: Verify mobile accessibility

- Evidence: Automated contrast checks and light/dark browser flows passed at 390×844 and 1280×900, including routes, board highlights, shading sliders, and pattern controls. Keyboard navigation, screen-reader labels, touch targets, and physical phone/tablet usability still need an accessibility audit.
- Impact: Usability and accessibility on target devices remain unverified.
- Acceptance: Audit phone/tablet layouts, keyboard navigation, cell/control labels, contrast, and touch targets; resolve findings and record verification in the PR.

## ISSUE-009: Resolve build dependency audit findings

- Evidence: `npm audit` during clean setup reports five high-severity findings in the locked Tailwind build dependency tree, including `braces` (GHSA-vfj7-8cjw-p6xm) and `source-map-js` (GHSA-68fv-2mgg-jv7q).
- Impact: Build-time pattern/source-map processing uses dependencies with reported denial-of-service vulnerabilities; npm's suggested Tailwind fix changes the selected version.
- Acceptance: Review applicability, update compatible build dependencies and lockfile, and verify audit results, CSS output, and the production build without a forced unreviewed upgrade.
