// Optional production browser regression for completed-number contrast.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const output = '/tmp/sudoku-completed-contrast';
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
  for (const theme of ['light','dark']) for (const width of [320,800]) {
    const context = await browser.newContext({viewport:{width,height:width===320?844:1280},colorScheme:theme,serviceWorkers:'block'});
    const page = await context.newPage(), errors=[];
    page.on('pageerror',error=>errors.push(error.message));
    await page.goto(url);
    await page.locator('#loading').waitFor({state:'detached'});
    const read=()=>page.evaluate(()=>JSON.parse(localStorage.sudoku_state));
    const number=n=>page.getByRole('group',{name:'Números',exact:true}).getByRole('button',{name:String(n),exact:true});
    const fixture=await read();
    fixture.solution=Array.from({length:81},(_,i)=>(Math.floor(i/9)*3+Math.floor(i/27)+i%9)%9+1);
    fixture.board=fixture.solution.map(v=>v===1?1:0);
    fixture.givens=[...fixture.board]; fixture.hinted=Array(81).fill(0);
    fixture.notes=Array(81).fill(0);fixture.notes[1]=4;
    fixture.history=[{board:[...fixture.board],notes:[...fixture.notes]}]; fixture.redo_stack=[...fixture.history];
    fixture.domino_enabled=false;fixture.sound_type='None';fixture.selected=null;
    fixture.drop_mode=false;fixture.note_mode=false;fixture.paused=false;fixture.won=false;
    delete fixture.highlights.completed_contrast;
    await page.evaluate(s=>localStorage.setItem('sudoku_state',JSON.stringify(s)),fixture);
    await page.reload();await page.locator('#loading').waitFor({state:'detached'});
    const unchanged=async()=>{const save=await read();for(const key of ['board','notes','history','redo_stack'])assert.deepEqual(save[key],fixture[key]);};
    await unchanged();assert.equal((await read()).highlights.completed_contrast,100);
    assert.equal(await number(1).getAttribute('data-completed'),'true');
    assert.equal(await number(2).getAttribute('data-completed'),'false');
    assert.equal(await number(1).isDisabled(),true);
    assert.equal(await number(1).locator('.number-complete-marker').count(),1);
    const color=button=>button.evaluate(el=>{const canvas=document.createElement('canvas');canvas.width=canvas.height=1;const ctx=canvas.getContext('2d');ctx.fillStyle=getComputedStyle(el).backgroundColor;ctx.fillRect(0,0,1,1);return Array.from(ctx.getImageData(0,0,1,1).data).join(',');});
    const colors=[];
    for(const value of [0,50,100]) {
      await page.getByRole('link',{name:'Configurações',exact:true}).click();
      const slider=page.getByRole('slider',{name:'Contraste dos números concluídos',exact:true});
      await slider.fill(String(value));assert.equal(await slider.getAttribute('aria-valuetext'),`${value}%`);
      await page.reload();await slider.waitFor();assert.equal(await slider.inputValue(),String(value));
      await page.getByRole('link',{name:'Voltar',exact:true}).click();
      colors.push(await color(number(1)));
      await unchanged();
      if(value===0) assert.equal(colors[0],await color(number(2)));
    }
    assert.equal(new Set(colors).size,3);
    await page.screenshot({path:path.join(output,`keypad-${theme}-${width}.png`),fullPage:true});
    await page.getByRole('button',{name:'Drop',exact:true}).click();
    assert.equal(await number(1).isDisabled(),true);
    await page.getByRole('button',{name:'Notas',exact:true}).click();
    assert.equal(await number(1).isEnabled(),true);
    assert.equal(await number(1).getAttribute('data-completed'),'true');
    await number(1).click();assert.equal(await number(1).getAttribute('aria-pressed'),'true');
    await page.getByRole('button',{name:'Notas',exact:true}).click();assert.equal(await number(1).isDisabled(),true);
    await page.getByRole('link',{name:'Configurações',exact:true}).click();
    await page.getByRole('slider',{name:'Contraste dos números concluídos',exact:true}).fill('37');
    await page.getByRole('link',{name:'Voltar',exact:true}).click();
    await page.getByRole('button',{name:'Novo jogo',exact:true}).click();await page.getByRole('button',{name:'Fácil',exact:true}).click();
    assert.equal((await read()).highlights.completed_contrast,37);
    const newBoard=(await read()).board;
    await page.getByRole('link',{name:'Configurações',exact:true}).click();
    await page.getByRole('button',{name:'Restaurar padrões',exact:true}).click();
    assert.equal((await read()).highlights.completed_contrast,100);assert.deepEqual((await read()).board,newBoard);
    assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
    await page.screenshot({path:path.join(output,`settings-${theme}-${width}.png`),fullPage:true});
    assert.deepEqual(errors,[]);await context.close();
    console.log(`PASS completed contrast: ${theme}, ${width}px; legacy save, endpoints/midpoint, reload, notes/Drop, new game and reset`);
  }
} finally {await browser.close();await new Promise(r=>server.close(r));}
