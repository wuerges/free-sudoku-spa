# Android APKs

The tracked Capacitor 8 project packages the release Rust/WASM bundle as **Sudoku**,
`io.github.wuerges.sudoku`. Minimum Android API is 24; compile/target API is 36.
Debug and disposable-key signed APK builds and lint have passed locally.
GitHub APK distribution is supported by the build workflow; a signed publication
and physical-device acceptance have not yet been verified. Play Store publishing
and save import/export are deferred.

## Local builds

Use Node 24 (`.nvmrc`), JDK 21, Android SDK platform 36, build-tools 36.0.0,
and platform-tools. Set `ANDROID_HOME` and accept SDK licenses. Run `just setup`
for the usual web tools, then:

```sh
just android-doctor
just android-sync       # fresh release bundle, copy, Capacitor sync
just android-debug      # rebuild, assemble debug APK, Android lint
just android-install    # rebuild and adb install -r on the connected device
just android-release    # rebuild, lint and signed release; all credentials required
```

Debug APK: `android/app/build/outputs/apk/debug/app-debug.apk`.
Release APK: `android/app/build/outputs/apk/release/app-release.apk`.
Android Studio can open `android/` after sync. Android tooling is optional for web
commands. `.android-dist/`, `.android-web/` and copied Android assets are ignored;
Android builds leave production `dist/` untouched.

The local origin stays `https://localhost`. There is no remote server URL or live
code-update plugin. Bundled assets work without a prior online installation;
native builds suppress service workers and browser installation prompts. External
links open through Capacitor's Android browser intent handling. Back navigates
through help/settings history, then finishes the activity from the game screen.
The WebView receives system-bar and cutout insets; the manifest requests portrait
and declares `appCategory="game"`. Android 16 exempts games from its large-screen
orientation policy, but Xiaomi's physical tablet behavior remains an acceptance
check. See [Android 16 behavior changes](https://developer.android.com/about/versions/16/behavior-changes-16)
and [Capacitor Android](https://capacitorjs.com/docs/android).

## Saves and upgrades

`src/state.rs` serialization and the `sudoku_state` localStorage key are unchanged.
Game-state compatibility: compatible. Browser and APK saves live in separate
storage and do not transfer automatically. Uninstalling or clearing app data
clears progress; Android backup is disabled to make this boundary explicit.
APK updates must retain package ID, signing certificate and local origin.
A debug APK cannot upgrade a production APK signed with a different key.

Android versionName comes from `package.json`; Cargo/npm versions must agree.
versionCode is `major * 1,000,000 + minor * 1,000 + patch`; minor and patch are
limited to 999, and the code must be between 1 and 2,100,000,000. Compatible minor
and breaking major bumps produce increasing codes. Never publish a lower code
or reuse a code for different application content.

## Private signing setup

Create the production key once on a private machine (passwords are prompted):

```sh
keytool -genkeypair -keystore sudoku-release.jks -alias sudoku \
  -keyalg RSA -keysize 4096 -validity 10000
base64 -w 0 sudoku-release.jks > sudoku-release.base64
```

Keep encrypted offline backups of the keystore, alias and passwords. Losing the
key prevents upgrades for existing installations. Do not commit either file or
passwords. Configure these GitHub Actions secrets:

- `ANDROID_KEYSTORE_BASE64`: base64 contents of the keystore.
- `ANDROID_KEYSTORE_PASSWORD`: keystore password.
- `ANDROID_KEY_ALIAS`: alias, for example `sudoku`.
- `ANDROID_KEY_PASSWORD`: key password.

For local releases, supply the same environment variables privately. The wrapper
decodes into a restricted temporary file and removes it on completion/failure.
Direct Gradle release tasks require `ANDROID_KEYSTORE_PATH` plus the three other
credentials. Missing signing credentials fail clearly; release tasks never use
a debug signing fallback. Record the production certificate fingerprint privately
and compare it with `apksigner verify --verbose --print-certs` on every release.

## CI and publication

`Android checks` builds debug APKs, runs lint, uploads debug artifacts and runs
offline route/persistence smoke tests on API-36 phone/tablet emulator profiles.
PR jobs receive no signing secrets. These checks should be required alongside CI
in repository branch protection before merging.

The existing CI web-release workflow is unchanged. After successful main CI and Android checks,
`Android release` resolves the already published version tag, checks it against
the successful commit, builds a signed APK, verifies its signature and attaches
`sudoku-<version>.apk` and `sudoku-<version>.apk.sha256` to that release. Failures
are separate from web publication. Retry with Actions → Android release → Run
workflow, specifying the existing version on the main workflow branch. A retry
checks out the immutable release commit. Existing assets are never overwritten;
a different rebuilt APK requires investigation rather than replacing a published
binary. Publication is not verified until the job and downloaded-APK checks pass.

## Release acceptance checklist

Record actual results in the change description; keep pending items in ISSUES.md.

1. Fresh install, airplane mode before first launch: game, help/config routes,
   new game at each of five difficulties, notes, Drop, domino, sound and touch.
2. Phone and API-36 tablet: system bars, Back, links, background/resume,
   force-stop/reopen and saved settings/history. Rotate and verify portrait.
3. Xiaomi 11-inch physical tablet: portrait during rotation, board/control layout,
   sound and generation latency. Manifest/emulator checks do not replace this.
4. Using a private disposable test key, build/install version N, make board/note
   edits and history/settings changes, then install version N+1 with `adb install
   -r`. Verify board, notes, undo/redo history and settings survive. Restore source
   versions afterward; never publish test builds or test keys.
5. Download the published APK/checksum, run `sha256sum -c` and `apksigner verify
   --verbose --print-certs`, compare with the expected production fingerprint,
   and install/upgrade it on a device.

With optional Playwright installed, `node tests/android-browser.mjs` checks staged
native assets through an intercepted `https://localhost` origin with networking
disabled, at phone/tablet sizes. Build and stage with `just android-sync` first;
the asset build/stage helpers can also be used without an SDK. This checks the
web bundle and does not emulate Android lifecycle, intents or system bars.

Emulator smoke tests cover bundled startup/routes and a reload retaining the board;
they do not establish all gameplay flows or a real signed in-place upgrade.
