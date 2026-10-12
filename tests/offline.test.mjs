import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, writeFile, readFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import vm from 'node:vm';
import { MessageChannel } from 'node:worker_threads';
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

function worker(source, { failAsset = null, offline = false, clients = [], stores = new Map(), pendingWorker = false, activeVersion = null } = {}) {
  const handlers = {};
  let skips = 0, claims = 0;
  const origin = 'https://sudoku.test';
  const key = value => new URL(typeof value === 'string' ? value : value.url, origin).href;
  let networkCalls = 0;
  const context = {
    self: { registration: {waiting:pendingWorker?{}:null,active:activeVersion?{postMessage:(_,ports)=>{ports[0].postMessage({type:'SUDOKU_UPDATE_STATUS',version:activeVersion});ports[0].close();}}:null}, location: { origin }, addEventListener: (name, callback) => { handlers[name] = callback; }, skipWaiting: async () => {skips++;}, clients: {matchAll: async () => clients, claim: async () => {claims++;}} },
    MessageChannel, setTimeout, clearTimeout,
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
          put: async (request, response) => entries.set(key(request), response),
          delete: async request => entries.delete(key(request)),
        };
      },
      keys: async () => [...stores.keys()],
      delete: async name => stores.delete(name),
    },
  };
  vm.runInNewContext(source, context);
  return {
    stores,
    get skips() {return skips;}, get claims() {return claims;},
    message: async data => {let pending;handlers.message({data, ports:[], waitUntil:p=>{pending=p;}});await pending;},
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
  assert.equal(w.claims,0);assert.equal(w.skips,0);
  assert.deepEqual([...w.stores.keys()].sort(), ['other-app-cache', 'sudoku-offline-' + version].sort());
});

function client(id, options={}) {
  const c={id,url:'https://sudoku.test/',aborts:0,commits:0,postMessage(message,ports=[]) {
    if (message.type==='SUDOKU_UPDATE_ABORT') {this.aborts++;return;}
    if (options.legacy) return;
    if (message.phase==='commit') {this.commits++;options.onCommit?.();}
    ports[0].postMessage({type:'SUDOKU_UPDATE_ACK',version:options.wrongVersion?'other':message.version,ready:message.phase==='running'?options.running!==false:message.phase==='commit'?options.commit!==false:options.ready!==false});
    ports[0].close();
  }};return c;
}
test('explicit activation requires every tab and rejects stale versions',async t=>{
  const dir=await fixture(t);const {version}=await generateOfflineBundle(dir);const source=await readFile(path.join(dir,'sw.js'),'utf8');
  for (const options of [{ready:false},{wrongVersion:true},{commit:false},{legacy:true}]) {
    const a=client('a'),b=client('b',options);const w=worker(source,{clients:[a,b]});await w.lifecycle('install');
    await w.message({type:'SUDOKU_UPDATE_ACTIVATE',version:'stale'});assert.equal(w.skips,0);
    await w.message({type:'SUDOKU_UPDATE_ACTIVATE',version});assert.equal(w.skips,0);assert.equal(a.aborts,1);
  }
});
test('all-tab consent survives worker restart and claims only the approved bundle',async t=>{
  const dir=await fixture(t);const {version}=await generateOfflineBundle(dir);const source=await readFile(path.join(dir,'sw.js'),'utf8');
  const a=client('a'),b=client('b');const w=worker(source,{clients:[a,b]});await w.lifecycle('install');
  w.stores.set('other-app-cache',new Map());await w.message({type:'SUDOKU_UPDATE_ACTIVATE',version});assert.equal(w.skips,1);assert.equal(a.commits,1);assert.equal(b.commits,1);
  const restarted=worker(source,{stores:w.stores,clients:[a,b]});await restarted.lifecycle('activate');assert.equal(restarted.claims,1);assert.ok(restarted.stores.has('other-app-cache'));
  assert.equal(await restarted.stores.get('sudoku-offline-'+version).has('https://sudoku.test/__sudoku_update_commit'),false);
});
test('a newly opened tab during commit blocks activation',async t=>{
  const dir=await fixture(t);const {version}=await generateOfflineBundle(dir);const source=await readFile(path.join(dir,'sw.js'),'utf8');
  const clients=[];clients.push(client('a',{onCommit:()=>clients.push(client('new',{ready:false}))}));
  const w=worker(source,{clients});await w.lifecycle('install');await w.message({type:'SUDOKU_UPDATE_ACTIVATE',version});assert.equal(w.skips,0);
});

test('coordinated activation retains old assets until every client runs the new bundle',async t=>{
  const dir=await fixture(t);const {version}=await generateOfflineBundle(dir);const source=await readFile(path.join(dir,'sw.js'),'utf8');
  const options={running:false};const a=client('a',options);const w=worker(source,{clients:[a],offline:true});await w.lifecycle('install');
  const old=new Map([['https://sudoku.test/app-deadbeef.js',new Response('previous script')]]);w.stores.set('sudoku-offline-old',old);
  await w.message({type:'SUDOKU_UPDATE_ACTIVATE',version});await w.lifecycle('activate');assert.ok(w.stores.has('sudoku-offline-old'));
  assert.equal(await (await w.request('/app-deadbeef.js','cors')).text(),'previous script');
  await w.message({type:'SUDOKU_UPDATE_CLEANUP',version});assert.ok(w.stores.has('sudoku-offline-old'));
  options.running=true;await w.message({type:'SUDOKU_UPDATE_CLEANUP',version});assert.equal(w.stores.has('sudoku-offline-old'),false);
});

test('an old active worker never prunes a waiting update cache',async t=>{
  const dir=await fixture(t);const {version}=await generateOfflineBundle(dir);const source=await readFile(path.join(dir,'sw.js'),'utf8');
  const w=worker(source,{clients:[client('a')],pendingWorker:true});await w.lifecycle('install');w.stores.set('sudoku-offline-next',new Map());
  await w.message({type:'SUDOKU_UPDATE_CLEANUP',version});assert.ok(w.stores.has('sudoku-offline-next'));
});

test('an older worker event cannot prune the newly active bundle',async t=>{
  const dir=await fixture(t);const {version}=await generateOfflineBundle(dir);const source=await readFile(path.join(dir,'sw.js'),'utf8');
  const w=worker(source,{clients:[client('a')],activeVersion:'newer'});await w.lifecycle('install');w.stores.set('sudoku-offline-newer',new Map());
  await w.message({type:'SUDOKU_UPDATE_CLEANUP',version});assert.ok(w.stores.has('sudoku-offline-newer'));
});
