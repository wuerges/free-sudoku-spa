# Sharing review

Production Chromium captures at 390×844 (phone) and 800×1280 (tablet), in light
and dark themes. `tests/sharing-browser.mjs` generates these images and verifies
current/initial snapshots, clipboard fallback, system-share cancellation, import
confirmation/cancellation, settings/history, fragment navigation, invalid links,
completed imports, restarting received progress from original clues and offline
reload/resume. These are emulated viewports, not
physical Android acceptance or a social-network crawler acceptance test.

| View | Phone | Tablet |
| --- | --- | --- |
| Share, light | [Capture](share-light-390.png) | [Capture](share-light-800.png) |
| Import, light | [Capture](import-light-390.png) | [Capture](import-light-800.png) |
| Share, dark | [Capture](share-dark-390.png) | [Capture](share-dark-800.png) |
| Import, dark | [Capture](import-dark-390.png) | [Capture](import-dark-800.png) |
| Initial import, light | [Capture](import-initial-light-390.png) | [Capture](import-initial-light-800.png) |
| Initial import, dark | [Capture](import-initial-dark-390.png) | [Capture](import-initial-dark-800.png) |
