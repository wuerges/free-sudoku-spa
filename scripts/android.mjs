import { readFileSync, writeFileSync, cpSync, rmSync, existsSync, mkdtempSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

export function versionCode(version) {
  if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version)) throw Error('Invalid application version');
  const [major, minor, patch] = version.split('.').map(Number);
  const code = major * 1000000 + minor * 1000 + patch;
  if (minor > 999 || patch > 999 || code < 1 || code > 2100000000) throw Error('Android version components out of bounds');
  return code;
}
export function requireSigning(env) {
  for (const key of ['ANDROID_KEYSTORE_BASE64', 'ANDROID_KEYSTORE_PASSWORD', 'ANDROID_KEY_ALIAS', 'ANDROID_KEY_PASSWORD']) {
    if (!env[key]) throw Error(`Missing release signing credential: ${key}`);
  }
}
export function stage(source, destination) {
  if (resolve(source) === resolve(destination)) throw Error('Android staging must be isolated');
  rmSync(destination, { recursive: true, force: true });
  cpSync(source, destination, { recursive: true });
  const html = readFileSync(`${destination}/index.html`, 'utf8');
  writeFileSync(`${destination}/index.html`, html.replace('<head>', '<head>\n  <script>window.__SUDOKU_NATIVE__ = true;</script>'));
  rmSync(`${destination}/sw.js`, { force: true });
}
function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: 'inherit', ...options });
  if (result.error) throw result.error;
  if (result.status !== 0) throw Error(`${command} failed (${result.status})`);
}
function doctor() {
  if (Number(process.versions.node.split('.')[0]) !== 24) throw Error('Node 24 is required');
  const java = spawnSync('java', ['-version'], { encoding: 'utf8' });
  if (java.status !== 0 || !/version "21[."]/.test(java.stderr + java.stdout)) throw Error('JDK 21 is required');
  const sdk = process.env.ANDROID_HOME || process.env.ANDROID_SDK_ROOT;
  if (!sdk || !existsSync(`${sdk}/platforms/android-36/android.jar`)) throw Error('Set ANDROID_HOME to an SDK with platform android-36 and build-tools 36.0.0');
  run(`${sdk}/platform-tools/adb`, ['version']);
}
function sync() {
  // Build to an isolated directory: ordinary production dist remains untouched.
  run('sh', ['scripts/with-build-env.sh', 'npm', 'run', 'css']);
  run('sh', ['scripts/with-build-env.sh', 'trunk', 'build', '--release', '--locked', '--dist', '.android-dist']);
  stage('.android-dist', '.android-web');
  run('node_modules/.bin/cap', ['sync', 'android']);
}
async function main() {
  const command = process.argv[2];
  if (command === 'doctor') return doctor();
  if (command === 'release') requireSigning(process.env);
  if (!['sync', 'debug', 'install', 'release'].includes(command)) throw Error('Expected doctor, sync, debug, install or release');
  doctor();
  sync();
  if (command === 'sync') return;
  let privateDir;
  try {
    const env = { ...process.env };
    if (command === 'release') {
      privateDir = mkdtempSync(`${tmpdir()}/sudoku-signing-`);
      env.ANDROID_KEYSTORE_PATH = `${privateDir}/release.jks`;
      writeFileSync(env.ANDROID_KEYSTORE_PATH, Buffer.from(env.ANDROID_KEYSTORE_BASE64, 'base64'), { mode: 0o600 });
    }
    run('./gradlew', [command === 'release' ? 'assembleRelease' : 'assembleDebug', 'lint'], { cwd: 'android', env });
    if (command === 'install') run(`${env.ANDROID_HOME || env.ANDROID_SDK_ROOT}/platform-tools/adb`, ['install', '-r', 'android/app/build/outputs/apk/debug/app-debug.apk']);
  } finally {
    if (privateDir) rmSync(privateDir, { recursive: true, force: true });
  }
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch(error => { console.error(error.message); process.exitCode = 1; });
}
