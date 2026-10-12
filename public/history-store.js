// IndexedDB transactions only; retention and game transitions live in Rust.
(() => {
  let database;
  const open = () => database ||= new Promise((resolve, reject) => {
    const request = indexedDB.open('sudoku_game_history', 1);
    let blocked = false;
    request.onupgradeneeded = () => {
      request.result.createObjectStore('games', { keyPath: 'id' });
      request.result.createObjectStore('meta');
    };
    request.onsuccess = () => {
      if (blocked) { request.result.close(); return; }
      request.result.onversionchange = () => { request.result.close(); database = undefined; };
      resolve(request.result);
    };
    request.onerror = () => { database = undefined; reject(request.error); };
    request.onblocked = () => { blocked = true; reject(new Error('History database blocked')); };
  }).catch(error => { database = undefined; throw error; });
  const transaction = async (mode, work) => {
    const db = await open();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(['games', 'meta'], mode);
      let result;
      tx.oncomplete = () => resolve(result);
      tx.onabort = tx.onerror = () => reject(tx.error || new Error('History transaction failed'));
      try { work(tx.objectStore('games'), tx.objectStore('meta'), value => { result = value; }); }
      catch (error) { tx.abort(); reject(error); }
    });
  };
  window.__sudokuHistory = {
    list: () => transaction('readonly', (games, meta, done) => {
      const request = games.getAll(); request.onsuccess = () => done(JSON.stringify(request.result));
    }),
    save: (json, activate) => transaction('readwrite', (games, meta) => {
      const value = JSON.parse(json);
      const request = games.get(value.id);
      request.onsuccess = () => {
        const previous = request.result;
        value.pinned = previous?.pinned || false;
        if (activate || !previous) {
          const counter = meta.get('counter');
          counter.onsuccess = () => {
            const previousCounter = counter.result ?? 0;
            if (!Number.isSafeInteger(previousCounter) || previousCounter < 0 || previousCounter === Number.MAX_SAFE_INTEGER) { games.transaction.abort(); return; }
            value.order = previousCounter + 1;
            value.last_played_ms = Date.now();
            meta.put(value.order, 'counter'); games.put(value);
          };
        } else {
          value.order = previous.order; value.last_played_ms = previous.last_played_ms;
          games.put(value);
        }
      };
    }),
    pin: (id, pinned) => transaction('readwrite', games => {
      const request = games.get(id);
      request.onsuccess = () => {
        if (!request.result) { games.transaction.abort(); return; }
        games.put({ ...request.result, pinned });
      };
    }),
    prune: (json, active) => transaction('readwrite', games => {
      for (const { id, order } of JSON.parse(json)) {
        const request = games.get(id);
        request.onsuccess = () => {
          let persistedActive;
          try { persistedActive = JSON.parse(localStorage.sudoku_state).game_id; }
          catch { games.transaction.abort(); return; }
          if (request.result && !request.result.pinned && request.result.order === order && id !== active && id !== persistedActive) games.delete(id);
        };
      }
    }),
  };
})();
