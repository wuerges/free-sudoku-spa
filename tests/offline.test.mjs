import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import vm from 'node:vm';
import { generateOfflineBundle } from '../scripts/offline-bundle.mjs';

async function fixture(t) {
  const dir = await mkdtemp(path.join(tmpdir(), 'sudoku-offline-'));
  t.after(() => rm(dir, { recursive: true, force: true }));
  await mkdir(path.join(dir, 'public/icons'), { recursive: true });
  for (const [name, body] of Object.entries({
    'index.html': '<html>bundle A</html>', 'manifest.json': '{}',
    'app-123.js': 'init()', 'app-123_bg.wasm': 'wasm',
    'output-123.css': 'body{}', 'public/icons/icon.png': 'png',
  })) await writeFile(path.join(dir, name), body);
  return dir;
}

function worker(source, { failAsset = null, offline = false } = {}) {
  const handlers = {}, stores = new Map();
  const origin = 'https://sudoku.test';
  const key = value => new URL(typeof value === 'string' ? value : value.url, origin).href;
  let networkCalls = 0;
  const context = {
    self: { location: { origin }, addEventListener: (name, callback) => { handlers[name] = callback; } },
    Request: class extends Request { constructor(url, options) { super(new URL(url, origin), options); } },
    URL, Set, Response,
    fetch: async () => { networkCalls++; if (offline) throw new Error('offline'); return new Response('network'); },
    caches: {
      open: async name => {
        if (!stores.has(name)) stores.set(name, new Map());
        const entries = stores.get(name);
        return {
          addAll: async requests => {
            for (const request of requests) {
              assert.equal(request.cache, 'reload');
              if (key(request) === key(failAsset || '/not-present')) throw new Error('missing asset');
            }
            for (const request of requests) entries.set(key(request), new Response(key(request)));
          },
          match: async request => entries.get(key(request))?.clone(),
        };
      },
      keys: async () => [...stores.keys()],
      delete: async name => stores.delete(name),
    },
  };
  vm.runInNewContext(source, context);
  return {
    stores,
    get networkCalls() { return networkCalls; },
    lifecycle: async name => {
      let pending;
      handlers[name]({ waitUntil: promise => { pending = promise; } });
      await pending;
    },
    request: (url, mode = 'navigate', method = 'GET') => {
      let pending;
      handlers.fetch({ request: { url: key(url), mode, method }, respondWith: promise => { pending = promise; } });
      return pending;
    },
  };
}

test('generator lists full assets, ignores worker, and versions contents and logic', async t => {
  const dir = await fixture(t);
  const a = await generateOfflineBundle(dir);
  assert.equal(a.assets.length, 6);
  assert.ok(a.assets.includes('/public/icons/icon.png'));
  assert.ok(!a.assets.includes('/sw.js'));
  assert.equal((await generateOfflineBundle(dir)).version, a.version);
  await writeFile(path.join(dir, 'manifest.json'), '{"name":"new"}');
  const b = await generateOfflineBundle(dir);
  assert.notEqual(b.version, a.version);
  // Put the alternate template outside the asset tree for this check.
  const alt = await mkdtemp(path.join(tmpdir(), 'sudoku-template-'));
  t.after(() => rm(alt, { recursive: true, force: true }));
  const altPath = path.join(alt, 'sw.js');
  await writeFile(altPath, (await readFile('sw.js', 'utf8')) + '\n// changed logic');
  assert.notEqual((await generateOfflineBundle(dir, altPath)).version, b.version);
});

test('generator rejects incomplete app bundles', async t => {
  const dir = await fixture(t);
  await rm(path.join(dir, 'app-123_bg.wasm'));
  await assert.rejects(generateOfflineBundle(dir), /Missing .wasm bundle/);
});

test('offline routes share app shell and every asset loads without network', async t => {
  const dir = await fixture(t);
  const { assets } = await generateOfflineBundle(dir);
  const w = worker(await readFile(path.join(dir, 'sw.js'), 'utf8'), { offline: true });
  await w.lifecycle('install');
  for (const route of ['/', '/help', '/config', '/help?source=offline']) {
    const response = await w.request(route);
    assert.equal(response.status, 200);
    assert.equal(await response.text(), 'https://sudoku.test/index.html');
  }
  for (const url of assets) assert.equal((await w.request(url, 'cors')).status, 200);
  assert.equal(w.networkCalls, 0);
  assert.equal(w.request('/unrelated.json', 'cors'), undefined);
  assert.equal(w.request('https://other.test/app.js', 'cors'), undefined);
  assert.equal(w.request('/config', 'navigate', 'POST'), undefined);
});

test('failed installation removes only incomplete new cache', async t => {
  const dir = await fixture(t);
  await generateOfflineBundle(dir);
  const w = worker(await readFile(path.join(dir, 'sw.js'), 'utf8'), { failAsset: '/app-123_bg.wasm' });
  w.stores.set('sudoku-offline-old', new Map());
  await assert.rejects(w.lifecycle('install'), /missing asset/);
  assert.deepEqual([...w.stores.keys()], ['sudoku-offline-old']);
});

test('activation removes Sudoku legacy caches only and never forces takeover', async t => {
  const dir = await fixture(t);
  const { version } = await generateOfflineBundle(dir);
  const w = worker(await readFile(path.join(dir, 'sw.js'), 'utf8'));
  for (const key of ['sudoku-v1', 'sudoku-offline-old', 'other-app-cache']) w.stores.set(key, new Map());
  await w.lifecycle('install');
  await w.lifecycle('activate');
  assert.deepEqual([...w.stores.keys()].sort(), ['other-app-cache', 'sudoku-offline-' + version].sort());
});
