// Production UI audit and PR screenshots. Optional Playwright environment matches theme-browser.mjs.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const screenshots = process.env.UI_SCREENSHOTS || 'docs/screenshots/ui-ux';
await mkdir(screenshots, {recursive:true});
const mime = {'.html':'text/html','.css':'text/css','.js':'application/javascript','.wasm':'application/wasm','.json':'application/json','.png':'image/png'};
const server = createServer(async(req,res) => {
  const pathname = new URL(req.url,'http://localhost').pathname;
  const name = ['/','/config','/help'].includes(pathname) ? 'index.html' : pathname.slice(1);
  try { const data = await readFile(path.join('dist',name)); res.writeHead(200,{'Content-Type':mime[path.extname(name)] || 'application/octet-stream'}); res.end(data); }
  catch {res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url = `http://127.0.0.1:${server.address().port}`;
assert.equal(JSON.parse(await readFile('dist/manifest.json','utf8')).orientation,'portrait');
const browser = await chromium.launch({headless:true,executablePath:process.env.CHROMIUM_PATH || undefined,args:['--no-sandbox']});
try {
  let sharedFixture;
  for(const theme of ['light','dark']) {
    for(const {width,height} of [{width:320,height:844},{width:390,height:844},{width:768,height:900},{width:800,height:1280},{width:1024,height:768},{width:1280,height:800},{width:1280,height:900}]) {
      const context = await browser.newContext({viewport:{width,height},colorScheme:theme,serviceWorkers:'block'});
      const page = await context.newPage(); const errors=[];
      page.on('pageerror',e=>errors.push(e.message));
      const read = ()=>page.evaluate(()=>JSON.parse(localStorage.getItem('sudoku_state')));
      const cell = i=>page.locator(`[data-row="${Math.floor(i/9)}"][data-col="${i%9}"]`);
      async function overflow(route) {
        assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`${theme} ${width}: ${route} overflow`);
        // Content must fit its section, not merely the document.
        const escaped=await page.locator('.settings-section > *, .entry-actions > *, .header-tools > *').evaluateAll(es=>es.filter(e=>{
          const r=e.getBoundingClientRect(),p=e.parentElement.getBoundingClientRect();
          return r.left<p.left-1 || r.right>p.right+1;
        }).map(e=>e.textContent));
        assert.deepEqual(escaped,[]);
      }
      async function ready() { await page.locator('#loading').waitFor({state:'detached'}); }
      await page.goto(url); await page.locator('.sudoku-cell').first().waitFor(); await ready();
      if(sharedFixture) {
        await page.evaluate(s=>localStorage.setItem('sudoku_state',JSON.stringify(s)),sharedFixture);
        await page.reload(); await page.locator('.sudoku-cell').first().waitFor(); await ready();
      } else sharedFixture=await read();
      await overflow('game');
      const allTargets=await page.locator('.ui-action, .ui-icon-button, .number-choices > button').evaluateAll(es=>es.map(e=>({label:e.getAttribute('aria-label')||e.textContent.trim(),r:e.getBoundingClientRect().toJSON()})));
      for(const {label,r} of allTargets) {
        assert.ok(r.height>=44 && r.width>=24,`${width}: ${label} touch target ${r.width}×${r.height}`);
        if(/^\d$/.test(label)) assert.ok(r.width>=48 && r.height>=52,`${label}: number target`);
        else assert.ok(r.width>=44,`${label}: action width`);
      }
      const digits=await page.locator('.number-choices > button').evaluateAll(es=>es.map(e=>e.getBoundingClientRect().toJSON()));
      assert.equal(new Set(digits.map(r=>r.top)).size,width<640?2:1);
      const board=await page.locator('.sudoku-board').boundingBox();
      const controls=await page.locator('.control-panel').boundingBox();
      if(width>=768) {
        assert.ok(await page.locator('.cell-number').first().evaluate(e=>parseFloat(getComputedStyle(e).fontSize))>=30, 'tablet board digits scale up');
        assert.ok(await page.locator('.cell-note').first().evaluate(e=>parseFloat(getComputedStyle(e).fontSize))>10, 'tablet notes scale up');
        assert.ok(board.width>500,`${width}×${height}: board should grow beyond phone cap`);
        assert.ok(board.y+board.height<=height,`${width}×${height}: board fits vertically`);
        for(const r of digits) assert.ok(r.width>=48 && r.height>=64);
        assert.ok(controls.y>=board.y+board.height, 'controls stay below board');
        const cellSize=await cell(0).boundingBox();
        assert.ok(cellSize.width>=56 && Math.abs(cellSize.width-cellSize.height)<1,'tablet cells are large squares');
      }
      // Regular browser tabs may resize despite the installed PWA portrait lock.
      if(width===800 || (width===1280 && height===800)) {
        const savedBoard=(await read()).board;
        await page.setViewportSize({width:height,height:width});
        await overflow('rotated game');
        assert.deepEqual((await read()).board,savedBoard);
        const rotatedBoard=await page.locator('.sudoku-board').boundingBox();
        const rotatedControls=await page.locator('.control-panel').boundingBox();
        assert.ok(rotatedControls.y>=rotatedBoard.y+rotatedBoard.height);
        await page.setViewportSize({width,height});
      }
      for(let i=0;i<digits.length;i++) for(let j=i+1;j<digits.length;j++) {
        const a=digits[i], b=digits[j];
        assert.ok(a.right<=b.left || b.right<=a.left || a.bottom<=b.top || b.bottom<=a.top, `${width}: number buttons overlap`);
      }
      assert.equal(await page.locator('.ui-icon:not([aria-hidden="true"])').count(),0);
      const settingsLink=page.getByRole('link',{name:'Configurações',exact:true});
      await settingsLink.focus();
      assert.equal(await settingsLink.evaluate(e=>getComputedStyle(e).outlineStyle),'solid');
      await page.keyboard.press('Tab');
      assert.equal(await page.getByRole('link',{name:'Ajuda',exact:true}).evaluate(e=>e===document.activeElement),true);
      const given=sharedFixture.board.findIndex(v=>v!==0);
      await cell(given).click();
      if(width===390 || width===800 || (width===1280 && height===900)) await page.screenshot({path:path.join(screenshots,`game-${theme}-${width}.png`),fullPage:true});
      await page.getByRole('button',{name:'Pausar jogo',exact:true}).click();
      assert.equal((await read()).paused,true);
      await page.getByRole('button',{name:'Retomar jogo',exact:true}).click();
      assert.equal((await read()).paused,false);
      await page.getByRole('button',{name:'Notas',exact:true}).click();
      assert.equal(await page.getByRole('button',{name:'Notas',exact:true}).getAttribute('aria-pressed'),'true');
      await page.getByRole('button',{name:'Notas',exact:true}).click();
      await page.getByRole('button',{name:'Drop',exact:true}).click();
      await page.getByRole('group',{name:'Números',exact:true}).getByRole('button',{name:'1',exact:true}).click();
      assert.equal(await page.getByRole('button',{name:'Drop',exact:true}).getAttribute('aria-pressed'),'true');
      assert.match(await page.getByRole('status').innerText(),/Drop: número 1/);
      await page.getByRole('button',{name:'Drop',exact:true}).click();
      await page.getByRole('button',{name:'Novo jogo',exact:true}).click();
      await overflow('difficulty');
      await page.getByRole('button',{name:'Fácil',exact:true}).click();
      assert.equal((await read()).difficulty,'Easy');
      await settingsLink.click();
      await page.getByRole('heading',{name:'Configurações',exact:true}).waitFor();
      await overflow('settings');
      assert.deepEqual(await page.locator('.settings-section > h2').allTextContents(),['Destaques do tabuleiro','Assistências','Som']);
      assert.equal(await page.locator('.timing-details').getAttribute('open'),null);
      assert.equal(await page.locator('#domino-initial').isVisible(),false);
      assert.equal(await page.locator('#domino-threshold').inputValue(),'10');
      assert.equal(await page.locator('#domino-enabled').isChecked(),true);
      if(width===390 || (width===1280 && height===900)) await page.screenshot({path:path.join(screenshots,`settings-${theme}-${width}.png`),fullPage:true});
      for(const id of ['selected-shading','matching-shading','available-shading']) {
        assert.equal(await page.locator('#'+id).inputValue(),id==='available-shading'?'100':'20');
      }
      for(const choice of ['Desligado','Bip','Explosão']) {
        await page.getByRole('radio',{name:choice,exact:true}).check();
        assert.equal((await read()).sound_type,{Desligado:'None',Bip:'Beep','Explosão':'Explosion'}[choice]);
      }
      const bip=page.getByRole('radio',{name:'Bip',exact:true});
      await bip.check(); await bip.focus(); await page.keyboard.press('ArrowRight');
      assert.equal(await page.getByRole('radio',{name:'Explosão',exact:true}).isChecked(),true);
      // Clicking a full preference row activates its associated checkbox.
      await page.locator('label[for="undo-enabled"] .preference-label').click();
      assert.equal(await page.locator('#undo-enabled').isChecked(),false);
      await page.locator('#domino-enabled').uncheck();
      const summary=page.getByText('Ajustar velocidade',{exact:true});
      await summary.focus(); await page.keyboard.press('Enter');
      assert.equal(await page.locator('#domino-initial').isVisible(),true);
      assert.equal(await page.locator('#domino-initial').isEnabled(),true);
      await page.locator('#domino-initial').fill('1000');
      await page.locator('#domino-minimum').fill('800');
      await page.locator('#domino-initial').fill('200');
      assert.equal(await page.locator('#domino-minimum').inputValue(),'200');
      await page.locator('#domino-acceleration').fill('0');
      await page.locator('#domino-threshold').fill('0');
      await page.locator('#selected-shading').fill('35');
      await page.locator('#selected-shading').focus(); await page.keyboard.press('ArrowRight');
      assert.equal(await page.locator('#selected-shading').inputValue(),'36');
      await page.getByRole('radio',{name:'Desligado',exact:true}).check();
      const beforeReload=await read();
      await page.reload(); await page.getByRole('heading',{name:'Configurações',exact:true}).waitFor(); await ready();
      const afterReload=await read();
      assert.deepEqual(afterReload.domino,beforeReload.domino);
      assert.deepEqual(afterReload.highlights,beforeReload.highlights);
      assert.equal(afterReload.sound_type,'None');
      assert.equal(afterReload.undo_enabled,false);
      assert.equal(afterReload.domino_enabled,false);
      assert.equal(await page.locator('.timing-details').getAttribute('open'),null);
      await page.getByRole('link',{name:'Voltar',exact:true}).click();
      assert.equal(await page.getByRole('button',{name:'Desfazer',exact:true}).count(),0);
      await page.getByRole('link',{name:'Configurações',exact:true}).click();
      await page.getByRole('button',{name:'Restaurar padrões',exact:true}).click();
      const reset=await read();
      assert.deepEqual(reset.board,beforeReload.board);
      assert.deepEqual(reset.highlights,{selected_shading:20,matching_shading:20,available_shading:100,dots:true,stripes:true});
      assert.equal(reset.sound_type,'Explosion'); assert.equal(reset.domino_enabled,true);
      assert.equal(reset.domino.empty_cell_threshold,10);
      await page.getByRole('link',{name:'Voltar',exact:true}).click();
      assert.equal(await page.getByRole('button',{name:'Desfazer',exact:true}).count(),1);
      await page.evaluate(()=>{
        window.__sudoku={_installPrompt:{},showInstall:()=>window.__installTest=true};
        window.dispatchEvent(new Event('sudoku:installavailable'));
      });
      await page.getByRole('button',{name:'Instalar aplicativo',exact:true}).waitFor();
      await overflow('install');
      const install=await page.locator('.install-row').boundingBox(),header=await page.locator('.header-main').boundingBox();
      assert.ok(install.y>=header.y+header.height);
      await page.getByRole('button',{name:'Instalar aplicativo',exact:true}).click();
      assert.equal(await page.evaluate(()=>window.__installTest),true);
      await page.getByRole('link',{name:'Ajuda',exact:true}).click();
      await page.getByRole('heading',{name:'Como jogar',exact:true}).waitFor(); await overflow('help');
      assert.deepEqual(errors,[]);
      await context.close();
      console.log(`UI audit passed: ${theme}, ${width}×${height}px`);
    }
  }
} finally { await browser.close(); await new Promise(resolve=>server.close(resolve)); }
