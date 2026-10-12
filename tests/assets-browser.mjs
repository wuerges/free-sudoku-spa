// Browser APIs decode artwork and exercise real service-worker asset delivery.
import {createServer} from 'node:http';
import {readFile,mkdir,readdir} from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const output=process.env.ASSET_SCREENSHOTS || '/tmp/sudoku-assets';await mkdir(output,{recursive:true});
const mime={'.html':'text/html','.wasm':'application/wasm','.js':'application/javascript','.css':'text/css','.svg':'image/svg+xml','.png':'image/png','.json':'application/json'};
const server=createServer(async(req,res)=>{
 const pathname=new URL(req.url,'http://localhost').pathname;
 const file=['/','/config','/help','/history'].includes(pathname)?'index.html':pathname.slice(1);
 try{res.setHeader('Content-Type',mime[path.extname(file)] || 'application/octet-stream');res.end(await readFile(path.join('dist',file)));}catch{res.writeHead(404);res.end();}
});await new Promise(r=>server.listen(0,'127.0.0.1',r));const url=`http://127.0.0.1:${server.address().port}`;
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH || undefined,args:['--no-sandbox']});
try {
 for(const width of [320,1280]) for(const theme of ['light','dark']) {
  const context=await browser.newContext({viewport:{width,height:844},colorScheme:theme,reducedMotion:'reduce',serviceWorkers:'block'});
  const page=await context.newPage();await page.route('**/*.wasm',route=>route.abort());
  await page.goto(url);const image=page.locator('#loading img');await image.waitFor();
  assert.equal(await image.evaluate(e=>e.complete && e.naturalWidth>0),true);
  assert.equal(await image.evaluate(e=>getComputedStyle(e).animationName),'none');
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
  await page.screenshot({path:path.join(output,`loading-${theme}-${width}.png`)});await context.close();
 }
 const context=await browser.newContext();const page=await context.newPage();const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(url);await page.locator('.sudoku-cell').first().waitFor();await page.locator('#loading').waitFor({state:'detached'});
 await page.evaluate(()=>navigator.serviceWorker.ready);await page.reload();await page.locator('.sudoku-cell').first().waitFor();await page.locator('#loading').waitFor({state:'detached'});
 const assets=[];async function visit(dir){for(const e of await readdir(path.join('dist',dir),{withFileTypes:true})){const p=path.posix.join(dir,e.name);if(e.isDirectory())await visit(p);else assets.push('/'+p);}}await visit('public');
 const manifest=JSON.parse(await readFile('manifest.json','utf8'));
 const images=[...manifest.icons.map(i=>({src:i.src,size:Number(i.sizes.split('x')[0]),mask:i.purpose==='maskable'})),{src:'/public/social-sudoku.png',size:1200,height:630},{src:'/public/icons/sudoku.svg',size:512}];
 for(const item of images){const result=await page.evaluate(async item=>{
  const image=new Image();image.src=item.src;await image.decode();const c=document.createElement('canvas');c.width=image.naturalWidth;c.height=image.naturalHeight;const x=c.getContext('2d');x.drawImage(image,0,0);const pixels=x.getImageData(0,0,c.width,c.height).data;
  const colors=new Set();let unsafe=0;for(let i=0;i<pixels.length;i+=4){const r=pixels[i],g=pixels[i+1],b=pixels[i+2];colors.add(`${r},${g},${b}`);if(item.mask && (r!==15||g!==23||b!==42)){const n=i/4;const dx=n%c.width+.5-c.width/2,dy=Math.floor(n/c.width)+.5-c.height/2;if(Math.hypot(dx,dy)>c.width*.4)unsafe++;}}
  return {width:c.width,height:c.height,colors:colors.size,unsafe};
 },item);assert.equal(result.width,item.size);assert.equal(result.height,item.height || item.size);assert.ok(result.colors>10,`${item.src}: blank artwork`);assert.equal(result.unsafe,0,`${item.src}: mask crops artwork`);}
 assert.deepEqual(await readFile('android/app/src/main/res/mipmap-nodpi/sudoku_brand.png'),await readFile('public/icons/icon-512.png'));
 assert.deepEqual(await readFile('android/app/src/main/res/mipmap-nodpi/sudoku_foreground.png'),await readFile('public/icons/icon-maskable-512.png'));
 // Inspect every inline control icon, including controls not mounted on the game route.
 const source=await readFile('src/components/icon.rs','utf8');const paths=[...source.matchAll(/IconName::(\w+) => "([^"]+)"/g)].map(m=>({name:m[1],d:m[2]}));assert.equal(paths.length,21);
 await page.evaluate(paths=>{const gallery=document.createElement('div');gallery.id='asset-gallery';gallery.style='background:white;color:#0f172a;display:flex;flex-wrap:wrap;gap:20px;padding:20px';gallery.innerHTML=paths.map(p=>`<div>${p.name}<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="${p.d}"/></svg></div>`).join('');document.body.prepend(gallery);},paths);
 const boxes=await page.locator('#asset-gallery path').evaluateAll(es=>es.map(e=>{const b=e.getBBox();return {x:b.x,y:b.y,width:b.width,height:b.height};}));for(const b of boxes){assert.ok(b.width>0 && b.height>0);assert.ok(b.x>=.9 && b.y>=.9 && b.x+b.width<=23.1 && b.y+b.height<=23.1);}
 await page.locator('#asset-gallery').screenshot({path:path.join(output,'control-icons.png')});await page.locator('#asset-gallery').evaluate(e=>e.remove());
 await context.setOffline(true);
 for(const asset of assets){assert.equal(await page.evaluate(async src=>{const r=await fetch(src);return r.ok;},asset),true,`offline asset ${asset}`);}
 for(const route of ['/config','/help','/history','/']){await page.goto(url+route);await page.locator('#loading').waitFor({state:'detached'});}
 assert.deepEqual(errors,[]);await context.close();console.log(`Assets passed: ${assets.length} packaged files, ${images.length} images, ${paths.length} control icons, loading themes/viewports and offline routes`);
}finally{await browser.close();await new Promise(r=>server.close(r));}
