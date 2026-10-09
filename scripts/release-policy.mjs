import { readFileSync, writeFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

export function validateVersion(files, baseVersion, body, initial = false) {
  const version = JSON.parse(files['package.json']).version;
  const lock = JSON.parse(files['package-lock.json']);
  const cargo = files['Cargo.toml'].match(/^version = "([^"]+)"/m)?.[1];
  const cargoLock = files['Cargo.lock'].match(/name = "free-sudoku-pwa"\nversion = "([^"]+)"/)?.[1];
  if (!/^\d+\.\d+\.\d+$/.test(version) || [lock.version, lock.packages[''].version, cargo, cargoLock].some(v => v !== version)) throw Error('Manifest and lockfile versions must agree');
  const notes = files['CHANGELOG.md'].split(`## [${version}]`)[1]?.split(/^## /m)[0]?.trim();
  if (!notes) throw Error('Current version needs a changelog entry');
  const compatibility = body.match(/^Game-state compatibility: (compatible|breaking)\s*$/m)?.[1];
  if (!compatibility) throw Error('Declare Game-state compatibility: compatible or breaking in the PR body');
  if (initial) {
    if (version !== '0.9.0' || compatibility !== 'compatible') throw Error('Initial release must be compatible 0.9.0');
  } else {
    const [major, minor] = baseVersion.split('.').map(Number);
    const expected = compatibility === 'breaking' ? `${major + 1}.0.0` : `${major}.${minor + 1}.0`;
    if (version !== expected) throw Error(`Expected ${expected} for ${compatibility} changes`);
  }
  return { version, notes };
}

const filenames = ['Cargo.toml', 'Cargo.lock', 'package.json', 'package-lock.json', 'CHANGELOG.md'];
function api(path) { return JSON.parse(execFileSync('gh', ['api', path], { encoding: 'utf8' })); }
function baseFile(repo, sha, name) {
  return Buffer.from(api(`repos/${repo}/contents/${name}?ref=${sha}`).content, 'base64').toString();
}
export function assertTagTarget(object, sha) {
  if (object.type !== 'commit' || object.sha !== sha) throw Error('Existing release tag points at a different commit');
}
function optionalApi(path) {
  try { return api(path); }
  catch (error) {
    if (/HTTP 404/.test(error.stderr?.toString() ?? '')) return null;
    throw error;
  }
}
export function publishRelease({version, notes}, sha, repo, client, create) {
  const tag = `v${version}`;
  const refs = client(`repos/${repo}/git/matching-refs/tags/${tag}`).filter(r => r.ref === `refs/tags/${tag}`);
  if (refs.length) {
    let object = refs[0].object;
    while (object.type === 'tag') object = client(`repos/${repo}/git/tags/${object.sha}`).object;
    assertTagTarget(object, sha);
  }
  const release = client(`repos/${repo}/releases/tags/${tag}`);
  if (release) {
    if (!refs.length || release.draft) throw Error('Existing release must have the expected published tag');
    return;
  }
  create(tag, notes);
}
export function selectMergedPR(pulls, sha) {
  return pulls.find(p => p.merged_at && p.base.ref === 'main' && p.merge_commit_sha === sha);
}
function main() {
  const event = JSON.parse(readFileSync(process.env.GITHUB_EVENT_PATH, 'utf8'));
  const repo = process.env.GITHUB_REPOSITORY;
  if (process.argv[2] === 'publish') {
    publishRelease(JSON.parse(readFileSync('release-metadata.json', 'utf8')), process.env.GITHUB_SHA, repo, optionalApi, (tag, notes) => {
      writeFileSync('release-notes.md', notes);
      execFileSync('gh', ['release', 'create', tag, '--repo', repo, '--target', process.env.GITHUB_SHA, '--title', tag, '--notes-file', 'release-notes.md'], { stdio: 'inherit' });
    });
    return;
  }
  let pr = event.pull_request;
  if (!pr) {
    const associated = api(`repos/${repo}/commits/${event.after}/pulls`);
    pr = selectMergedPR(associated, event.after);
    if (!pr) return; // Direct pushes do not publish releases.
  }
  const baseSha = event.pull_request ? pr.base.sha : event.before;
  const listing = api(`repos/${repo}/contents?ref=${baseSha}`);
  const initial = !listing.some(f => f.name === 'CHANGELOG.md');
  const baseVersion = initial ? undefined : JSON.parse(baseFile(repo, baseSha, 'package.json')).version;
  const files = Object.fromEntries(filenames.map(name => [name, readFileSync(name, 'utf8')]));
  const metadata = validateVersion(files, baseVersion, pr.body ?? '', initial);
  writeFileSync('release-metadata.json', JSON.stringify(metadata));
  if (process.env.GITHUB_OUTPUT) writeFileSync(process.env.GITHUB_OUTPUT, `version=${metadata.version}\nrelease=${!event.pull_request}\n`, { flag: 'a' });
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
