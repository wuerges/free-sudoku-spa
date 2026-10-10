// Optional production browser regression for solved-cell Drop selection.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const output = '/tmp/sudoku-drop-selection';
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
  for(const theme of ['light','dark']) for(const width of [320,800]) {
    const context=await browser.newContext({viewport:{width,height:width===320?844:1280},colorScheme:theme,serviceWorkers:'block'});
    const page=await context.newPage(),errors=[];
    page.on('pageerror',e=>errors.push(e.message));
    await page.goto(url);await page.locator('.sudoku-cell').first().waitFor();
    await page.locator('#loading').waitFor({state:'detached'});
    const read=()=>page.evaluate(()=>JSON.parse(localStorage.getItem('sudoku_state')));
    const cell=i=>page.locator(`[data-row="${Math.floor(i/9)}"][data-col="${i%9}"]`);
    const number=n=>page.getByRole('group',{name:'Números',exact:true}).getByRole('button',{name:String(n),exact:true});
    const s=await read();
    s.solution=Array.from({length:81},(_,i)=>(Math.floor(i/9)*3+Math.floor(i/27)+i%9)%9+1);
    s.board=Array(81).fill(0);for(const i of [2,3,4,5]) s.board[i]=s.solution[i];s.board[6]=8;
    s.givens=Array(81).fill(0);s.givens[2]=3;s.givens[3]=4;
    s.hinted=Array(81).fill(0);s.hinted[4]=5;s.notes=Array(81).fill(0);
    s.history=[{board:[...s.board],notes:[...s.notes]}];s.redo_stack=[...s.history];
    s.error_count=2;s.timer_seconds=123;s.selected=null;s.paused=false;s.won=false;
    s.domino_enabled=false;s.sound_type='None';s.note_mode=false;
    delete s.drop_pick_solved; // Previous-version save keeps progress and gains the default.
    async function seed() {
      await page.evaluate(s=>localStorage.setItem('sudoku_state',JSON.stringify(s)),s);
      await page.reload();await cell(2).waitFor();await page.locator('#loading').waitFor({state:'detached'});
    }
    const noOverflow=async()=>assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
    await seed();assert.equal((await read()).drop_pick_solved,true);
    assert.deepEqual((await read()).board,s.board);assert.equal((await read()).history.length,1);
    await cell(2).click();assert.equal(await page.getByRole('button',{name:'Drop',exact:true}).getAttribute('aria-pressed'),'false');
    await page.getByRole('button',{name:'Drop',exact:true}).click();
    await number(9).click();
    const before=await read();
    for(const i of [2,2,4,5]) {
      await cell(i).click();
      assert.equal(await number(s.board[i]).getAttribute('aria-pressed'),'true');
      assert.match(await page.getByRole('status').innerText(),new RegExp(`Drop: número ${s.board[i]}`));
      const after=await read();
      for(const field of ['board','notes','history','redo_stack','error_count']) assert.deepEqual(after[field],before[field]);
    }
    await number(9).click();await cell(6).click();
    assert.equal((await read()).board[6],9); // Incorrect cells remain editable.
    await seed();await page.getByRole('button',{name:'Drop',exact:true}).click();
    await page.getByRole('button',{name:'Notas',exact:true}).click();
    await cell(2).click();await cell(0).click();
    assert.equal((await read()).notes[0],1<<2);assert.equal((await read()).board[0],0);
    await seed();await page.getByRole('link',{name:'Configurações',exact:true}).click();
    const pref=page.getByRole('checkbox',{name:'Selecionar número pelas células resolvidas',exact:true});
    assert.equal(await pref.isChecked(),true);await pref.uncheck();await noOverflow();
    await page.reload();await pref.waitFor();assert.equal(await pref.isChecked(),false);
    assert.deepEqual((await read()).board,s.board);
    await page.getByRole('link',{name:'Voltar',exact:true}).click();
    await page.getByRole('button',{name:'Drop',exact:true}).click();await number(9).click();await cell(5).click();
    assert.equal((await read()).board[5],9);assert.equal(await number(9).getAttribute('aria-pressed'),'true');
    await page.getByRole('button',{name:'Novo jogo',exact:true}).click();await page.getByRole('button',{name:'Fácil',exact:true}).click();
    assert.equal((await read()).drop_pick_solved,false);
    const newBoard=(await read()).board;
    await page.getByRole('link',{name:'Configurações',exact:true}).click();
    await page.getByRole('button',{name:'Restaurar padrões',exact:true}).click();
    assert.equal(await pref.isChecked(),true);assert.deepEqual((await read()).board,newBoard);
    await noOverflow();await page.screenshot({path:path.join(output,`settings-${theme}-${width}.png`),fullPage:true});
    await page.getByRole('link',{name:'Voltar',exact:true}).click();await noOverflow();
    // A completed digit is blocked for placement, even when picked from the board.
    s.board=s.solution.map(v=>v===1?1:0);s.givens=[...s.board];s.notes=Array(81).fill(0);
    s.hinted=Array(81).fill(0);s.history=[];s.redo_stack=[];s.note_mode=false;
    await seed();await page.getByRole('button',{name:'Drop',exact:true}).click();
    assert.equal(await number(1).isDisabled(),true);
    await cell(0).click();await cell(1).click();
    assert.equal((await read()).board[1],0);assert.equal((await read()).history.length,0);
    await page.getByRole('button',{name:'Notas',exact:true}).click();
    assert.equal(await number(1).isEnabled(),true);
    // Keypad can deselect/reselect a completed digit in notes mode.
    await number(1).click();await number(1).click();await cell(1).click();
    assert.equal((await read()).notes[1],1);await cell(1).click();assert.equal((await read()).notes[1],0);
    await page.getByRole('button',{name:'Notas',exact:true}).click();
    assert.equal(await number(1).isDisabled(),true);await cell(1).click();assert.equal((await read()).board[1],0);
    s.board[0]=0;s.givens[0]=0;s.board[1]=1; // Nine visible ones, only eight correct.
    await seed();await page.getByRole('button',{name:'Drop',exact:true}).click();
    assert.equal(await number(1).isEnabled(),true);assert.equal(await number(1).getAttribute('data-completed'),'false');await number(1).click();await cell(0).click();
    assert.equal((await read()).board[0],1);assert.equal(await number(1).isDisabled(),true);assert.equal(await number(1).getAttribute('data-completed'),'true');
    assert.deepEqual(errors,[]);await context.close();
    console.log(`PASS solved-cell Drop selection: ${theme}, ${width}px; legacy save, picking, notes, disabled behavior, reload, new game, reset and completed-number placement/notes`);
  }
} finally {await browser.close();await new Promise(r=>server.close(r));}
