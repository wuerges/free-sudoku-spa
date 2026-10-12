// Optional build-artifact check for web and Android-staged URL configuration.
import { createServer } from 'node:http';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import { stage } from '../scripts/android.mjs';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const expected = process.env.SHARING_EXPECTED_ORIGIN;
assert.ok(expected, 'Set SHARING_EXPECTED_ORIGIN to the build origin');
const source = process.env.SHARING_BUILD_DIR || 'dist';
const native = await mkdtemp(path.join(tmpdir(), 'sudoku-url-native-'));
stage(source, native);
const mime = {'.html':'text/html','.css':'text/css','.js':'application/javascript','.wasm':'application/wasm','.json':'application/json','.png':'image/png'};
let directory=source;
const server=createServer(async(req,res)=>{
  const pathname=new URL(req.url,'http://localhost').pathname;
  const name=pathname==='/'?'index.html':pathname.slice(1);
  try {const bytes=await readFile(path.join(directory,name));res.writeHead(200,{'Content-Type':mime[path.extname(name)]||'application/octet-stream'});res.end(bytes);}
  catch {res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({headless:true,executablePath:process.env.CHROMIUM_PATH||undefined,args:['--no-sandbox']});
try {
  for(const isNative of [false,true]) {
    directory=isNative?native:source;
    const context=await browser.newContext({viewport:{width:390,height:844},serviceWorkers:'block'});
    const page=await context.newPage();
    await page.goto(`http://127.0.0.1:${server.address().port}`);
    await page.locator('#loading').waitFor({state:'detached'});
    assert.equal(await page.evaluate(()=>Boolean(window.__SUDOKU_NATIVE__)),isNative);
    assert.equal(await page.locator('meta[property="og:url"]').getAttribute('content'),expected+'/');
    for(const selector of ['meta[property="og:image"]','meta[name="twitter:image"]']) assert.equal(await page.locator(selector).getAttribute('content'),expected+'/public/social-sudoku.png');
    await page.getByRole('button',{name:'Ações',exact:true}).click();
    await page.getByRole('button',{name:'Compartilhar',exact:true}).click();
    await page.locator('#share-url').waitFor();
    const firstLink=await page.locator('#share-url').inputValue();
    assert.ok(firstLink.startsWith(expected+'/#sudoku='));
    const first=JSON.parse(Buffer.from(firstLink.split('=')[1],'base64url').toString());
    await page.waitForFunction(seconds=>JSON.parse(localStorage.sudoku_state).timer_seconds>seconds,first.seconds);
    assert.equal(await page.locator('#share-url').inputValue(),firstLink,'Timer ticks must not regenerate the snapshot');
    await page.getByLabel('Sudoku inicial',{exact:true}).check();
    const initialLink=await page.locator('#share-url').inputValue();
    assert.equal(JSON.parse(Buffer.from(initialLink.split('=')[1],'base64url').toString()).seconds,0);
    await page.getByLabel('Estado atual',{exact:true}).check();
    const updated=JSON.parse(Buffer.from((await page.locator('#share-url').inputValue()).split('=')[1],'base64url').toString());
    assert.ok(updated.seconds>first.seconds,'Mode changes capture the latest game state');
    await page.getByLabel('Sudoku inicial',{exact:true}).check();
    await page.getByRole('button',{name:'Fechar',exact:true}).click();
    await page.getByRole('button',{name:'Compartilhar',exact:true}).click();
    assert.equal(await page.getByLabel('Estado atual',{exact:true}).isChecked(),true);
    assert.ok(JSON.parse(Buffer.from((await page.locator('#share-url').inputValue()).split('=')[1],'base64url').toString()).seconds>0);
    await context.close();
  }
  console.log(`Web and Android-staged links/metadata use ${expected}`);
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));await rm(native,{recursive:true,force:true});}
