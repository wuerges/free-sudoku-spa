// Browser instrumentation uses JavaScript to inspect the native Web Audio API.
import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const server=createServer(async(req,res)=>{
 const p=new URL(req.url,'http://localhost').pathname;
 const file=['/','/config'].includes(p)?'index.html':p.slice(1);
 try {const b=await readFile(path.join('dist',file)); res.setHeader('Content-Type',({'.wasm':'application/wasm','.js':'application/javascript','.css':'text/css','.html':'text/html'})[path.extname(file)]||'application/octet-stream');res.end(b);}catch{res.writeHead(404);res.end();}
});
await new Promise(r=>server.listen(0,'127.0.0.1',r));
const url=`http://127.0.0.1:${server.address().port}`;
const browser=await chromium.launch({args:['--no-sandbox']});
try {
 for(const width of [320,800]) for(const theme of ['light','dark']) {
  const context=await browser.newContext({viewport:{width,height:1280},locale:'en-US',colorScheme:theme,serviceWorkers:'block'});
  await context.addInitScript(()=>{
   window.audit={contexts:[],starts:[],stops:0};
   const Native=window.AudioContext;
   window.AudioContext=class extends Native {constructor(...args){super(...args);audit.contexts.push(this);}};
   const stop=AudioScheduledSourceNode.prototype.stop;
   AudioScheduledSourceNode.prototype.stop=function(...args){audit.stops++;return stop.apply(this,args);};
   const start=AudioBufferSourceNode.prototype.start;
   AudioBufferSourceNode.prototype.start=function(when){audit.starts.push({when,duration:this.buffer.duration});return start.call(this,when);};
  });
  const page=await context.newPage(); const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(url+'/config');await page.locator('#sound-volume').waitFor();
  assert.equal(await page.locator('#sound-volume').inputValue(),'75');
  assert.equal(await page.locator('#balloon-count').inputValue(),'25');
  await page.locator('#sound-preview').click();
  await page.waitForFunction(()=>audit.starts.length>0);
  assert.equal(await page.evaluate(()=>audit.contexts.length),1);
  await page.evaluate(()=>audit.contexts[0].suspend());
  await page.locator('#sound-preview').click();
  await page.waitForFunction(()=>audit.contexts[0].state==='running' && audit.starts.length>=2);
  await page.evaluate(()=>audit.contexts[0].close());
  await page.locator('#sound-preview').click();
  await page.waitForFunction(()=>audit.contexts.length===2 && audit.starts.length>=3);
  // A rejected resume must recover on the next trusted interaction.
  await page.evaluate(async()=>{const c=audit.contexts[1];await c.suspend();const native=c.resume.bind(c);c.resume=()=>{c.resume=native;return Promise.reject(new Error('test interruption'));};});
  await page.locator('#sound-preview').click();
  await page.locator('#sound-preview').click();
  await page.waitForFunction(()=>audit.contexts[1].state==='running');
  // Pending playback cannot escape a mute while resume is unresolved.
  await page.evaluate(async()=>{const c=audit.contexts[1];await c.suspend();const native=c.resume.bind(c);c.resume=()=>new Promise(resolve=>{audit.release=async()=>{c.resume=native;await native();resolve();};});audit.before=audit.starts.length;});
  await page.locator('#sound-preview').click();
  await page.locator('#sound-volume').evaluate(e=>{e.value='0';e.dispatchEvent(new Event('input',{bubbles:true}));});
  await page.evaluate(()=>audit.release());
  assert.equal(await page.evaluate(()=>audit.starts.length),await page.evaluate(()=>audit.before));
  for(const [id,value] of [['sound-volume','37'],['balloon-count','100']]) await page.locator('#'+id).evaluate((e,v)=>{e.value=v;e.dispatchEvent(new Event('input',{bubbles:true}));},value);
  await page.waitForFunction(()=>JSON.parse(localStorage.sudoku_state).effects.volume===37);
  await page.reload();await page.locator('#sound-volume').waitFor();
  assert.equal(await page.locator('#sound-volume').inputValue(),'37');assert.equal(await page.locator('#balloon-count').inputValue(),'100');
  await page.locator('#sound-volume').evaluate(e=>{e.value='0';e.dispatchEvent(new Event('input',{bubbles:true}));});
  await page.waitForFunction(()=>document.querySelector('#sound-preview').disabled);
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
  // A nearly complete legacy-compatible fixture exercises victory and reload suppression.
  await page.evaluate(()=>{const s=JSON.parse(localStorage.sudoku_state);s.solution=Array.from({length:81},(_,i)=>(Math.floor(i/9)*3+Math.floor(Math.floor(i/9)/3)+i%9)%9+1);s.board=[...s.solution];s.board[80]=0;s.givens=[...s.board];s.notes=Array(81).fill(0);s.cell_colors=Array(81).fill(0);s.hinted=Array(81).fill(0);s.history=[];s.redo_stack=[];s.selected=null;s.won=false;s.paused=false;s.note_mode=false;s.drop_mode=false;s.domino_enabled=false;s.sound_type='Explosion';s.effects={volume:75,balloons:0};localStorage.sudoku_state=JSON.stringify(s);});
  await page.goto(url);await page.locator('[data-row="8"][data-col="8"]').click();
  await page.locator('.number-choices > button').filter({hasText:/^8$/}).click();
  await page.locator('.victory-effects').waitFor();
  assert.equal(await page.locator('.victory-effects').getAttribute('data-balloon-count'),'0');
  await page.waitForFunction(()=>audit.starts.length>=20);
  const times=await page.evaluate(()=>audit.starts.map(s=>s.when));assert.ok(Math.max(...times)-Math.min(...times)>=6);
  await page.evaluate(()=>{audit.before=audit.starts.length;Object.defineProperty(document,'hidden',{configurable:true,get:()=>true});document.dispatchEvent(new Event('visibilitychange'));});
  assert.ok(await page.evaluate(()=>audit.stops)>=20);
  await page.evaluate(()=>{delete document.hidden;document.dispatchEvent(new Event('visibilitychange'));});
  await page.locator('body').click({position:{x:2,y:2}});
  assert.equal(await page.evaluate(()=>audit.starts.length),await page.evaluate(()=>audit.before));
  for(const count of [25,100]) {
   await page.locator('.entry-actions').getByRole('button',{name:'Undo',exact:true}).click();
   await page.evaluate(count=>{const s=JSON.parse(localStorage.sudoku_state);s.effects.balloons=count;localStorage.sudoku_state=JSON.stringify(s);},count);
   await page.reload();await page.locator('.sudoku-cell').first().waitFor();
   await page.locator('.entry-actions').getByRole('button',{name:'Redo',exact:true}).click();
   await page.locator('.victory-effects').waitFor();
   assert.equal(await page.locator('.victory-effects').getAttribute('data-balloon-count'),String(count));
  }
  await page.reload();await page.locator('.sudoku-cell').first().waitFor();assert.equal(await page.locator('.victory-effects').count(),0);
  assert.deepEqual(errors,[]);await context.close();console.log(`effects ${width} ${theme}: passed`);
 }
}finally{await browser.close();await new Promise(r=>server.close(r));}
