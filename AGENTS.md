# Repository instructions

## Workflow

- Use `jj` (Jujutsu) for all version control operations, not Git commands.
- When preparing a PR, create a dedicated change with `jj new` and describe it with `jj describe`; keep each PR's work isolated from unrelated changes.
- Inspect `jj status` and `jj diff` before editing; preserve unrelated work. If repository metadata is read-only, use `jj --ignore-working-copy` for inspection and report that it omits unsnapshotted changes.
- Keep changes focused on the requested task. Do not commit, push, or deploy unless requested.
- Read `README.md` for setup and `docs/architecture.md` for the implementation map.
- `VISION.md` defines product direction, current implementation, and known gaps. Verify behavior against the code before changing its status.
- Track unresolved issues in `ISSUES.md`. Remove issues once their acceptance criteria are verified; keep only open issues in that file and record resolution history in the PR or change description.

## Project conventions

- This is a client-side Rust/Leptos 0.7 Sudoku PWA compiled to WebAssembly with Trunk. Node is used for Tailwind CSS tooling.
- Keep Sudoku algorithms in `src/sudoku_engine.rs`, reactive state and persistence in `src/state.rs`, and UI in `src/components/`.
- Keep identifiers and technical documentation in English and player-facing strings in Brazilian Portuguese.
- Preserve mobile/touch usability, offline behavior, and existing saved games. Avoid HTML input fields in the game board and number pad, which uses custom buttons.
- Treat persisted state changes as compatibility-sensitive: inspect serialization and loading before changing fields.
- Edit `style/input.css`, not generated `style/output.css`. Do not hand-edit `dist/`, `target/`, or `node_modules/`.

## Validation

- Rust changes: run `cargo fmt --check`, `just test`, and `just check`. Add meaningful regression tests for changes to game rules or state transitions.
- UI, CSS, or asset changes: run `just build` and check the affected flow in a browser when available, including a narrow mobile viewport.
- PWA changes: verify the manifest, asset paths, cache/update behavior, and offline reload.
- Documentation-only changes: verify referenced paths and commands; no application tests are required.
- Report checks actually run and any environment blockers. Do not claim unrun checks passed.
