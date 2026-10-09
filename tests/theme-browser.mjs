// Optional browser audit: provide Playwright via PLAYWRIGHT_MODULE or an installed playwright package.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const output = process.env.THEME_SCREENSHOTS || '/tmp/sudoku-themes';
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
  let sharedFixture;
  for (const theme of ['light','dark']) {
    const context = await browser.newContext({viewport:{width:390,height:844},colorScheme:theme,serviceWorkers:'block'});
    const page = await context.newPage();
    const errors=[]; page.on('pageerror',e=>errors.push(e.message));
    await page.goto(url); await page.locator('.sudoku-cell').first().waitFor();
    const fixture = await page.evaluate(() => {
      const s=JSON.parse(localStorage.getItem('sudoku_state'));
      const empty=s.board.map((v,i)=>v===0?i:-1).filter(i=>i>=0);
      const user=empty[0], hint=empty[1], notes=empty[2];
      s.board[user]=s.solution[user];s.board[hint]=s.solution[hint];s.hinted[hint]=s.solution[hint];
      s.notes[notes]=0b101010101;s.selected=[Math.floor(user/9),user%9];
      s.domino_enabled=false;s.auto_notes_enabled=false;s.sound_type='None';s.history=[];s.redo_stack=[];
      localStorage.setItem('sudoku_state',JSON.stringify(s));return {user,hint,notes,s};
    });
    if (sharedFixture) {
      fixture.s=sharedFixture.s;fixture.user=sharedFixture.user;fixture.hint=sharedFixture.hint;fixture.notes=sharedFixture.notes;
      await page.evaluate(s=>localStorage.setItem('sudoku_state',JSON.stringify(s)),fixture.s);
    } else sharedFixture=fixture;
    await page.reload(); await page.locator('[data-number-origin="user"]').waitFor();
    assert.equal(await page.locator('[data-cell-state="matching"]').count()>0,true);
    const cell = i=>page.locator(`[data-row="${Math.floor(i/9)}"][data-col="${i%9}"]`);
    assert.equal(await cell(fixture.hint).getAttribute('data-number-origin'),'hint');
    const selected = await cell(fixture.user).evaluate(e=>getComputedStyle(e).outlineStyle);
    assert.equal(selected,'solid');
    assert.equal(await cell(fixture.user).evaluate(e=>getComputedStyle(e).outlineWidth),'3px');
    // A same-box cell outside the selected row and column is also a peer.
    const row=Math.floor(fixture.user/9),col=fixture.user%9;
    const boxPeer=fixture.s.board.findIndex((v,i)=>v===0 && Math.floor(i/27)===Math.floor(row/3) && Math.floor((i%9)/3)===Math.floor(col/3) && Math.floor(i/9)!==row && i%9!==col);
    if(boxPeer>=0) assert.equal(await cell(boxPeer).getAttribute('data-cell-state'),'blocked');
    async function verifyPreview(board, digit) {
      const cells=await page.locator('.sudoku-cell').evaluateAll(es=>es.map(e=>({r:Number(e.dataset.row),c:Number(e.dataset.col),placement:e.dataset.placement,blocker:e.dataset.blocker,state:e.dataset.cellState,selected:e.dataset.selected==='true',label:e.getAttribute('aria-label')})));
      const selected=cells.find(e=>e.selected);
      for(const e of cells) {
        const index=e.r*9+e.c;
        const blocked=board.some((v,i)=>v===digit && (Math.floor(i/9)===e.r || i%9===e.c || (Math.floor(i/27)===Math.floor(e.r/3) && Math.floor((i%9)/3)===Math.floor(e.c/3))));
        const expected=board[index]!==0 ? 'none' : blocked ? 'blocked' : 'available';
        assert.equal(e.placement,expected,`${theme} digit ${digit} cell ${index}`);
        const primary=selected && board[selected.r*9+selected.c]===digit && (selected.r===e.r || selected.c===e.c || (Math.floor(selected.r/3)===Math.floor(e.r/3) && Math.floor(selected.c/3)===Math.floor(e.c/3)));
        const source=expected==='blocked' ? primary ? 'selected' : 'matching' : 'none';
        assert.equal(e.blocker,source,`${theme} blocker source ${index}`);
        if(expected==='blocked' && !e.selected) assert.equal(e.state,source==='matching'?'matching-blocked':'blocked');
        if(expected!=='none') assert.match(e.label,new RegExp(`${expected==='available'?'disponível':'bloqueada'} para ${digit}`));
      }
    }
    await verifyPreview(fixture.s.board,fixture.s.board[fixture.user]);
    await page.locator('#loading').waitFor({state:'detached'});
    await page.screenshot({path:path.join(output,`${theme}-mobile.png`),fullPage:true});
    await cell(fixture.user).click();await page.getByRole('button',{name:'⌫ Apagar',exact:true}).click();
    assert.equal(await cell(fixture.user).getAttribute('data-number-origin'),'empty');
    await page.getByRole('button',{name:'↩ Desfazer',exact:true}).click();
    assert.equal(await cell(fixture.user).getAttribute('data-number-origin'),'user');
    // A selected error keeps its error fill and selection outline.
    const wrong=fixture.s.solution[fixture.user]%9+1;
    await page.locator('.grid-cols-9 > button').filter({hasText:new RegExp(`^${wrong}$`)}).click();
    assert.equal(await cell(fixture.user).getAttribute('data-cell-state'),'error');
    assert.equal(await cell(fixture.user).getAttribute('data-selected'),'true');
    assert.equal(await cell(fixture.user).locator('.cell-error-marker').innerText(),'!');
    assert.equal(await cell(fixture.user).evaluate(e=>getComputedStyle(e).outlineStyle),'solid');
    await cell(fixture.hint).click();
    assert.equal(await cell(fixture.hint).getAttribute('data-cell-state'),'hint');
    assert.equal(await cell(fixture.hint).getAttribute('data-selected'),'true');
    for(const width of [390,1280]) {
      await page.setViewportSize({width,height:width===390?844:900});
      for(const route of ['/','/help','/config']) {
        await page.goto(url+route);await page.waitForFunction(()=>!!window.wasmBindings);
        assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`${theme} ${width} ${route} overflow`);
      }
    }
    await page.goto(url);await page.locator('.sudoku-cell').first().waitFor();
    await page.locator('#loading').waitFor({state:'detached'});
    await page.screenshot({path:path.join(output,`${theme}-desktop.png`),fullPage:true});
    // Drop preview uses the keypad digit rather than the selected cell's value.
    await page.getByRole('button',{name:'🎯 Drop',exact:true}).click();
    assert.equal(await page.locator('[data-placement="available"], [data-placement="blocked"]').count(),0);
    for(const digit of [1,7]) {
      await page.locator('.grid-cols-9 > button').filter({hasText:new RegExp(`^${digit}$`)}).click();
      const board=await page.evaluate(()=>JSON.parse(localStorage.getItem('sudoku_state')).board);
      await verifyPreview(board,digit);
    }
    await page.locator('.grid-cols-9 > button').filter({hasText:/^7$/}).click();
    assert.equal(await page.locator('[data-placement="available"], [data-placement="blocked"]').count(),0);
    await page.getByRole('button',{name:'🎯 Drop ON',exact:true}).click();
    // Load a real legacy save whose history predates a correct player entry.
    await page.evaluate(({s,user})=>{
      const legacy=structuredClone(s);delete legacy.givens;
      const previous=[...legacy.board];previous[user]=0;
      legacy.history=[{board:previous,notes:[...legacy.notes]}];legacy.redo_stack=[];
      localStorage.setItem('sudoku_state',JSON.stringify(legacy));
    },fixture);
    await page.reload();await page.locator('.sudoku-cell').first().waitFor();
    assert.equal(await cell(fixture.user).getAttribute('data-number-origin'),'given');
    await page.getByRole('button',{name:'↩ Desfazer',exact:true}).click();
    assert.equal(await cell(fixture.user).getAttribute('data-number-origin'),'empty');
    await cell(fixture.user).click();
    await page.locator('.grid-cols-9 > button').filter({hasText:new RegExp(`^${fixture.s.solution[fixture.user]}$`)}).click();
    assert.equal(await cell(fixture.user).getAttribute('data-number-origin'),'user');
    assert.deepEqual(errors,[]);
    await context.close();
    console.log(`PASS ${theme}: mobile/desktop routes, highlights, editable correct entry, undo, error/hint overlaps, legacy history unlock; screenshots ${output}`);
  }
} finally { await browser.close();await new Promise(resolve=>server.close(resolve)); }
