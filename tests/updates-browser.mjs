// Real browser update lifecycle against two independently versioned complete bundles.
import {createServer} from 'node:http';
import {readFile,writeFile,mkdtemp,cp,rm,mkdir,rename} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {generateOfflineBundle} from '../scripts/offline-bundle.mjs';
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const root=await mkdtemp(path.join(tmpdir(),'sudoku-updates-'));
const screenshots=process.env.UPDATE_SCREENSHOTS || '/tmp/sudoku-updates-review';await mkdir(screenshots,{recursive:true});
const mime={'.html':'text/html','.wasm':'application/wasm','.js':'application/javascript','.css':'text/css','.json':'application/json','.svg':'image/svg+xml','.png':'image/png'};
let active='a',failAsset=null;
const server=createServer(async(req,res)=>{
 const pathname=new URL(req.url,'http://localhost').pathname;
 const file=['/','/config','/help','/history'].includes(pathname)?'index.html':pathname.slice(1);
 if(failAsset===pathname){res.writeHead(503);res.end();return;}
 try{const data=await readFile(path.join(root,active,file));res.writeHead(200,{'Content-Type':mime[path.extname(file)] || 'application/octet-stream','Cache-Control':'no-store'});res.end(data);}catch{res.writeHead(404);res.end();}
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));const url=`http://127.0.0.1:${server.address().port}`;
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH || undefined,args:['--no-sandbox']});
const errors=[];
const catalogs=Object.fromEntries(await Promise.all(['en-US','pt-BR','es-AR','gn-PY'].map(async tag=>[tag,JSON.parse(await readFile('locales/'+tag+'.json','utf8'))])));
const languages={'en-US':'English (United States)','pt-BR':'Português (Brasil)','es-AR':'Español (Argentina)','gn-PY':'Avañe’ẽ (Paraguái)'};
async function pageFor(context){const page=await context.newPage();await page.addInitScript(()=>{window.updateChecks=0;const native=ServiceWorkerRegistration.prototype.update;ServiceWorkerRegistration.prototype.update=function(...args){updateChecks++;return native.apply(this,args);};});page.on('pageerror',e=>errors.push(e.message));return page;}
async function mounted(page){await page.locator('#loading').waitFor({state:'detached'});await page.locator('#history-saving').waitFor({state:'hidden'});}
async function boot(page){await page.goto(url);await mounted(page);await page.evaluate(()=>navigator.serviceWorker.ready);assert.equal(await page.evaluate(()=>navigator.serviceWorker.controller),null);await page.reload();await mounted(page);assert.equal(await page.evaluate(()=>!!navigator.serviceWorker.controller),true);}
async function waiting(page){await page.evaluate(async()=>{const r=await navigator.serviceWorker.getRegistration();await r.update();});await page.locator('#update-now').waitFor({timeout:35000});}
const read=page=>page.evaluate(()=>JSON.parse(localStorage.sudoku_state));
const snapshot=page=>page.evaluate(()=>JSON.parse(sessionStorage.sudoku_update_resume).game);
const stable=s=>Object.fromEntries(['game_id','board','solution','notes','cell_colors','givens','hinted','difficulty','history','redo_stack','effects','highlights','drop_mode','drop_number','note_mode','paused','error_count'].map(k=>[k,s[k]]));
async function foreground(page){await page.evaluate(()=>document.dispatchEvent(new Event('visibilitychange')));}
async function upgraded(page){await page.waitForFunction(()=>document.body.dataset.testBuild==='b');await mounted(page);assert.equal(await page.evaluate(()=>sessionStorage.getItem('sudoku_update_resume')),null);}
try {
 for(const name of ['a','b','broken']) {
  await cp('dist',path.join(root,name),{recursive:true});const file=path.join(root,name,'index.html');
  let html=(await readFile(file,'utf8')).replace('<body>',`<body data-test-build="${name==='broken'?'c':name}">`);
  if(name!=='a') {
   const css=html.match(/href="([^" ]+\.css)"/)[1];const replacement=`update-${name==='b'?'babe0001':'feed0002'}.css`;
   await rename(path.join(root,name,new URL(css,url).pathname.slice(1)),path.join(root,name,replacement));html=html.replaceAll(css,'/'+replacement);
  }
  await writeFile(file,html);
  await generateOfflineBundle(path.join(root,name));
 }
 for(const width of [320,800]) for(const theme of ['light','dark']) {
  active='a';const context=await browser.newContext({locale:'en-US',viewport:{width,height:width===320?844:1280},colorScheme:theme});
  const page=await pageFor(context);await boot(page);
  // Make real notes/colors/history and configure saved controls before the upgrade.
  const fixture=await read(page);const empty=fixture.board.findIndex(v=>v===0);
  await page.locator(`[data-row="${Math.floor(empty/9)}"][data-col="${empty%9}"]`).click();
  await page.getByRole('button',{name:'Green',exact:true}).click();await page.locator('.number-choices > button').filter({hasText:/^1$/}).click();
  await page.locator('.entry-actions').getByRole('button',{name:'Undo',exact:true}).click();
  if(width===800 && theme==='dark')await page.getByRole('button',{name:'Pause game',exact:true}).click();
  const checks=await page.evaluate(()=>updateChecks);await foreground(page);await page.evaluate(()=>window.dispatchEvent(new Event('online')));assert.equal(await page.evaluate(()=>updateChecks),checks);
  active='b';await waiting(page);assert.equal(await page.evaluate(()=>document.body.dataset.testBuild),'a');
  await page.screenshot({path:path.join(screenshots,`available-${theme}-${width}.png`)});
  if(width===320) {
   await page.goto(url+'/config');await mounted(page);await page.locator('#update-now').waitFor();
   for(const tag of ['es-AR','gn-PY','pt-BR','en-US']) {
    await page.getByRole('button',{name:languages[tag],exact:true}).click();
    assert.equal(await page.locator('#update-now').textContent(),catalogs[tag].UpdateNow);
    assert.equal(await page.locator('#update-later').textContent(),catalogs[tag].UpdateLater);
    assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
   }
   await page.goto(url);await mounted(page);await page.locator('#update-now').waitFor();
  }

  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
  await page.locator('#update-later').click();assert.equal(await page.locator('#update-now').count(),0);await foreground(page);await page.locator('#update-now').waitFor();
  // A denied per-tab save must retain the old version and restore gameplay.
  await page.evaluate(()=>{const native=Storage.prototype.setItem;Storage.prototype.setItem=function(k,v){if(k==='sudoku_update_resume'){Storage.prototype.setItem=native;throw new DOMException('test denied','SecurityError');}return native.call(this,k,v);};});
  await page.locator('#update-now').click();await page.getByRole('alert').filter({hasText:'could not be saved'}).waitFor();assert.equal(await page.evaluate(()=>document.body.dataset.testBuild),'a');
  // Observe exact snapshot contents before the worker can commit.
  const blocker=await pageFor(context);await blocker.goto(url);await mounted(blocker);
  await page.locator('#update-now').click();await page.waitForFunction(()=>sessionStorage.sudoku_update_resume);const saved=await snapshot(page);
  await page.locator('#update-waiting').waitFor({state:'visible'});await page.screenshot({path:path.join(screenshots,`waiting-${theme}-${width}.png`)});await page.locator('#update-cancel').click();assert.equal(await page.locator('#update-waiting').isVisible(),false);assert.equal(await page.evaluate(()=>sessionStorage.getItem('sudoku_update_resume')),null);
  await page.locator('#update-now').click();await page.waitForFunction(()=>sessionStorage.sudoku_update_resume);const final=await snapshot(page);assert.deepEqual(stable(final),stable(saved));
  // Once the complete update is downloaded it can be applied offline.
  let reloads=0;page.on('framenavigated',frame=>{if(frame===page.mainFrame())reloads++;});await blocker.close();await context.setOffline(true);await foreground(page);await upgraded(page);
  assert.deepEqual(stable(await read(page)),stable(final));assert.ok((await read(page)).timer_seconds>=final.timer_seconds);assert.equal(reloads,1);await page.waitForTimeout(150);assert.equal(reloads,1);
  for(const route of ['/config','/help','/history','/']){await page.goto(url+route);await mounted(page);assert.equal(await page.evaluate(()=>document.body.dataset.testBuild),'b');}
  await context.close();console.log(`Update ${theme} ${width}px: consent, Later, failed save, cancel, offline apply and reload passed`);
 }
 // An initial visit remains uncontrolled until navigation, but can still update.
 active='a';const freshContext=await browser.newContext({locale:'en-US'});const fresh=await pageFor(freshContext);await fresh.goto(url);await mounted(fresh);await fresh.evaluate(()=>navigator.serviceWorker.ready);assert.equal(await fresh.evaluate(()=>navigator.serviceWorker.controller),null);
 active='b';await waiting(fresh);await fresh.locator('#update-now').click();await upgraded(fresh);await freshContext.close();console.log('Initial uncontrolled visit updates without a reload loop');
 // Distinct per-tab progress must survive coordinated reloads.
 active='a';const context=await browser.newContext({locale:'en-US',viewport:{width:390,height:844}});const first=await pageFor(context);await boot(first);
 const seed=await read(first);const index=seed.board.findIndex(v=>v===0);const cell=`[data-row="${Math.floor(index/9)}"][data-col="${index%9}"]`;
 await first.locator(cell).click();await first.getByRole('button',{name:'Green',exact:true}).click();await first.locator('.number-choices > button').filter({hasText:/^1$/}).click();
 const second=await pageFor(context);await second.goto(url);await mounted(second);if(await second.locator(cell).getAttribute('data-selected')!=='true')await second.locator(cell).click();await second.getByRole('button',{name:'Purple',exact:true}).click();await second.locator('.number-choices > button').filter({hasText:/^2$/}).click();
 active='b';await waiting(first);await waiting(second);await second.locator('#update-later').click();await first.locator('#update-now').click();await first.waitForFunction(()=>sessionStorage.sudoku_update_resume);const one=await snapshot(first);
 assert.equal(await first.evaluate(()=>document.body.dataset.testBuild),'a');await foreground(second);await second.locator('#update-now').click();
 await Promise.all([upgraded(first),upgraded(second)]);
 assert.equal(await first.locator(cell).getAttribute('data-color'),String(one.cell_colors[index]));
 assert.notEqual(await first.locator(cell).textContent(),await second.locator(cell).textContent());
 await context.close();console.log('Multiple tabs: independent progress and all-tab consent passed');

 // Updates wait for a pending game replacement before taking their snapshot.
 active='a';const pendingContext=await browser.newContext({locale:'en-US'});const pending=await pageFor(pendingContext);await boot(pending);const previous=await read(pending);active='b';await waiting(pending);
 await pending.evaluate(()=>{const native=__sudokuHistory.save;__sudokuHistory.save=(...args)=>{__sudokuHistory.save=native;return new Promise((resolve,reject)=>{window.releaseHistory=()=>native(...args).then(resolve,reject);});};});
 await pending.getByRole('button',{name:'Actions',exact:true}).click();await pending.getByRole('button',{name:'New game',exact:true}).click();await pending.locator('#difficulty-choices').getByRole('button',{name:'Easy',exact:true}).click();
 await pending.waitForFunction(()=>typeof releaseHistory==='function');await pending.evaluate(()=>document.getElementById('update-now').click());
 await pending.getByRole('heading',{name:'Saving your game…',exact:true}).waitFor();assert.equal(await pending.evaluate(()=>sessionStorage.getItem('sudoku_update_resume')),null);
 await pending.evaluate(()=>releaseHistory());await upgraded(pending);assert.notEqual((await read(pending)).game_id,previous.game_id);await pendingContext.close();console.log('Pending history replacement completed before activation');
 // The wrapper must not register or offer browser updates.
 active='a';const nativeContext=await browser.newContext();await nativeContext.addInitScript(()=>window.__SUDOKU_NATIVE__=true);const nativePage=await pageFor(nativeContext);await nativePage.goto(url);await mounted(nativePage);assert.equal(await nativePage.evaluate(async()=>(await navigator.serviceWorker.getRegistrations()).length),0);assert.equal(await nativePage.locator('#update-now').count(),0);await nativeContext.close();

 // Superseding a waiting bundle invalidates a save still awaiting IndexedDB.
 active='a';const supersedeContext=await browser.newContext({locale:'en-US'});const supersede=await pageFor(supersedeContext);await boot(supersede);active='b';await waiting(supersede);
 await supersede.evaluate(()=>{const native=__sudokuHistory.save;__sudokuHistory.save=(...args)=>{__sudokuHistory.save=native;return new Promise((resolve,reject)=>{window.releaseSave=()=>native(...args).then(resolve,reject);});};});
 await supersede.locator('#update-now').click();await supersede.waitForFunction(()=>typeof releaseSave==='function');active='broken';
 await supersede.evaluate(async()=>{await (await navigator.serviceWorker.getRegistration()).update();});await supersede.locator('#update-now').waitFor();
 await supersede.evaluate(()=>releaseSave());await supersede.waitForTimeout(100);assert.equal(await supersede.evaluate(()=>sessionStorage.getItem('sudoku_update_resume')),null);
 await supersede.locator('#update-now').click();await supersede.waitForFunction(()=>document.body.dataset.testBuild==='c');await mounted(supersede);await supersedeContext.close();console.log('Superseded pending save discarded safely');
 // A failed bundle installation leaves the previous bundle intact.
 active='a';const failureContext=await browser.newContext({locale:'en-US'});const page=await pageFor(failureContext);await boot(page);active='broken';
 const css=(await readFile(path.join(root,'broken','index.html'),'utf8')).match(/href="([^" ]+\.css)"/)[1];failAsset=new URL(css,url).pathname;
 await page.evaluate(async()=>{const r=await navigator.serviceWorker.getRegistration();await r.update().catch(()=>{});});await page.waitForTimeout(1000);assert.equal(await page.locator('#update-now').count(),0);
 failAsset=null;await failureContext.setOffline(true);await page.reload();await mounted(page);assert.equal(await page.evaluate(()=>document.body.dataset.testBuild),'a');await failureContext.close();
 assert.deepEqual(errors,[]);console.log('Failed installation rollback passed');
} finally {await browser.close();await new Promise(r=>server.close(r));await rm(root,{recursive:true,force:true});}
