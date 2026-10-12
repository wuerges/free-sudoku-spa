# Shipped asset review

The loading screen previously used a platform emoji, which depended on emoji-font
support. The maskable PWA PNG was a flat background with no artwork. Ordinary
PWA and Android icons had overlapping digits. A bundled Sudoku SVG now supplies
the loading image and favicon; corresponding PNGs provide installation icons.
The normal icon separates all nine digits. The maskable icon keeps its artwork
inside the central 80% diameter safe circle, and Android adaptive foregrounds use
the same safe artwork with a matching navy background. Legacy Android launchers
retain the ordinary icon. Apple touch icons use the 192px PNG.

The 1200×630 social preview remains valid, with readable board artwork. It is a
generic illustration with Portuguese promotional text, not a per-game or
per-locale preview. Shared line icons retain their existing local SVG paths.
Generated CSS, WASM and JavaScript remain build products; the service worker
includes all public assets and changes its cache version when artwork changes.
No saved game fields changed.

Regenerate the PNGs from `public/icons/sudoku.svg` with
`node scripts/render-icons.mjs` using the optional Playwright environment below.
The rasterizer uses Chromium SVG support to keep web and native artwork aligned.

After `just build`, run `node tests/assets-browser.mjs` with the optional
`PLAYWRIGHT_MODULE`/`CHROMIUM_PATH` browser environment described in README.
The audit decodes all shipped artwork, checks PNG dimensions and nonblank content,
verifies the maskable safe circle and matching Android PNGs, renders every line
icon, holds WASM startup to inspect loading on 320px/1280px viewports in both
themes, checks reduced motion, and fetches every public asset and route offline.
Screenshots go to `/tmp/sudoku-assets` (`ASSET_SCREENSHOTS` overrides this).
JavaScript is used for this audit because browser automation and canvas image
inspection integrate directly with the project's existing Playwright tests.

Verified locally: production build, offline worker tests, Android staging tests,
release policy tests, and the browser asset audit (7 public files, 5 image
variants, 21 control icons, both loading themes at 320px/1280px and all offline
routes) pass. Loading and control-gallery screenshots were visually inspected.

Physical Android launcher/splash masking still requires device acceptance under
ISSUE-010; the browser safe-circle audit does not establish that acceptance.
