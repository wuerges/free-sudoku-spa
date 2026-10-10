import { execFileSync } from 'node:child_process';
import { readFileSync, appendFileSync, copyFileSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
const gh = (...args) => execFileSync('gh', args, { encoding: 'utf8' }).trim();
const version = process.env.REQUESTED_VERSION || JSON.parse(readFileSync('package.json')).version;
if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version)) throw Error('Invalid release version');
const tag = `v${version}`;
if (process.argv[2] === 'resolve') {
  // CI has finished its web release job before workflow_run is delivered.
  const release = JSON.parse(gh('api', `repos/${process.env.GITHUB_REPOSITORY}/releases/tags/${tag}`));
  if (release.draft || release.prerelease) throw Error('Expected a published stable release');
  const ref = JSON.parse(gh('api', `repos/${process.env.GITHUB_REPOSITORY}/git/ref/tags/${tag}`));
  let object = ref.object;
  while (object.type === 'tag') object = JSON.parse(gh('api', `repos/${process.env.GITHUB_REPOSITORY}/git/tags/${object.sha}`)).object;
  if (object.type !== 'commit') throw Error('Release tag must resolve to a commit');
  if (process.env.EXPECTED_SHA && object.sha !== process.env.EXPECTED_SHA) throw Error('Release tag does not match successful CI commit');
  // Publication waits for the independent main Android build/emulator checks.
  let validated = false;
  for (let attempt = 0; attempt < 40; attempt++) {
    const runs = JSON.parse(gh('api', `repos/${process.env.GITHUB_REPOSITORY}/actions/workflows/android.yml/runs?head_sha=${object.sha}&event=push`)).workflow_runs;
    const run = runs[0];
    if (run?.status === 'completed') {
      if (run.conclusion !== 'success') throw Error('Android main checks did not pass');
      validated = true;
      break;
    }
    await new Promise(resolve => setTimeout(resolve, 30000));
  }
  if (!validated) throw Error('Android main checks have not completed; retry after they pass');
  appendFileSync(process.env.GITHUB_ENV, `ANDROID_RELEASE_SHA=${object.sha}\nANDROID_RELEASE_VERSION=${version}\n`);
} else if (process.argv[2] === 'publish') {
  const current = JSON.parse(readFileSync('package.json')).version;
  if (current !== process.env.ANDROID_RELEASE_VERSION) throw Error('Release manifest mismatch');
  const apk = `sudoku-${current}.apk`;
  copyFileSync('android/app/build/outputs/apk/release/app-release.apk', apk);
  execFileSync(`${process.env.ANDROID_HOME}/build-tools/36.0.0/apksigner`, ['verify', '--verbose', '--print-certs', apk], { stdio:'inherit' });
  writeFileSync(`${apk}.sha256`, `${createHash('sha256').update(readFileSync(apk)).digest('hex')}  ${apk}\n`);
  // Do not overwrite existing assets on retries: a published APK is immutable.
  const assets = JSON.parse(gh('api', `repos/${process.env.GITHUB_REPOSITORY}/releases/tags/v${current}`)).assets;
  for (const name of [apk, `${apk}.sha256`]) {
    if (assets.some(asset => asset.name === name)) {
      const old = execFileSync('gh',['api',`repos/${process.env.GITHUB_REPOSITORY}/releases/assets/${assets.find(a=>a.name===name).id}`,'-H','Accept: application/octet-stream']);
      if (!old.equals(readFileSync(name))) throw Error(`Existing release asset differs: ${name}`);
    } else gh('release','upload',`v${current}`,name);
  }
} else throw Error('Expected resolve or publish');
