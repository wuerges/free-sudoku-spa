// Trunk replaces this marker with the complete, content-versioned asset list.
/* OFFLINE_BUNDLE */
const CACHE_PREFIX = 'sudoku-offline-';
const CACHE = CACHE_PREFIX + VERSION;
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
  // Updates wait until every tab controlled by the previous worker has closed.
});

self.addEventListener('activate', event => {
  event.waitUntil((async () => {
    const keys = await caches.keys();
    await Promise.all(keys.filter(key =>
      key !== CACHE && (key.startsWith(CACHE_PREFIX) || key === 'sudoku-v1')
    ).map(key => caches.delete(key)));
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
  } else if (ASSET_URLS.has(request.url)) {
    event.respondWith((async () => {
      const cache = await caches.open(CACHE);
      return await cache.match(request) || fetch(request);
    })());
  }
});
