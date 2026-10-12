// Optional production browser regression for Sudoku sharing.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const output = process.env.SHARING_SCREENSHOTS || 'docs/screenshots/sharing';
const expectedOrigin = process.env.SHARING_EXPECTED_ORIGIN || 'https://free-sudoku-spa.vercel.app';
await mkdir(output, { recursive: true });
const mime = {'.html':'text/html','.css':'text/css','.js':'application/javascript','.wasm':'application/wasm','.json':'application/json','.png':'image/png'};
const server = createServer(async (req,res) => {
  const pathname = new URL(req.url, 'http://localhost').pathname;
  const name = ['/','/config','/help'].includes(pathname) ? 'index.html' : pathname.slice(1);
  try { const data = await readFile(path.join('dist',name)); res.writeHead(200, {'Content-Type':mime[path.extname(name)] || 'application/octet-stream'}); res.end(data); }
  catch {res.writeHead(404);res.end();}
});
await new Promise(resolve => server.listen(0,'127.0.0.1',resolve));
const url = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch({headless:true, executablePath:process.env.CHROMIUM_PATH || undefined, args:['--no-sandbox']});
try {
 for (const theme of ['light','dark']) for (const width of [390,800]) {
  console.log(`Checking ${theme} ${width}`);
  const context=await browser.newContext({locale: 'pt-BR', viewport:{width,height:width===390?844:1280},colorScheme:theme});
  const page=await context.newPage(); const errors=[]; page.on('pageerror',e=>errors.push(e.message));
  await page.goto(url);await page.locator('#loading').waitFor({state:'detached'});
  const read=()=>page.evaluate(()=>JSON.parse(localStorage.sudoku_state));
  for(const selector of ['meta[property="og:url"]','meta[property="og:image"]','meta[name="twitter:image"]']) assert.ok((await page.locator(selector).getAttribute('content')).startsWith(expectedOrigin+'/'));
  const fixture=await read(); fixture.paused=true;fixture.domino_enabled=false;fixture.sound_type='None';fixture.timer_seconds=123;
  const i=fixture.board.indexOf(0);fixture.board[i]=fixture.solution[i]%9+1;fixture.notes[i]=257;fixture.cell_colors[i]=3;fixture.error_count=7;
  const hint=fixture.board.findIndex((n,j)=>n===0 && j!==i);fixture.board[hint]=fixture.solution[hint];fixture.hinted[hint]=1;
  await page.evaluate(s=>localStorage.setItem('sudoku_state',JSON.stringify(s)),fixture);await page.reload();await page.locator('#loading').waitFor({state:'detached'});
  await page.evaluate(()=>{Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async()=>{throw Error('denied')}}});Object.defineProperty(navigator,'share',{configurable:true,value:async()=>{throw new DOMException('cancelled','AbortError')}});});
  await page.getByRole('button',{name:'Ações',exact:true}).click();await page.getByRole('button',{name:'Compartilhar',exact:true}).click();
  const dialog=page.locator('#share-dialog');await dialog.waitFor({state:'visible'});
  assert.equal(await page.getByRole('button',{name:'Gerar link',exact:true}).count(),0);
  const link=await page.locator('#share-url').inputValue();assert.ok(link.startsWith(expectedOrigin+'/#sudoku='));
  const payload=JSON.parse(Buffer.from(link.split('=')[1],'base64url').toString());
  assert.deepEqual(payload.board,fixture.board);assert.deepEqual(payload.notes,fixture.notes);assert.deepEqual(payload.colors,fixture.cell_colors);assert.deepEqual(payload.hinted,fixture.hinted);assert.equal(payload.seconds,123);
  assert.equal(payload.sound_type,undefined);assert.equal(payload.history,undefined);
  await page.screenshot({path:path.join(output,`share-${theme}-${width}.png`),fullPage:true});
  await page.evaluate(()=>Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async value=>{window.copiedSudoku=value}}}));
  await page.getByRole('button',{name:'Copiar link',exact:true}).click();await page.getByText('Link copiado.',{exact:true}).waitFor();assert.equal(await page.evaluate(()=>window.copiedSudoku),link);
  await page.evaluate(()=>Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async()=>{throw Error('denied')}}}));
  await page.getByRole('button',{name:'Copiar link',exact:true}).click();await page.getByText('Selecione e copie o link manualmente.').waitFor();
  await page.getByRole('button',{name:'Compartilhar link',exact:true}).click();
  await page.getByLabel('Sudoku inicial',{exact:true}).check();
  const initial=JSON.parse(Buffer.from((await page.locator('#share-url').inputValue()).split('=')[1],'base64url').toString());assert.deepEqual(initial.board,fixture.givens);assert.equal(initial.seconds,0);assert.ok(initial.notes.every(n=>n===0));
  await page.getByLabel('Estado atual',{exact:true}).check();assert.equal(await page.locator('#share-url').inputValue(),link);
  await page.keyboard.press('Escape');assert.equal(await dialog.isVisible(),false);assert.equal(await page.locator('#share-trigger').evaluate(e=>e===document.activeElement),true);
  await page.evaluate(hash=>location.hash=hash,link.split('#')[1]);await page.locator('#import-dialog').waitFor({state:'visible'});
  const importDialog=page.locator('#import-dialog');
  assert.equal(await importDialog.getByLabel('Estado atual',{exact:true}).isChecked(),true);
  await importDialog.getByLabel('Sudoku inicial',{exact:true}).check();
  assert.deepEqual((await page.locator('.share-preview-cell').allTextContents()).map(s=>s.trim()),fixture.givens.map(n=>n?String(n):''));
  assert.equal(await page.locator('.share-preview-cell[data-color="3"]').count(),0);
  await importDialog.getByText(/Tempo: 0min 0s · Erros: 0/).waitFor();
  assert.deepEqual((await read()).board,fixture.board);
  await importDialog.getByLabel('Estado atual',{exact:true}).check();
  assert.equal(await page.locator('.share-preview-cell[data-color="3"]').count(),1);
  await page.screenshot({path:path.join(output,`import-${theme}-${width}.png`),fullPage:true});
  assert.deepEqual((await read()).board,fixture.board);
  await page.getByRole('button',{name:'Cancelar',exact:true}).click();assert.equal(await page.evaluate(()=>location.hash),'');assert.deepEqual((await read()).board,fixture.board);
  await page.goto(url+'/#sudoku=!!!');await page.getByText('Link de Sudoku inválido.',{exact:true}).waitFor();assert.deepEqual((await read()).board,fixture.board);await page.keyboard.press('Escape');assert.equal(await page.evaluate(()=>location.hash),'');
  // Initial-load import, retaining a distinct recipient preference and wiping history.
  const recipient=await read();recipient.highlights.completed_contrast=37;recipient.board=[...recipient.givens];recipient.history=[{board:recipient.board,notes:recipient.notes,cell_colors:recipient.cell_colors}];
  await page.evaluate(s=>localStorage.setItem('sudoku_state',JSON.stringify(s)),recipient);
  await page.reload();await page.locator('#loading').waitFor({state:'detached'});
  await page.goto(url+'/#'+link.split('#')[1]);await page.reload();await page.getByRole('button',{name:'Carregar jogo',exact:true}).waitFor();await page.getByRole('button',{name:'Carregar jogo',exact:true}).click();
  await page.waitForFunction(()=>JSON.parse(localStorage.sudoku_state).history.length===0);
  const imported=await read();assert.deepEqual(imported.board,fixture.board);assert.deepEqual(imported.notes,fixture.notes);assert.deepEqual(imported.cell_colors,fixture.cell_colors);assert.equal(imported.highlights.completed_contrast,37);assert.equal(imported.paused,true);assert.equal(imported.selected,null);assert.equal(imported.note_mode,false);assert.equal(await page.evaluate(()=>location.hash),'');
  await page.evaluate(async()=>{await navigator.serviceWorker.ready});await page.reload();await page.locator('#loading').waitFor({state:'detached'});
  await context.setOffline(true);await page.reload();await page.locator('#loading').waitFor({state:'detached'});assert.deepEqual((await read()).board,fixture.board);assert.equal(await page.locator('#import-dialog').isVisible(),false);
  await page.getByRole('button',{name:'Retomar jogo',exact:true}).click();assert.equal((await read()).paused,false);
  await context.setOffline(false);
  const completed={...payload,board:payload.solution,seconds:456};
  await page.evaluate(hash=>location.hash=hash,'sudoku='+Buffer.from(JSON.stringify(completed)).toString('base64url'));
  await page.getByRole('button',{name:'Carregar jogo',exact:true}).click();
  await page.waitForFunction(()=>JSON.parse(localStorage.sudoku_state).won);
  assert.equal((await read()).paused,true);assert.equal(await page.locator('canvas').count(),0);
  await page.reload();await page.locator('#loading').waitFor({state:'detached'});assert.equal((await read()).won,true);
  await page.getByRole('button',{name:'Ações',exact:true}).click();await page.getByRole('button',{name:'Dica',exact:true}).click();assert.equal(await page.locator('canvas').count(),0);await page.keyboard.press('Escape');
  const invalids=[['a'.repeat(8193),'O link é grande demais.'],[Buffer.from(JSON.stringify({...payload,v:99})).toString('base64url'),'Versão de compartilhamento não suportada.'],[Buffer.from(JSON.stringify({...payload,solution:Array(81).fill(1)})).toString('base64url'),'O link contém um Sudoku inválido.']];
  for(const [encoded,message] of invalids) {
   await page.evaluate(hash=>location.hash=hash,'sudoku='+encoded);await page.getByText(message,{exact:true}).waitFor();
   assert.equal((await read()).won,true);await page.getByRole('button',{name:'Cancelar',exact:true}).click();
  }
  // Restart the very same completed snapshot from its original clues.
  await page.evaluate(hash=>location.hash=hash,'sudoku='+Buffer.from(JSON.stringify(completed)).toString('base64url'));
  assert.equal(await importDialog.getByLabel('Estado atual',{exact:true}).isChecked(),true);
  await importDialog.getByLabel('Sudoku inicial',{exact:true}).check();
  await page.screenshot({path:path.join(output,`import-initial-${theme}-${width}.png`),fullPage:true});
  assert.equal((await read()).won,true,'Preview must not replace the saved completed game');
  await page.getByRole('button',{name:'Carregar jogo',exact:true}).click();
  await page.waitForFunction(()=>!JSON.parse(localStorage.sudoku_state).won);
  const restarted=await read();
  assert.deepEqual(restarted.board,fixture.givens);
  for(const key of ['notes','cell_colors','hinted']) assert.ok(restarted[key].every(n=>n===0));
  assert.equal(restarted.timer_seconds,0);assert.equal(restarted.error_count,0);
  assert.equal(restarted.highlights.completed_contrast,37);assert.equal(restarted.paused,true);
  assert.equal(restarted.history.length,0);assert.equal(restarted.redo_stack.length,0);
  assert.equal(await page.evaluate(()=>location.hash),'');
  await page.reload();await page.locator('#loading').waitFor({state:'detached'});
  assert.deepEqual((await read()).board,fixture.givens);assert.equal((await read()).timer_seconds,0);
  assert.equal(await importDialog.isVisible(),false);
  assert.deepEqual(errors,[]);await context.close();
 }
 console.log('Sharing phone/tablet, both themes, fallback/cancellation/import/navigation/offline passed.');
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
