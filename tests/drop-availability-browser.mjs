// Production localization, compatibility, accessibility and offline audit.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import { readCatalogs } from '../scripts/localization.mjs';
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const catalogs=await readCatalogs();
const screenshots=process.env.DROP_AVAILABILITY_SCREENSHOTS || '/tmp/sudoku-drop-availability';
await mkdir(screenshots,{recursive:true});
const mime={'.html':'text/html','.js':'application/javascript','.wasm':'application/wasm','.css':'text/css','.json':'application/json','.png':'image/png'};
const server=createServer(async(req,res)=>{
  const pathname=new URL(req.url,'http://localhost').pathname;
  const file=['/','/config','/help','/history'].includes(pathname) ? 'index.html' : pathname.slice(1);
  try {res.writeHead(200,{'Content-Type':mime[path.extname(file)] || 'application/octet-stream'});res.end(await readFile(path.join('dist',file)));}
  catch {res.end();}
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const url=`http://127.0.0.1:${server.address().port}`;
const browser=await chromium.launch({headless:true,executablePath:process.env.CHROMIUM_PATH || undefined,args:['--no-sandbox']});
const mounted=async page=>{await page.locator('#loading').waitFor({state:'detached'});await page.locator('#history-saving').waitFor({state:'hidden'});};
const read=page=>page.evaluate(()=>JSON.parse(localStorage.getItem('sudoku_state')));
async function fit(page,label) {
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth <= innerWidth),true,`${label}: horizontal overflow`);
  const overflow=await page.locator('button').evaluateAll(elements=>elements.filter(element=>element.offsetWidth && element.scrollWidth>element.clientWidth+2).map(element=>element.textContent));
  assert.deepEqual(overflow,[],`${label}: button text overflow`);
}
try {
  for(const locale of Object.keys(catalogs)) for(const theme of ['light','dark']) for(const width of [320,800]) {
    const c=catalogs[locale];
    const context=await browser.newContext({locale,viewport:{width,height:width===320?844:1280},colorScheme:theme});
    const page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
    const button=name=>page.getByRole('button',{name,exact:true});
    const cell=i=>page.locator(`[data-row="${Math.floor(i/9)}"][data-col="${i%9}"]`);
    const config=async()=>{await page.goto(url+'/config');await mounted(page);};
    const back=async()=>{await page.getByRole('link',{name:c.Back,exact:true}).click();await mounted(page);};
    const choose=async label=>{await config();await page.getByRole('radio',{name:label,exact:true}).check();await back();};
    const widths=async()=>{
      const notes=await page.locator('.notes-action').boundingBox();
      for(const box of await page.locator('.entry-actions > button:not(.notes-action),.actions-trigger').evaluateAll(es=>es.map(e=>e.getBoundingClientRect().toJSON()))) {
        assert.ok(Math.abs(notes.width-1.5*box.width)<1,'Notes width ratio');
        assert.ok(Math.abs(notes.height-box.height)<1,'equal heights');
        assert.ok(box.width>=44 && box.height>=44,'touch target');
      }
      await fit(page,`${locale}/${theme}/${width}`);
    };
    await page.goto(url);await mounted(page);
    const old=await read(page);delete old.drop_availability;
    old.drop_mode=false;old.drop_number=null;old.note_mode=false;old.selected=null;old.domino_enabled=false;old.sound_type='None';
    await page.evaluate(s=>localStorage.setItem('sudoku_state',JSON.stringify(s)),old);await page.reload();await mounted(page);
    assert.equal((await read(page)).drop_availability,'Both');assert.deepEqual((await read(page)).board,old.board);
    await button(c.Drop).click();assert.equal((await read(page)).drop_mode,true);
    await button(c.Drop).click();assert.equal((await read(page)).drop_mode,false);
    await config();assert.equal(await page.getByRole('radio',{name:c.BothModes,exact:true}).isChecked(),true);await back();
    const target=old.board.findIndex(v=>!v);await cell(target).click();
    await choose(c.DropOnly);assert.equal(await button(c.Drop).count(),0);assert.equal((await read(page)).drop_mode,true);assert.equal((await read(page)).selected,null);await widths();
    await button(String(old.solution[target])).click();await cell(target).click();assert.equal((await read(page)).board[target],old.solution[target]);
    await page.reload();await mounted(page);assert.equal((await read(page)).drop_mode,true);assert.equal((await read(page)).drop_availability,'DropOnly');
    await button(c.Green).click();assert.equal((await read(page)).note_mode,true);
    await choose(c.NoDrop);assert.equal(await button(c.Drop).count(),0);assert.equal((await read(page)).drop_mode,false);assert.equal((await read(page)).drop_number,null);assert.equal((await read(page)).note_mode,true);await widths();
    const next=old.board.findIndex((v,i)=>!v && i!==target);
    const before=await read(page);await cell(next).click();await button('1').click();
    assert.deepEqual((await read(page)).board,before.board);assert.equal((await read(page)).notes[next],before.notes[next]^1);
    await cell(next).click();assert.equal((await read(page)).selected,null);
    await config();await page.locator('#undo-enabled').uncheck();
    await page.screenshot({path:path.join(screenshots,`${locale}-${theme}-${width}-settings.png`),fullPage:true,animations:'disabled'});await back();await widths();
    assert.equal(await page.locator('.entry-actions button').count(),2);
    const preserved=await read(page);
    await page.evaluate(()=>navigator.serviceWorker.ready);await page.reload();await mounted(page);
    await context.setOffline(true);await page.reload();await mounted(page);
    assert.equal((await read(page)).drop_availability,'NoDrop');assert.equal((await read(page)).drop_mode,false);
    for(const field of ['board','notes','cell_colors','history'])assert.deepEqual((await read(page))[field],preserved[field]);
    await context.setOffline(false);
    await button(c.Actions).click();await button(c.NewGame).click();const id=(await read(page)).game_id;
    await page.locator('#difficulty-choices').getByRole('button',{name:c.Easy,exact:true}).click();
    await page.waitForFunction(id=>JSON.parse(localStorage.sudoku_state).game_id!==id,id);await mounted(page);
    assert.equal((await read(page)).drop_availability,'NoDrop');assert.equal((await read(page)).drop_mode,false);assert.equal((await read(page)).note_mode,true);
    await config();const noDrop=page.getByRole('radio',{name:c.NoDrop,exact:true});await noDrop.focus();await page.keyboard.press('ArrowRight');
    assert.equal(await page.getByRole('radio',{name:c.BothModes,exact:true}).isChecked(),true);await back();assert.equal(await button(c.Drop).count(),1);await widths();
    await choose(c.DropOnly);await config();await button(c.RestoreDefaults).click();assert.equal(await page.getByRole('radio',{name:c.BothModes,exact:true}).isChecked(),true);await back();
    assert.equal((await read(page)).drop_mode,true);await button(c.Drop).click();assert.equal((await read(page)).drop_mode,false);await widths();
    assert.deepEqual(errors,[]);await context.close();console.log(`PASS Drop policies: ${locale}, ${theme}, ${width}px`);
  }
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
