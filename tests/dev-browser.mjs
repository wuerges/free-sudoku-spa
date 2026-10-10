// Optional Playwright audit. Run after just build, with Node 24 and Just installed.
// Temporary Rust/CSS probes are restored even when a check fails.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile, writeFile, readdir } from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import path from 'node:path';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const mime={'.html':'text/html','.css':'text/css','.js':'application/javascript','.wasm':'application/wasm','.json':'application/json','.png':'image/png'};
async function digest(dir) {
  const hash=createHash('sha256');
  for(const name of (await readdir(dir)).sort()) {
    const file=path.join(dir,name);
    const {stat}=await import('node:fs/promises');
    hash.update(name);
    hash.update((await stat(file)).isDirectory()?await digest(file):await readFile(file));
  }
  return hash.digest('hex');
}
const releaseDigest=await digest('dist');
const cssFile='style/input.css', rustFile='src/components/game_page.rs';
const css=await readFile(cssFile,'utf8'), rust=await readFile(rustFile,'utf8');
const cssProbe=css+'\n.game-page { --dev-reload-probe: 1; }\n';
assert.ok(rust.includes('class="game-page"'));
const rustProbe=rust.replace('class="game-page"','class="game-page opacity-[0.97]"');
const server=createServer(async(req,res)=>{
  const route=new URL(req.url,'http://localhost').pathname;
  const file=['/','/config','/help'].includes(route)?'index.html':route.slice(1);
  try { const data=await readFile(path.join('dist',file));res.writeHead(200,{'Content-Type':mime[path.extname(file)]||'application/octet-stream'});res.end(data); }
  catch {res.writeHead(404);res.end();}
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const port=server.address().port,url=`http://127.0.0.1:${port}`;
const browser=await chromium.launch({headless:true,executablePath:process.env.CHROMIUM_PATH||undefined,args:['--no-sandbox']});
let child,logs='';
async function until(fn,label,timeout=180000) {
  const deadline=Date.now()+timeout;
  while(Date.now()<deadline) {
    if(child?.exitCode!=null) throw new Error(`Dev server exited: ${logs.slice(-3000)}`);
    try {if(await fn()) return;} catch {}
    await new Promise(r=>setTimeout(r,300));
  }
  throw new Error(`${label} timed out: ${logs.slice(-3000)}`);
}
async function restore(file,original,probe) {
  const current=await readFile(file,'utf8');
  if(current===original) return;
  if(current!==probe) throw new Error(`Concurrent edit to ${file}; leaving it intact`);
  await writeFile(file,original);
}
try {
  const context=await browser.newContext({viewport:{width:390,height:844}});
  const page=await context.newPage(),errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  const read=()=>page.evaluate(()=>JSON.parse(localStorage.getItem('sudoku_state')));
  await page.goto(url);await page.locator('.sudoku-cell').first().waitFor();
  await page.evaluate(async()=>{await navigator.serviceWorker.ready;});
  await page.reload();await page.waitForFunction(()=>!!navigator.serviceWorker.controller);
  const board=(await read()).board;
  await context.setOffline(true);
  for(const route of ['/','/config','/help']) {await page.goto(url+route);await page.locator('main').waitFor();}
  await page.goto(url);await page.locator('.sudoku-cell').first().waitFor();
  assert.deepEqual((await read()).board,board);
  await context.setOffline(false);
  await page.evaluate(async()=>{await caches.open('other-app');});
  console.log('PASS release PWA: offline routes and saved-game reload');
  await new Promise(r=>server.close(r));
  child=spawn('just',['dev','--address','127.0.0.1','--port',String(port)],{detached:true,stdio:['ignore','pipe','pipe']});
  for(const stream of [child.stdout,child.stderr]) stream.on('data',data=>{logs=(logs+data).slice(-80000);});
  child.on('error',error=>{logs+=String(error);});
  await until(async()=>{const r=await fetch(url+'/sw.js');return r.ok&&(await r.text()).includes('Development-only worker');},'dev startup');
  assert.match(logs,/Finished `dev` profile/);
  await page.evaluate(async()=>{await (await navigator.serviceWorker.getRegistration()).update();});
  await page.waitForFunction(async()=>!(await caches.keys()).some(k=>k.startsWith('sudoku-offline-')));
  assert.ok(await page.evaluate(async()=>(await caches.keys()).includes('other-app')));
  await page.reload();await page.locator('.sudoku-cell').first().waitFor();
  await page.locator('#loading').waitFor({state:'detached'});
  assert.deepEqual((await read()).board,board);
  for(const width of [390,800]) {
    await page.setViewportSize({width,height:width===390?844:1280});
    for(const route of ['/','/config','/help']) {
      await page.goto(url+route);await page.locator('main').waitFor();
      assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
    }
  }
  await page.goto(url);await page.locator('.game-page').waitFor();
  await writeFile(cssFile,cssProbe);
  await until(()=>page.evaluate(()=>getComputedStyle(document.querySelector('.game-page')).getPropertyValue('--dev-reload-probe').trim()==='1'),'CSS hot reload');
  assert.deepEqual((await read()).board,board);
  await writeFile(rustFile,rustProbe);
  await until(()=>page.evaluate(()=>getComputedStyle(document.querySelector('.game-page')).opacity==='0.97'),'Rust class and CSS hot reload');
  assert.deepEqual((await read()).board,board);
  await restore(cssFile,css,cssProbe);await restore(rustFile,rust,rustProbe);
  await until(()=>page.evaluate(()=>{
    const e=document.querySelector('.game-page');
    return e&&!e.classList.contains('opacity-[0.97]')&&getComputedStyle(e).getPropertyValue('--dev-reload-probe').trim()==='';
  }),'restored source reload');
  assert.deepEqual((await read()).board,board);
  assert.equal(await digest('dist'),releaseDigest);
  assert.deepEqual(errors,[]);
  console.log('PASS just dev: debug profile, cached-worker takeover, mobile/tablet routes, CSS/Rust-class reload, saved progress and isolated release output');
  await context.close();
} finally {
  await restore(cssFile,css,cssProbe);await restore(rustFile,rust,rustProbe);
  await browser.close();
  if(server.listening) await new Promise(r=>server.close(r));
  if(child?.pid) {
    try {process.kill(-child.pid,'SIGINT');} catch(error) {if(error.code!=='ESRCH') throw error;}
    if(child.exitCode===null&&child.signalCode===null) {
      await Promise.race([new Promise(r=>child.once('close',r)),new Promise(r=>setTimeout(r,5000))]);
      if(child.exitCode===null&&child.signalCode===null) try {process.kill(-child.pid,'SIGKILL');} catch(error) {if(error.code!=='ESRCH') throw error;}
    }
  }
}
