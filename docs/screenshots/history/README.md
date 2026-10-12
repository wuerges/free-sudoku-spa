# Local game history review

Production Chromium at phone (390×844) and tablet (800×1280) sizes, in both
system themes. Miniatures remain inside their cards and do not cover controls.
`tests/history-browser.mjs` generates the captures and verifies retention,
pinning, identity/undo/settings resume, separate restart at capacity, confirmed
shared-link archiving, archive/primary-save failures, cleanup retry and offline
history reload. These are emulated browser checks; physical/native upgrade and
full screen-reader acceptance remain open in ISSUE-008/010.

| View | Phone | Tablet |
| --- | --- | --- |
| History, light | [Capture](history-light-390.png) | [Capture](history-light-800.png) |
| Restart, light | [Capture](restart-light-390.png) | [Capture](restart-light-800.png) |
| History, dark | [Capture](history-dark-390.png) | [Capture](history-dark-800.png) |
| Restart, dark | [Capture](restart-dark-390.png) | [Capture](restart-dark-800.png) |
