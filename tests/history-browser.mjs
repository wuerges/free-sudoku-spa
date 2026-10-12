// Optional production history/IndexedDB audit and review captures.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE||'playwright');
const output=process.env.HISTORY_SCREENSHOTS||'docs/screenshots/history';await mkdir(output,{recursive:true});
const mime={'.html':'text/html','.js':'application/javascript','.css':'text/css','.wasm':'application/wasm','.json':'application/json','.png':'image/png'};
const server=createServer(async(req,res)=>{const pathname=new URL(req.url,'http://localhost').pathname;const name=['/','/history','/config','/help'].includes(pathname)?'index.html':pathname.slice(1);try{const data=await readFile(path.join('dist',name));res.writeHead(200,{'Content-Type':mime[path.extname(name)]||'application/octet-stream'});res.end(data);}catch{res.writeHead(404);res.end();}});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const url=`http://127.0.0.1:${server.address().port}`;
const browser=await chromium.launch({headless:true,executablePath:process.env.CHROMIUM_PATH||undefined,args:['--no-sandbox']});
try {
 for(const theme of ['light','dark']) for(const width of [390,800]) {
  console.log(`History ${theme} ${width}`);
  const context=await browser.newContext({viewport:{width,height:width===390?844:1280},colorScheme:theme});
  const page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(url);await page.locator('#loading').waitFor({state:'detached'});await page.locator('#history-saving').waitFor({state:'hidden'});
  const read=()=>page.evaluate(()=>JSON.parse(localStorage.sudoku_state));
  const records=()=>page.evaluate(async()=>JSON.parse(await window.__sudokuHistory.list()));
  const saved=await read();delete saved.game_id;saved.paused=true;saved.domino_enabled=false;saved.sound_type='None';saved.highlights.completed_contrast=37;saved.timer_seconds=321;saved.error_count=7;
  const empty=saved.board.indexOf(0);saved.board[empty]=saved.solution[empty]%9+1;saved.cell_colors[empty]=3;saved.notes[empty]=257;
  saved.history=[{board:[...saved.givens],notes:Array(81).fill(0),cell_colors:Array(81).fill(0)}];saved.redo_stack=[...saved.history];
  await page.evaluate(s=>localStorage.setItem('sudoku_state',JSON.stringify(s)),saved);await page.reload();await page.locator('#loading').waitFor({state:'detached'});await page.locator('#history-saving').waitFor({state:'hidden'});
  const original=await read();assert.ok(original.game_id);for(const key of ['board','notes','cell_colors','history','redo_stack'])assert.deepEqual(original[key],saved[key]);
  const openHistory=async()=>{if(new URL(page.url()).pathname!='/history'){await page.getByRole('button',{name:'Ações',exact:true}).click();await page.getByRole('link',{name:'Histórico',exact:true}).click();}await page.getByRole('heading',{name:'Histórico',exact:true}).waitFor();await page.locator('#history-saving').waitFor({state:'hidden'});};
  const back=async()=>{await page.getByRole('link',{name:'Voltar',exact:true}).click();};
  const newGame=async()=>{const id=(await read()).game_id;await page.getByRole('button',{name:'Ações',exact:true}).click();await page.getByRole('button',{name:'Novo jogo',exact:true}).click();await page.getByRole('group',{name:'Dificuldade do novo jogo'}).getByRole('button',{name:'Fácil',exact:true}).click();await page.waitForFunction(id=>JSON.parse(localStorage.sudoku_state).game_id!==id,id);await page.locator('#history-saving').waitFor({state:'hidden'});};
  const card=id=>page.locator(`article[data-game-id="${id}"]`);
  await openHistory();await card(original.game_id).getByRole('button',{name:'Fixar',exact:true}).click();await page.locator('#history-saving').waitFor({state:'hidden'});assert.ok((await records()).find(e=>e.id===original.game_id).pinned);
  await back();for(let i=0;i<12;i++)await newGame();
  let rows=await records(),active=(await read()).game_id;
  assert.equal(rows.filter(e=>e.id!==active&&!e.pinned).length,10);assert.ok(rows.find(e=>e.id===original.game_id).pinned);
  await openHistory();assert.equal(await page.locator('section[aria-label="Jogos recentes"] article').count(),10);assert.equal(await page.locator('section[aria-label="Jogos fixados"] article').count(),1);
  assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
  assert.ok(await page.locator('.history-card').evaluateAll(cards=>cards.every(card=>card.querySelector('.history-miniature').getBoundingClientRect().right<=card.querySelector('.history-card-info').getBoundingClientRect().left+1)), 'Miniatures must not cover card controls');
  await page.screenshot({path:path.join(output,`history-${theme}-${width}.png`),fullPage:true});
  // Restarting the oldest unpinned game at capacity must retain that attempt.
  const oldest=rows.filter(e=>e.id!==active&&!e.pinned).sort((a,b)=>a.order-b.order)[0];
  await card(oldest.id).getByRole('button',{name:'Abrir jogo',exact:true}).click();await page.locator('#history-load').getByLabel('Sudoku inicial',{exact:true}).check();await page.locator('#history-load').getByRole('button',{name:'Carregar jogo',exact:true}).click();await page.waitForURL(url+'/');await page.locator('#history-saving').waitFor({state:'hidden'});
  assert.deepEqual((await records()).find(e=>e.id===oldest.id).game.board,oldest.game.board);assert.notEqual((await read()).game_id,oldest.id);
  await openHistory();
  // Resume restores the same identity and undo/redo, while preserving current settings.
  await card(original.game_id).getByRole('button',{name:'Abrir jogo',exact:true}).click();
  await page.locator('#history-load').getByRole('button',{name:'Carregar jogo',exact:true}).click();await page.waitForURL(url+'/');await page.locator('#history-saving').waitFor({state:'hidden'});
  let current=await read();assert.equal(current.game_id,original.game_id);assert.deepEqual(current.board,original.board);assert.deepEqual(current.history,original.history);assert.deepEqual(current.redo_stack,original.redo_stack);assert.equal(current.highlights.completed_contrast,37);assert.equal(current.paused,true);assert.equal(current.selected,null);
  // Confirmed link imports archive outgoing progress; previews/cancellation do not replace it.
  const payload={v:1,board:original.givens,givens:original.givens,solution:original.solution,notes:Array(81).fill(0),colors:Array(81).fill(0),hinted:Array(81).fill(0),difficulty:original.difficulty,requested:original.requested_difficulty,rating:original.rating,seed:original.seed,seconds:0,errors:0};
  const hash='sudoku='+Buffer.from(JSON.stringify(payload)).toString('base64url');
  await page.evaluate(hash=>location.hash=hash,hash);await page.locator('#import-dialog').waitFor({state:'visible'});await page.getByRole('button',{name:'Cancelar',exact:true}).click();assert.equal((await read()).game_id,original.game_id);
  await page.evaluate(hash=>location.hash=hash,hash);await page.getByRole('button',{name:'Carregar jogo',exact:true}).click();await page.waitForFunction(id=>JSON.parse(localStorage.sudoku_state).game_id!==id,original.game_id);await page.locator('#history-saving').waitFor({state:'hidden'});
  assert.deepEqual((await records()).find(e=>e.id===original.game_id).game.board,original.board);
  // Restart a pinned saved attempt creates another identity and preserves the source.
  await openHistory();await card(original.game_id).getByRole('button',{name:'Abrir jogo',exact:true}).click();await page.locator('#history-load').getByLabel('Sudoku inicial',{exact:true}).check();
  await page.screenshot({path:path.join(output,`restart-${theme}-${width}.png`),fullPage:true});
  await page.locator('#history-load').getByRole('button',{name:'Carregar jogo',exact:true}).click();await page.waitForURL(url+'/');await page.locator('#history-saving').waitFor({state:'hidden'});
  current=await read();assert.notEqual(current.game_id,original.game_id);assert.deepEqual(current.board,original.givens);assert.equal(current.timer_seconds,0);assert.equal(current.error_count,0);assert.equal(current.history.length,0);assert.equal(current.highlights.completed_contrast,37);assert.ok((await records()).find(e=>e.id===original.game_id).pinned);
  // Archive failure keeps active progress and the pending shared link.
  await page.evaluate(()=>{window.actualHistorySave=window.__sudokuHistory.save;window.__sudokuHistory.save=()=>Promise.reject(Error('quota'));});
  await page.evaluate(hash=>location.hash=hash,hash);await page.getByRole('button',{name:'Carregar jogo',exact:true}).click();await page.locator('#import-dialog').getByText(/Não foi possível salvar no histórico/).waitFor();assert.equal((await read()).game_id,current.game_id);assert.ok(await page.evaluate(()=>location.hash));assert.equal(await page.locator('#import-dialog').isVisible(),true);
  await page.getByRole('button',{name:'Cancelar',exact:true}).click();await page.evaluate(()=>{window.__sudokuHistory.save=window.actualHistorySave;});await page.getByRole('button',{name:'Fechar aviso',exact:true}).click();
  // Primary-save failure also blocks replacement after the archive succeeds.
  await page.evaluate(id=>{window.originalStorageSet=Storage.prototype.setItem;Storage.prototype.setItem=function(key,value){if(key==='sudoku_state' && JSON.parse(value).game_id!==id)throw new DOMException('quota','QuotaExceededError');return window.originalStorageSet.call(this,key,value);};},current.game_id);
  await page.evaluate(hash=>location.hash=hash,hash);await page.getByRole('button',{name:'Carregar jogo',exact:true}).click();await page.locator('#import-dialog').getByText(/Não foi possível salvar o jogo/).waitFor();assert.equal((await read()).game_id,current.game_id);
  await page.getByRole('button',{name:'Cancelar',exact:true}).click();await page.evaluate(()=>{Storage.prototype.setItem=window.originalStorageSet;});await page.getByRole('button',{name:'Fechar aviso',exact:true}).click();
  // Cleanup failure keeps extras and retry restores the ten-game window without losing pins.
  await page.evaluate(()=>{window.actualHistoryPrune=window.__sudokuHistory.prune;window.__sudokuHistory.prune=()=>Promise.reject(Error('cleanup'));});
  await newGame();active=(await read()).game_id;assert.equal((await records()).filter(e=>e.id!==active&&!e.pinned).length,11);
  await page.getByText(/Não foi possível limitar o histórico/).waitFor();
  await page.evaluate(()=>{window.__sudokuHistory.prune=window.actualHistoryPrune;});await page.getByRole('button',{name:'Tentar novamente',exact:true}).click();await page.locator('#history-saving').waitFor({state:'hidden'});
  active=(await read()).game_id;assert.equal((await records()).filter(e=>e.id!==active&&!e.pinned).length,10);assert.ok((await records()).find(e=>e.id===original.game_id).pinned);
  // Older pinned entry is removed only after explicitly confirming an unpin.
  await openHistory();for(let i=0;i<12;i++){await back();await newGame();await openHistory();}
  page.once('dialog',dialog=>dialog.dismiss());await card(original.game_id).getByRole('button',{name:'Desafixar',exact:true}).click();assert.ok((await records()).find(e=>e.id===original.game_id).pinned);
  page.once('dialog',dialog=>dialog.accept());await card(original.game_id).getByRole('button',{name:'Desafixar',exact:true}).click();await page.locator('#history-saving').waitFor({state:'hidden'});assert.equal((await records()).some(e=>e.id===original.game_id),false);
  await page.evaluate(async()=>{await navigator.serviceWorker.ready});await page.reload();await page.locator('#loading').waitFor({state:'detached'});await page.locator('#history-saving').waitFor({state:'hidden'});
  await context.setOffline(true);await page.reload();await page.locator('#loading').waitFor({state:'detached'});await page.locator('#history-saving').waitFor({state:'hidden'});assert.equal(await page.locator('section[aria-label="Jogos recentes"] article').count(),10);
  assert.deepEqual(errors,[]);await context.close();
 }
 console.log('History retention, pinning, resume/restart, link import, failures and offline reload passed.');
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
