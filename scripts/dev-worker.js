// Development-only worker. Take over older localhost Sudoku caches, then let
// all requests use the network. Never change localStorage or production workers.
self.addEventListener('install', event => event.waitUntil(self.skipWaiting()));
self.addEventListener('activate', event => event.waitUntil((async () => {
  for (const key of await caches.keys()) {
    if (key.startsWith('sudoku-offline-') || key === 'sudoku-v1') await caches.delete(key);
  }
  await self.clients.claim();
})()));
