// Optional browser check of the staged native bundle, without an Android SDK.
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const browser = await chromium.launch({headless:true,args:['--no-sandbox']});
const mime = {'.html':'text/html','.js':'application/javascript','.wasm':'application/wasm','.css':'text/css','.png':'image/png','.json':'application/json'};
try {
  for (const viewport of [{width:390,height:844},{width:800,height:1280}]) {
    const context = await browser.newContext({viewport,offline:true});
    // Intercept the stable local origin exactly as the native asset server does.
    await context.route('https://localhost/**', async route => {
      const pathname = new URL(route.request().url()).pathname;
      const file = ['/','/config','/help'].includes(pathname) ? 'index.html' : pathname.slice(1);
      try { await route.fulfill({body:await readFile(path.join('.android-web',file)),contentType:mime[path.extname(file)] || 'application/octet-stream'}); }
      catch { await route.fulfill({status:404,body:''}); }
    });
    const page = await context.newPage();
    const errors=[];
    page.on('pageerror',error=>errors.push(error.message));
    for (const route of ['/','/config','/help','/']) {
      await page.goto(`https://localhost${route}`);
      await page.locator('#loading').waitFor({state:'detached'});
      assert.equal(await page.evaluate(()=>window.__SUDOKU_NATIVE__),true);
      assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth <= innerWidth),true);
      assert.equal(await page.evaluate(async()=>(await navigator.serviceWorker.getRegistrations()).length),0);
    }
    const board=await page.evaluate(()=>JSON.parse(localStorage.sudoku_state).board);
    await page.reload();
    await page.locator('#loading').waitFor({state:'detached'});
    assert.deepEqual(await page.evaluate(()=>JSON.parse(localStorage.sudoku_state).board),board);
    assert.deepEqual(errors,[]);
    await context.close();
    console.log(`Native staging offline routes/layout/save passed: ${viewport.width}x${viewport.height}`);
  }
} finally { await browser.close(); }
