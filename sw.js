// Trunk replaces this marker with the complete, content-versioned asset list.
/* OFFLINE_BUNDLE */
const CACHE_PREFIX = 'sudoku-offline-';
const CACHE = CACHE_PREFIX + VERSION;
const COMMIT_MARKER = '/__sudoku_update_commit';
let coordinating = false;
const ASSET_URLS = new Set(ASSETS.map(url => new URL(url, self.location.origin).href));

self.addEventListener('install', event => {
  event.waitUntil((async () => {
    const cache = await caches.open(CACHE);
    try {
      // Bypass HTTP caches so an update never precaches stale HTML or metadata.
      await cache.addAll(ASSETS.map(url => new Request(url, { cache: 'reload' })));
    } catch (error) {
      await caches.delete(CACHE);
      throw error;
    }
  })());
  // Explicit activation requires fresh consent from every open Sudoku window.
});

self.addEventListener('activate', event => {
  event.waitUntil((async () => {
    const cache = await caches.open(CACHE);
    const approved = await cache.match(COMMIT_MARKER);
    if (approved) {
      await self.clients.claim();
      await cache.delete(COMMIT_MARKER);
    }
    // Keep the previous assets during coordinated reloads, including a late tab
    // that opens between the membership check and activation.
    if (!approved) await pruneCaches(true);
  })());
});

self.addEventListener('fetch', event => {
  const request = event.request;
  if (request.method !== 'GET' || new URL(request.url).origin !== self.location.origin) return;
  if (request.mode === 'navigate') {
    event.respondWith((async () => {
      const cache = await caches.open(CACHE);
      return await cache.match('/index.html') || new Response('Offline app unavailable', { status: 503 });
    })());
  } else if (ASSET_URLS.has(request.url) || /-[0-9a-f]+(?:_bg)?\.(?:js|wasm|css)$/.test(new URL(request.url).pathname)) {
    event.respondWith((async () => {
      const cache = await caches.open(CACHE);
      const response = await cache.match(request);
      if (response) return response;
      for (const key of await caches.keys()) {
        if (key.startsWith(CACHE_PREFIX)) {
          const previous = await (await caches.open(key)).match(request);
          if (previous) return previous;
        }
      }
      return fetch(request);
    })());
  }
});

// The worker must query live pages anew: it can be terminated between messages.
const sudokuClients = async () => (await self.clients.matchAll({type: 'window', includeUncontrolled: true}))
  .filter(client => {
    const url = new URL(client.url);
    return url.origin === self.location.origin && ['/', '/index.html', '/config', '/help', '/history'].includes(url.pathname);
  });
function activeVersionMatches(worker) {
  return new Promise(resolve => {
    const channel = new MessageChannel();
    const timer = setTimeout(() => {channel.port1.close();resolve(false);},3000);
    channel.port1.onmessage = event => {
      clearTimeout(timer);channel.port1.close();
      resolve(event.data?.type === 'SUDOKU_UPDATE_STATUS' && event.data.version === VERSION);
    };
    try {worker.postMessage({type:'SUDOKU_UPDATE_STATUS'},[channel.port2]);}
    catch {clearTimeout(timer);channel.port1.close();resolve(false);}
  });
}
async function pruneCaches(activating = false) {
  const active = self.registration?.active;
  // An older worker's pending event may outlive the controller change.
  if (active && !activating && !await activeVersionMatches(active)) return;
  // Never delete a newer bundle being installed or waiting for consent.
  if (self.registration?.installing || self.registration?.waiting) return;
  const keys = await caches.keys();
  if (self.registration?.installing || self.registration?.waiting || (active && self.registration.active !== active)) return;
  await Promise.all(keys.filter(key => key !== CACHE && (key.startsWith(CACHE_PREFIX) || key === 'sudoku-v1')).map(key => caches.delete(key)));
}
async function cleanup() {
  const clients = await sudokuClients();
  if ((await Promise.all(clients.map(c => ask(c, 'running')))).every(Boolean)) await pruneCaches();
}
function ask(client, phase) {
  return new Promise(resolve => {
    const channel = new MessageChannel();
    const timer = setTimeout(() => {channel.port1.close(); resolve(false);}, 3000);
    channel.port1.onmessage = event => {
      clearTimeout(timer); channel.port1.close();
      resolve(event.data?.type === 'SUDOKU_UPDATE_ACK' && event.data.version === VERSION && event.data.ready === true);
    };
    try {client.postMessage({type: 'SUDOKU_UPDATE_QUERY', phase, version: VERSION}, [channel.port2]);}
    catch {clearTimeout(timer); channel.port1.close(); resolve(false);}
  });
}
async function coordinate() {
  if (coordinating) return;
  coordinating = true;
  let clients = [];
  let activated = false;
  try {
    clients = await sudokuClients();
    if (!clients.length || !(await Promise.all(clients.map(c => ask(c, 'probe')))).every(Boolean)) return;
    if (!(await Promise.all(clients.map(c => ask(c, 'commit')))).every(Boolean)) return;
    const current = await sudokuClients();
    if (current.map(c => c.id).sort().join(',') !== clients.map(c => c.id).sort().join(',')) return;
    const cache = await caches.open(CACHE);
    await cache.put(COMMIT_MARKER, new Response('approved'));
    await self.skipWaiting();
    activated = true;
  } finally {
    if (!activated) {
      await (await caches.open(CACHE)).delete(COMMIT_MARKER);
      for (const client of clients) client.postMessage({type: 'SUDOKU_UPDATE_ABORT', version: VERSION});
    }
    coordinating = false;
  }
}
self.addEventListener('message', event => {
  if (event.data?.type === 'SUDOKU_UPDATE_STATUS') {
    event.ports?.[0]?.postMessage({type: 'SUDOKU_UPDATE_STATUS', version: VERSION});
  } else if (event.data?.type === 'SUDOKU_UPDATE_CLEANUP' && event.data.version === VERSION) {
    event.waitUntil(cleanup());
  } else if (event.data?.type === 'SUDOKU_UPDATE_ACTIVATE' && event.data.version === VERSION) {
    event.waitUntil(coordinate());
  }
});
