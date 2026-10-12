// Chromium SVG rasterization keeps web and native artwork aligned.
import {readFile,writeFile} from 'node:fs/promises';
const {chromium}=await import(process.env.PLAYWRIGHT_MODULE || 'playwright');
const svg=await readFile('public/icons/sudoku.svg','utf8');
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH || undefined,args:['--no-sandbox']});
try {
 for(const [file,size,maskable] of [['public/icons/icon-192.png',192,false],['public/icons/icon-512.png',512,false],['public/icons/icon-maskable-512.png',512,true]]) {
  const page=await browser.newPage({viewport:{width:size,height:size},deviceScaleFactor:1});
  const content=svg.replace(/<svg[^>]*>/,'').replace('</svg>','').replace(/<rect width="512" height="512" rx="88" fill="#0f172a"\/>/,'');
  const artwork=maskable?`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><rect width="512" height="512" fill="#0f172a"/><g transform="translate(76.8 76.8) scale(.7)">${content}</g></svg>`:svg;
  await page.setContent(`<style>body{margin:0}svg{display:block;width:100vw;height:100vh}</style>${artwork}`);
  await page.screenshot({path:file});await page.close();
 }
 await writeFile('android/app/src/main/res/mipmap-nodpi/sudoku_brand.png',await readFile('public/icons/icon-512.png'));
 await writeFile('android/app/src/main/res/mipmap-nodpi/sudoku_foreground.png',await readFile('public/icons/icon-maskable-512.png'));
}finally{await browser.close();}
