// Production localization, compatibility, accessibility and offline audit.
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import { readCatalogs } from '../scripts/localization.mjs';
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const catalogs=await readCatalogs();
const names={'es-AR':'Español (Argentina)','gn-PY':'Avañe’ẽ (Paraguái)','pt-BR':'Português (Brasil)','en-US':'English (United States)'};
const flags={'es-AR':'🇦🇷','gn-PY':'🇵🇾','pt-BR':'🇧🇷','en-US':'🇺🇸'};
const screenshots=process.env.I18N_SCREENSHOTS || '/tmp/sudoku-localization';
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
const progress=game=>Object.fromEntries(['board','solution','notes','cell_colors','givens','hinted','history','redo_stack','game_id','difficulty','error_count'].map(key=>[key,game[key]]));
async function fit(page,label) {
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth <= innerWidth),true,`${label}: horizontal overflow`);
  const overflow=await page.locator('button').evaluateAll(elements=>elements.filter(element=>element.offsetWidth && element.scrollWidth>element.clientWidth+2).map(element=>element.textContent));
  assert.deepEqual(overflow,[],`${label}: button text overflow`);
}
try {
  const auditedLocales=process.env.I18N_LOCALES ? process.env.I18N_LOCALES.split(',') : Object.keys(catalogs);
  const widths=process.env.I18N_WIDTHS ? process.env.I18N_WIDTHS.split(',').map(Number) : [320,800];
  for(const locale of auditedLocales) for(const theme of ['light','dark']) for(const width of widths) {
    const context=await browser.newContext({locale,viewport:{width,height:width===320?844:1280},colorScheme:theme});
    const page=await context.newPage();const errors=[];page.on('pageerror',error=>{errors.push(error.message);console.error(error.message);});
    await page.goto(url);await mounted(page);
    assert.equal(await page.locator('html').getAttribute('lang'),locale);
    assert.equal(await page.title(),catalogs[locale].Title);
    assert.equal(await page.evaluate(()=>localStorage.getItem('sudoku_locale')),locale);
    const fixture=await read(page);
    const index=fixture.board.findIndex(value=>value===0);
    const cell=page.locator(`[data-row="${Math.floor(index/9)}"][data-col="${index%9}"]`);
    await cell.click();
    await page.getByRole('button',{name:String(fixture.solution[index]),exact:true}).click();
    await page.getByRole('button',{name:catalogs[locale].Green,exact:true}).click();
    const next=fixture.board.findIndex((value,i)=>value===0 && i!==index);
    await page.locator(`[data-row="${Math.floor(next/9)}"][data-col="${next%9}"]`).click();
    await page.getByRole('button',{name:catalogs[locale].Notes,exact:true}).click();
    await page.getByRole('button',{name:'1',exact:true}).click();
    await page.getByRole('button',{name:catalogs[locale].Notes,exact:true}).click();
    const before=progress(await read(page));
    assert.equal(before.board[index],fixture.solution[index]);
    assert.equal(before.cell_colors[index],1);
    assert.ok(before.notes[next]);
    assert.ok(before.history.length);
    assert.ok((await page.locator('.sudoku-cell').first().getAttribute('aria-label')).startsWith(catalogs[locale].CellLabel.split('{')[0]));
    await fit(page,`${locale}/${theme}/${width}/game`);
    await page.getByRole('button',{name:catalogs[locale].Actions,exact:true}).click();
    await page.getByRole('button',{name:catalogs[locale].NewGame,exact:true}).click();
    assert.equal(await page.locator('#difficulty-choices').getByRole('button',{name:catalogs[locale].Easy,exact:true}).count(),1);
    await fit(page,`${locale}/${theme}/${width}/actions`);
    await page.getByRole('button',{name:catalogs[locale].CloseActions,exact:true}).click();
    await page.getByRole('link',{name:catalogs[locale].Settings,exact:true}).click();
    await page.getByRole('heading',{name:catalogs[locale].Settings,exact:true}).waitFor();
    assert.equal(await page.locator('.language-choice').count(),4);
    for(const [tag,name] of Object.entries(names)) {
      const choice=page.getByRole('button',{name,exact:true});
      assert.equal(await choice.locator('[aria-hidden="true"]').textContent(),flags[tag]);
      assert.equal(await choice.getAttribute('lang'),tag);
      assert.equal(await choice.getAttribute('aria-pressed'),String(tag===locale));
      assert.ok((await choice.boundingBox()).height>=44);
    }
    for(const selected of ['en-US','gn-PY',locale]) {
      await page.getByRole('button',{name:names[selected],exact:true}).click();
      await page.getByRole('heading',{name:catalogs[selected].Settings,exact:true}).waitFor();
      assert.equal(await page.evaluate(()=>localStorage.getItem('sudoku_locale')),selected);
      assert.equal(await page.locator('html').getAttribute('lang'),selected);
    }
    await page.getByRole('button',{name:catalogs[locale].RestoreDefaults,exact:true}).click();
    assert.equal(await page.evaluate(()=>localStorage.getItem('sudoku_locale')),locale);
    assert.deepEqual(progress(await read(page)),before);
    await fit(page,`${locale}/${theme}/${width}/settings`);
    if(width===320) await page.screenshot({path:path.join(screenshots,`${locale}-${theme}-settings.png`),fullPage:true});
    await page.reload();await mounted(page);
    assert.equal(await page.getByRole('button',{name:names[locale],exact:true}).getAttribute('aria-pressed'),'true');
    for(const route of ['/help','/history']) {
      await page.goto(url+route);await mounted(page);
      const heading=route==='/help' ? catalogs[locale].HowPlay : catalogs[locale].History;
      await page.getByRole('heading',{name:heading,exact:true}).waitFor();
      await fit(page,`${locale}/${theme}/${width}${route}`);
      assert.ok(!(await page.locator('main').textContent()).includes('{number}'));
    }
    await page.goto(url);await mounted(page);
    await page.getByRole('button',{name:catalogs[locale].Actions,exact:true}).click();
    await page.getByRole('button',{name:catalogs[locale].Share,exact:true}).click();
    await page.locator('#share-dialog').waitFor({state:'visible'});
    await page.evaluate(()=>Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async()=>{throw Error('denied')}}}));
    await page.getByRole('button',{name:catalogs[locale].CopyLink,exact:true}).click();
    assert.equal(await page.locator('#share-status').textContent(),catalogs[locale].CopyManually);
    await page.getByRole('button',{name:catalogs[locale].Close,exact:true}).click();
    await page.goto(url+'/#sudoku=bad');await page.locator('#import-dialog').waitFor({state:'visible'});
    assert.equal(await page.locator('#import-dialog [role="alert"]').first().textContent(),catalogs[locale].InvalidLink);
    await page.getByRole('button',{name:catalogs[locale].Cancel,exact:true}).click();
    assert.deepEqual(progress(await read(page)),before);
    await page.evaluate(()=>navigator.serviceWorker.ready);await page.reload();await mounted(page);
    await context.setOffline(true);
    for(const route of ['/config','/help','/history','/']) {await page.goto(url+route);await mounted(page);assert.equal(await page.locator('html').getAttribute('lang'),locale);}
    await page.goto(url+'/config');await mounted(page);
    const alternate=locale==='en-US' ? 'es-AR' : 'en-US';
    await page.getByRole('button',{name:names[alternate],exact:true}).click();
    await page.reload();await mounted(page);
    assert.equal(await page.locator('html').getAttribute('lang'),alternate);
    assert.deepEqual(progress(await read(page)),before);
    assert.deepEqual(errors,[]);
    await context.close();
    console.log(`Passed ${locale}, ${theme}, ${width}px: language, flags, progress, routes, sharing, offline`);
  }
  for(const scenario of [{languages:undefined,language:undefined,expected:'en-US'}, {languages:['fr-FR'],language:'fr-FR',expected:'en-US'}, {languages:['fr-FR','es-MX','en-US'],language:'fr-FR',expected:'es-AR'}, {languages:['en-US'],language:'en-US',saved:'gn-PY',expected:'gn-PY'}]) {
    const context=await browser.newContext({viewport:{width:390,height:844}});
    await context.addInitScript(scenario=>{
      Object.defineProperty(navigator,'languages',{configurable:true,get:()=>scenario.languages});
      Object.defineProperty(navigator,'language',{configurable:true,get:()=>scenario.language});
      if(scenario.saved && !localStorage.getItem('sudoku_locale')) localStorage.setItem('sudoku_locale',scenario.saved);
    },scenario);
    const page=await context.newPage();await page.goto(url);await mounted(page);
    assert.equal(await page.locator('html').getAttribute('lang'),scenario.expected);
    assert.equal(await page.evaluate(()=>localStorage.getItem('sudoku_locale')),scenario.expected);
    await page.evaluate(()=>{Object.defineProperty(navigator,'languages',{configurable:true,get:()=>['pt-BR']});window.dispatchEvent(new Event('languagechange'));});
    assert.equal(await page.locator('html').getAttribute('lang'),scenario.expected);
    await page.reload();await mounted(page);
    assert.equal(await page.evaluate(()=>localStorage.getItem('sudoku_locale')),scenario.expected);
    await context.close();
  }
  const context=await browser.newContext({locale:'es-AR',viewport:{width:320,height:844}});
  await context.addInitScript(()=>{const set=Storage.prototype.setItem;Storage.prototype.setItem=function(key,value){if(key==='sudoku_locale')throw Error('denied');return set.call(this,key,value);};});
  const page=await context.newPage();await page.goto(url+'/config');await mounted(page);
  assert.equal(await page.locator('.locale-storage-error').textContent(),catalogs['es-AR'].LocaleSaveFailed);
  await page.getByRole('button',{name:names['gn-PY'],exact:true}).click();
  assert.equal(await page.locator('.locale-storage-error').textContent(),catalogs['gn-PY'].LocaleSaveFailed);
  assert.equal(await page.locator('html').getAttribute('lang'),'gn-PY');
  assert.notEqual(await page.locator('.locale-storage-error').evaluate(element=>getComputedStyle(element).backgroundColor),'rgba(0, 0, 0, 0)');
  await fit(page,'storage-denial');
  await context.close();
  console.log('Passed browser-preference fallback, saved override, reload and storage-denial scenarios');
} finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
