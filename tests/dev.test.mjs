import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile, writeFile, mkdtemp, rm } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import path from 'node:path';
import vm from 'node:vm';

const worker = await readFile('scripts/dev-worker.js', 'utf8');

test('development worker replaces old Sudoku caches without caching requests', async () => {
  const handlers = new Map(), deleted = [];
  let skipped = false, claimed = false;
  vm.runInNewContext(worker, {
    self: {
      addEventListener: (type, handler) => handlers.set(type, handler),
      skipWaiting: async () => { skipped = true; },
      clients: { claim: async () => { claimed = true; } },
    },
    caches: {
      keys: async () => ['sudoku-offline-old', 'sudoku-v1', 'other-app', 'sudoku-unrelated'],
      delete: async key => { deleted.push(key); },
    },
  });
  async function fire(type) {
    let pending;
    handlers.get(type)({waitUntil: promise => { pending = promise; }});
    await pending;
  }
  await fire('install');
  assert.equal(skipped, true);
  await fire('activate');
  assert.deepEqual(deleted, ['sudoku-offline-old', 'sudoku-v1']);
  assert.equal(claimed, true);
  assert.equal(handlers.has('fetch'), false);
});

test('development hook changes only the staged worker, keeping production output intact', async t => {
  const stage = await mkdtemp(path.join(tmpdir(), 'sudoku-dev-'));
  t.after(() => rm(stage, {recursive:true,force:true}));
  await writeFile(path.join(stage,'index.html'), '<html>development build</html>');
  await writeFile(path.join(stage,'sw.js'), 'old caching worker');
  const result = spawnSync(process.execPath, ['scripts/dev-bundle.mjs'], {
    encoding:'utf8',env:{...process.env,TRUNK_STAGING_DIR:stage},
  });
  assert.equal(result.status,0,result.stderr);
  assert.equal(await readFile(path.join(stage,'sw.js'),'utf8'), worker);
  assert.equal(await readFile(path.join(stage,'index.html'),'utf8'),'<html>development build</html>');
  const missing = spawnSync(process.execPath, ['scripts/dev-bundle.mjs'], {
    encoding:'utf8',env:{...process.env,TRUNK_STAGING_DIR:''},
  });
  assert.notEqual(missing.status,0);
  assert.match(missing.stderr,/TRUNK_STAGING_DIR is required/);
});

test('development config watches CSS inputs and isolates debug output from release assets', async () => {
  const dev = await readFile('Trunk.dev.toml','utf8');
  const release = await readFile('Trunk.toml','utf8');
  assert.match(dev,/dist = "\.dev-dist"/);
  assert.match(dev,/release = false/);
  assert.match(dev,/watch = \[.*"src".*"style\/input\.css"/);
  assert.doesNotMatch(dev,/watch = \[.*"style\/output\.css"/);
  assert.match(dev,/stage = "pre_build"\ncommand = "npm"\ncommand_arguments = \["run", "css"\]/);
  assert.match(dev,/scripts\/dev-bundle\.mjs/);
  assert.match(release,/dist = "dist"/);
  assert.match(release,/scripts\/offline-bundle\.mjs/);
  assert.doesNotMatch(release,/dev-bundle/);
});

test('Node 24 is consistent across local selection, npm metadata and CI', async () => {
  const pkg = JSON.parse(await readFile('package.json','utf8'));
  const lock = JSON.parse(await readFile('package-lock.json','utf8'));
  assert.equal((await readFile('.nvmrc','utf8')).trim(),'24');
  assert.equal(pkg.engines.node,'24.x');
  assert.equal(lock.packages[''].engines.node,pkg.engines.node);
  const ci = await readFile('.github/workflows/ci.yml','utf8');
  const versions = [...ci.matchAll(/node-version: '([^']+)'/g)].map(m=>m[1]);
  assert.deepEqual(versions,['24','24']);
});
