import { clickGameAction } from './game-actions.mjs';
// Optional production-bundle audit; same Playwright environment as theme-browser.mjs.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const output = '/tmp/sudoku-domino';
await mkdir(output, { recursive: true });
const mime = { '.html':'text/html', '.css':'text/css', '.js':'application/javascript', '.wasm':'application/wasm', '.json':'application/json', '.png':'image/png' };
const server = createServer(async (req, res) => {
  const pathname = new URL(req.url, 'http://localhost').pathname;
  const name = ['/','/config','/help'].includes(pathname) ? 'index.html' : pathname.slice(1);
  try { const data = await readFile(path.join('dist', name)); res.writeHead(200, {'Content-Type':mime[path.extname(name)] || 'application/octet-stream'}); res.end(data); }
  catch { res.writeHead(404); res.end(); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const url = `http://127.0.0.1:${server.address().port}`;
let browser;
try {
  browser = await chromium.launch({ headless:true, executablePath:process.env.CHROMIUM_PATH || undefined, args:['--no-sandbox'] });
  for (const width of [390, 1280]) {
    const context = await browser.newContext({locale: 'pt-BR',  viewport:{width, height:width===390?844:900}, serviceWorkers:'block' });
    const page = await context.newPage();
    const errors = []; page.on('pageerror', e => errors.push(e.message));
    const read = () => page.evaluate(() => JSON.parse(localStorage.getItem('sudoku_state')));
    const cell = i => page.locator(`[data-row="${Math.floor(i/9)}"][data-col="${i%9}"]`);
    const number = n => page.locator('.number-choices').getByRole('button',{name:String(n),exact:true});
    async function ready() {
      await page.locator('#loading').waitFor({state:'detached'});
      await page.waitForFunction(() => !!localStorage.getItem('sudoku_state'));
    }
    async function seed({threshold=0, delay=600, legacy=false}={}) {
      await page.goto(url); await ready();
      await page.evaluate(({threshold,delay,legacy}) => {
        const s = JSON.parse(localStorage.getItem('sudoku_state'));
        s.solution = Array.from({length:81},(_,i)=>(Math.floor(i/9)*3+Math.floor(i/27)+i%9)%9+1);
        s.board = [...s.solution]; for(const i of [0,1,9]) s.board[i]=0;
        s.givens = [...s.board]; s.hinted=Array(81).fill(0); s.notes=Array(81).fill(0);
        s.history=[];s.redo_stack=[];s.won=false;s.paused=false;s.selected=null;s.note_mode=false;s.drop_mode=false;s.drop_number=null;s.sound_type='None';s.domino_enabled=true;
        s.domino={initial_delay_ms:delay, acceleration_percent:0, minimum_delay_ms:100, empty_cell_threshold:threshold};
        if(legacy) { delete s.domino; s.domino_gen=4294967295; }
        localStorage.setItem('sudoku_state',JSON.stringify(s));
      },{threshold,delay,legacy});
      await page.reload(); await ready();
    }
    async function newGame() {
      const previousGameId=(await read()).game_id;
      await clickGameAction(page,'Novo jogo');
      await page.getByRole('button',{name:'Fácil',exact:true}).click();
      await page.waitForFunction(id=>JSON.parse(localStorage.getItem('sudoku_state')).game_id!==id,previousGameId);
      await page.locator('#history-saving').waitFor({state:'hidden'});
    }
    await seed({legacy:true});
    assert.deepEqual((await read()).domino, {initial_delay_ms:600, acceleration_percent:20, minimum_delay_ms:100, empty_cell_threshold:10});
    assert.equal((await read()).board.filter(v=>v===0).length,3); // Reload does not start a chain.
    await page.goto(url+'/config'); await ready();
    await page.getByText('Ajustar velocidade',{exact:true}).click();
    for(const [id,value] of [['domino-initial','1000'],['domino-acceleration','10'],['domino-minimum','250'],['domino-threshold','2']]) {
      await page.locator('#'+id).fill(value);
      assert.equal(await page.locator('#'+id).inputValue(),value);
    }
    assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
    await page.screenshot({path:path.join(output,`settings-${width}.png`),fullPage:true});
    await page.reload(); await ready();
    assert.deepEqual((await read()).domino,{initial_delay_ms:1000, acceleration_percent:10, minimum_delay_ms:250, empty_cell_threshold:2});
    await page.goto(url); await ready();
    await newGame();
    assert.equal((await read()).domino.empty_cell_threshold,2);
    await page.goto(url+'/config'); await ready();
    await page.getByRole('button',{name:'Restaurar padrões',exact:true}).click();
    assert.equal((await read()).domino_enabled,true);
    assert.equal((await read()).domino.empty_cell_threshold,10);
    assert.deepEqual((await read()).highlights, {selected_shading:20, matching_shading:20, available_shading:100, completed_contrast:100, dots:true, stripes:true});

    for(const drop of [false,true]) {
      await seed({threshold:2});
      if(drop) {
        await page.getByRole('button',{name:'Drop',exact:true}).click();
        await number(1).click(); await cell(0).click();
      } else { await cell(0).click(); await number(1).click(); }
      await page.waitForTimeout(200);
      assert.equal((await read()).board[1],0); // Initial delay is respected.
      await page.waitForFunction(()=>JSON.parse(localStorage.getItem('sudoku_state')).board[1]===2);
      assert.equal((await read()).board[9],0); // One cell per timeout.
      await page.waitForFunction(()=>JSON.parse(localStorage.getItem('sudoku_state')).won);
      assert.equal((await read()).history.length,1);
      await clickGameAction(page,'Desfazer');
      assert.equal((await read()).board.filter(v=>v===0).length,3);
      assert.equal((await read()).won,false);
      await clickGameAction(page,'Refazer');
      assert.equal((await read()).won,true);
    }
    await seed({threshold:1,delay:200});
    await cell(0).click(); await number(1).click();
    await page.waitForTimeout(500);
    assert.equal((await read()).board[1],0); // Two empties exceeds threshold 1.
    for(const action of ['disable','pause','undo','edit','new-game','settings']) {
      await seed({delay:1500});
      await cell(0).click(); await number(1).click();
      if(action==='disable' || action==='settings') {
        // SPA navigation preserves the pending callback.
        await page.getByRole('link',{name:'Configurações'}).click();
        if(action==='disable') await page.locator('#domino-enabled').uncheck();
        else { await page.getByText('Ajustar velocidade',{exact:true}).click(); await page.locator('#domino-initial').fill('1000'); }
      } else if(action==='pause') await page.getByRole('button',{name:'Pausar jogo',exact:true}).click();
      else if(action==='undo') await clickGameAction(page,'Desfazer');
      else if(action==='edit') { await cell(1).click(); await page.getByRole('button',{name:'Notas',exact:true}).click(); await number(2).click(); }
      else {
        await newGame();
      }
      const before = (await read()).board;
      await page.waitForTimeout(1700);
      assert.deepEqual((await read()).board,before,`${width}: ${action} cancels pending fills`);
    }
    assert.deepEqual(errors,[]);
    await context.close();
    console.log(`Domino browser checks passed at ${width}px`);
  }
} finally {
  await browser?.close();
  await new Promise(resolve => server.close(resolve));
}
