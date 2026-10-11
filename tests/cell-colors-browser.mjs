import { clickGameAction, openGameActions } from './game-actions.mjs';
// Production painting, saved-state compatibility and viewport-height regression.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const output = process.env.COLOR_SCREENSHOTS || '/tmp/sudoku-cell-colors';
await mkdir(output, {recursive:true});
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
  for(const theme of ['light','dark']) for(const [width,height] of [[320,844],[384,700],[384,824],[412,915],[800,1280],[960,1280]]) {
    const context=await browser.newContext({viewport:{width,height},colorScheme:theme});
    const page=await context.newPage(),errors=[];
    page.on('pageerror',e=>errors.push(e.message));
    const ready=async()=>{ await page.locator('.sudoku-cell').first().waitFor(); await page.locator('#loading').waitFor({state:'detached'}); };
    const read=()=>page.evaluate(()=>JSON.parse(localStorage.getItem('sudoku_state')));
    const cell=i=>page.locator(`[data-row="${Math.floor(i/9)}"][data-col="${i%9}"]`);
    const button=name=>page.getByRole('button',{name,exact:true});
    const fit=async()=>{
      const bounds=await page.evaluate(()=>({height:innerHeight,width:innerWidth,docHeight:document.documentElement.scrollHeight,docWidth:document.documentElement.scrollWidth,
        controls:document.querySelector('.control-panel').getBoundingClientRect().toJSON(),board:document.querySelector('.sudoku-board').getBoundingClientRect().toJSON()}));
      assert.ok(bounds.docHeight<=bounds.height+1,`${theme} ${width}×${height}: vertical overflow ${JSON.stringify(bounds)}`);
      assert.ok(bounds.docWidth<=bounds.width, 'horizontal overflow');
      assert.ok(bounds.controls.bottom<=bounds.height+1,'controls fit');
      assert.ok(bounds.controls.top>=bounds.board.bottom-1,'controls below board');
      assert.ok(Math.abs(bounds.board.width-bounds.board.height)<2,'square board');
      for(const {r,minimum} of await page.locator('.color-tool,.ui-action,.ui-icon-button').evaluateAll(es=>es.map(e=>({r:e.getBoundingClientRect().toJSON(),minimum:e.closest('.game-header')?36:44})))) {
        assert.ok(r.height>=minimum && r.width>=minimum, `${minimum}px action target ${r.width}×${r.height}`);
      }
      const buttonHeights=await page.locator('.number-choice,.color-tool').evaluateAll(es=>es.map(e=>e.getBoundingClientRect().height));
      assert.equal(new Set(buttonHeights).size,1,'numbers and colors have the same height');
      assert.ok(buttonHeights[0]>=(width>=768?48:44));
      if(width===384 && height===700) assert.ok(bounds.board.width>=350,'short phone board is larger than the previous 258px layout');
      if(width===960) assert.ok(bounds.board.width>=800,'11-inch tablet board grows beyond the previous 720px cap');
      const palette=await page.locator('.color-tools').boundingBox();
      const numbers=await page.locator('.number-choices').boundingBox();
      assert.ok(palette.y>=numbers.y+numbers.height,'palette below numbers');
    };
    await page.goto(url);await ready();
    const s=await read();
    s.solution=Array.from({length:81},(_,i)=>(Math.floor(i/9)*3+Math.floor(i/27)+i%9)%9+1);
    s.board=Array(81).fill(0); for(const i of [2,10,20,30,40,50,60,70,80]) s.board[i]=s.solution[i];
    s.givens=[...s.board]; s.hinted=Array(81).fill(0); s.hinted[10]=s.board[10];
    s.notes=Array(81).fill(0); s.notes[0]=5; s.board[1]=9;
    s.error_count=1;s.timer_seconds=123;s.paused=false;s.won=false;s.selected=null;s.note_mode=false;
    s.domino_enabled=false;s.sound_type='None';
    delete s.cell_colors;
    s.history=[{board:[...s.board],notes:[...s.notes]}];s.redo_stack=[{board:[...s.board],notes:[...s.notes]}];
    await page.evaluate(s=>localStorage.setItem('sudoku_state',JSON.stringify(s)),s);
    await page.reload();await ready();
    const legacy=await read();
    assert.deepEqual(legacy.board,s.board); assert.deepEqual(legacy.notes,s.notes);
    assert.deepEqual(legacy.cell_colors,Array(81).fill(0));
    assert.deepEqual(legacy.history[0].cell_colors,Array(81).fill(0));
    assert.deepEqual(legacy.redo_stack[0].cell_colors,Array(81).fill(0));
    await page.evaluate(()=>{
      window.__sudoku={_installPrompt:{},showInstall:()=>{}};
      window.dispatchEvent(new Event('sudoku:installavailable'));
    });
    await button('Instalar aplicativo').waitFor();await fit();
    await button('Instalar aplicativo').click();
    await fit();
    await button('Verde').click(); await cell(0).click(); await cell(2).click(); await cell(50).click();
    await button('Roxo').click();await cell(10).click();await cell(60).click();
    await button('Rosa').click();await cell(1).click();await cell(70).click();
    await button('Laranja').click();await cell(80).click();await cell(20).click();
    const marked=await read();
    assert.deepEqual(marked.board,s.board);assert.deepEqual(marked.notes,s.notes);assert.equal(marked.error_count,1);
    assert.equal(marked.cell_colors[2],1); assert.equal(marked.cell_colors[10],2);
    assert.equal(await cell(10).getAttribute('data-cell-state'),'hint');
    assert.equal(await cell(1).getAttribute('data-cell-state'),'error');
    assert.match(await cell(10).getAttribute('aria-label'),/cor roxo/);
    assert.equal(await page.locator('.cell-color-marker').count(),0,'no vertical cell color markers');
    assert.equal(await button('Remover cor').count(),0,'no eraser color tool');
    assert.equal(await cell(10).evaluate(e=>getComputedStyle(e).backgroundColor),await button('Roxo').locator('.color-swatch').evaluate(e=>getComputedStyle(e).backgroundColor));
    await cell(20).click();assert.equal((await read()).cell_colors[20],0);
    assert.equal((await read()).history.length,marked.history.length+1);
    await clickGameAction(page,'Desfazer');assert.equal((await read()).cell_colors[20],4);
    await clickGameAction(page,'Refazer');assert.equal((await read()).cell_colors[20],0);
    await clickGameAction(page,'Desfazer');
    await button('Verde').click();await cell(2).click();assert.equal((await read()).cell_colors[2],0);
    await clickGameAction(page,'Desfazer');assert.equal((await read()).cell_colors[2],1);
    await button('Limpar cores').click();assert.deepEqual((await read()).cell_colors,Array(81).fill(0));
    await clickGameAction(page,'Desfazer');assert.deepEqual((await read()).cell_colors,marked.cell_colors);
    await button('Pausar jogo').click();assert.equal(await button('Verde').isDisabled(),true);
    await cell(0).click();assert.deepEqual((await read()).cell_colors,marked.cell_colors);
    await button('Retomar jogo').click();await button('Laranja').click();await fit();
    await openGameActions(page);
    await page.screenshot({path:path.join(output,`actions-${theme}-${width}x${height}.png`)});
    await page.keyboard.press('Escape');
    assert.equal(await button('Ações').getAttribute('aria-expanded'),'false');
    assert.equal(await button('Ações').evaluate(e=>e===document.activeElement),true);
    await openGameActions(page);
    await page.locator('.actions-dismiss-layer').click({position:{x:2,y:2}});
    assert.equal(await button('Ações').getAttribute('aria-expanded'),'false');
    await page.screenshot({path:path.join(output,`painting-${theme}-${width}x${height}.png`)});
    await page.reload();await ready();assert.deepEqual((await read()).cell_colors,marked.cell_colors);
    assert.equal(await button('Laranja').getAttribute('aria-pressed'),'false');
    await button('Drop').click();await page.getByRole('group',{name:'Números',exact:true}).getByRole('button',{name:'3',exact:true}).click();await fit();
    await button('Verde').click();assert.equal(await button('Drop').getAttribute('aria-pressed'),'false');
    await button('Notas').click();assert.equal(await button('Verde').getAttribute('aria-pressed'),'false');
    await button('Notas').click();
    await page.getByRole('link',{name:'Configurações',exact:true}).click();
    await button('Restaurar padrões').click();assert.deepEqual((await read()).cell_colors,marked.cell_colors);
    await page.getByRole('link',{name:'Voltar',exact:true}).click();
    await button('Verde').click();await fit();
    await page.setViewportSize({width,height:Math.max(700,height-60)});await fit();await page.setViewportSize({width,height});
    // Successful install controls a reload and retains colors without network.
    await page.evaluate(()=>navigator.serviceWorker.ready);
    await page.reload();await ready();
    await context.setOffline(true);await page.reload();await ready();
    assert.deepEqual((await read()).cell_colors,marked.cell_colors);
    await context.setOffline(false);
    await clickGameAction(page,'Novo jogo');await button('Fácil').click();
    assert.deepEqual((await read()).cell_colors,Array(81).fill(0));
    assert.equal(await button('Limpar cores').isDisabled(),true);
    // Larger text may scroll, but all controls remain reachable.
    await page.addStyleTag({content:'.game-header h1 {font-size:40px} .ui-action {font-size:24px;min-height:64px} .number-choice {min-height:76px}'});
    const enlarged = await page.evaluate(()=>({width:innerWidth,scroll:document.documentElement.scrollWidth,
      escaped:[...document.querySelectorAll('button,main,header,h1,p,nav')].filter(e=>e.getBoundingClientRect().right>innerWidth).map(e=>({text:e.textContent,rect:e.getBoundingClientRect().toJSON()}))}));
    assert.ok(enlarged.scroll<=enlarged.width, `${theme} ${width}×${height}: enlarged text ${JSON.stringify(enlarged)}`);
    await openGameActions(page);
    await button('Novo jogo').scrollIntoViewIfNeeded();assert.equal(await button('Novo jogo').isVisible(),true);
    await page.keyboard.press('Escape');
    assert.deepEqual(errors,[]);
    console.log(`PASS ${theme} ${width}×${height}: painting, legacy saves, fit, reload, offline`);
    await context.close();
  }
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
