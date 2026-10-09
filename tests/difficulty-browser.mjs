// Optional release-WASM audit. Uses the same Playwright environment as theme-browser.mjs.
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const { chromium } = await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const mime={'.html':'text/html','.css':'text/css','.js':'application/javascript','.wasm':'application/wasm','.json':'application/json','.png':'image/png'};
const server=createServer(async(req,res)=>{
  const pathname=new URL(req.url,'http://localhost').pathname;
  const name=pathname==='/'?'index.html':pathname.slice(1);
  try {const bytes=await readFile(path.join('dist',name));res.writeHead(200,{'Content-Type':mime[path.extname(name)] || 'application/octet-stream'});res.end(bytes);}
  catch {res.writeHead(404);res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=await chromium.launch({headless:true,executablePath:process.env.CHROMIUM_PATH || undefined,args:['--no-sandbox']});
try {
  const context=await browser.newContext({viewport:{width:390,height:844},serviceWorkers:'block'});
  const page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`http://127.0.0.1:${server.address().port}`);
  await page.locator('.sudoku-cell').first().waitFor();
  await page.locator('#loading').waitFor({state:'detached'});
  console.log('AUDIT,requested,run,clues,handler_ms');
  for(const [requested,label] of [['Easy','Fácil'],['Medium','Médio'],['Hard','Difícil'],['Expert','Expert'],['Master','Mestre']]) {
    for(let run=0;run<20;run++) {
      await page.getByRole('button',{name:/Novo Jogo/}).click();
      await page.getByRole('button',{name:label,exact:true}).waitFor();
      const result=await page.evaluate(async label=>{
        const button=[...document.querySelectorAll('button')].find(b=>b.textContent.trim()===label);
        const start=performance.now();button.click();const elapsed=performance.now()-start;
        // Persistence effects flush after the synchronous click handler.
        await new Promise(resolve=>setTimeout(resolve,0));
        const s=JSON.parse(localStorage.getItem('sudoku_state'));
        return {elapsed,clues:s.givens.filter(v=>v!==0).length,difficulty:s.difficulty,board:s.board,givens:s.givens};
      },label);
      assert.equal(result.difficulty,requested);
      assert.deepEqual(result.board,result.givens);
      assert.equal(await page.locator('header p').innerText(),label);
      console.log(`AUDIT,${requested},${run},${result.clues},${result.elapsed.toFixed(3)}`);
    }
  }
  assert.deepEqual(errors,[]);
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
