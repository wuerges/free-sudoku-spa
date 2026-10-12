import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { resolvePublicUrl, configureSocialMetadata, DEFAULT_PUBLIC_URL } from '../scripts/public-url.mjs';
import { generateOfflineBundle } from '../scripts/offline-bundle.mjs';
import { stage } from '../scripts/android.mjs';

const vercel = {VERCEL_URL:'preview-123.vercel.app',VERCEL_PROJECT_PRODUCTION_URL:'sudoku.example.com'};
test('public URL precedence keeps production variables out of preview links', () => {
  assert.equal(resolvePublicUrl({}),DEFAULT_PUBLIC_URL);
  assert.equal(resolvePublicUrl({...vercel,VERCEL_ENV:'preview'}),'https://preview-123.vercel.app');
  assert.equal(resolvePublicUrl({...vercel,VERCEL_ENV:'development'}),'https://preview-123.vercel.app');
  assert.equal(resolvePublicUrl({...vercel,VERCEL_ENV:'production'}),'https://sudoku.example.com');
  assert.equal(resolvePublicUrl({VERCEL_ENV:'production',VERCEL_URL:vercel.VERCEL_URL}),'https://preview-123.vercel.app');
  assert.equal(resolvePublicUrl({VERCEL_PROJECT_PRODUCTION_URL:vercel.VERCEL_PROJECT_PRODUCTION_URL}),DEFAULT_PUBLIC_URL);
  assert.equal(resolvePublicUrl({...vercel,VERCEL_ENV:'production',SUDOKU_PUBLIC_URL:'https://custom.example.com/'}),'https://custom.example.com');
  assert.equal(resolvePublicUrl({SUDOKU_PUBLIC_URL:'http://localhost:8080/'}),'http://localhost:8080');
});
test('invalid public URLs fail rather than silently falling back', () => {
  for (const value of ['', 'example.com', 'ftp://example.com', 'https://user:pass@example.com', 'https://example.com/sudoku', 'https://example.com/?q=1', 'https://example.com/#x', 'https://example.com/?', 'https://example.com/#']) {
    assert.throws(()=>resolvePublicUrl({SUDOKU_PUBLIC_URL:value}),/Public URL/);
  }
});
test('social metadata is staged before offline hashing and retained by Android', async t => {
  const root=await mkdtemp(path.join(tmpdir(),'sudoku-public-url-'));
  t.after(()=>rm(root,{recursive:true,force:true}));
  const original=await readFile('index.html','utf8');
  await writeFile(path.join(root,'index.html'),original);
  for (const [name,content] of [['manifest.json','{}'],['app.js','js'],['app.wasm','wasm'],['app.css','css']]) await writeFile(path.join(root,name),content);
  await configureSocialMetadata(root,{...vercel,VERCEL_ENV:'preview'});
  const first=await generateOfflineBundle(root);
  const firstHtml=await readFile(path.join(root,'index.html'),'utf8');
  assert.match(firstHtml,/content="https:\/\/preview-123.vercel.app\/"/);
  assert.match(firstHtml,/content="https:\/\/preview-123.vercel.app\/public\/social-sudoku.png"/);
  await configureSocialMetadata(root,{SUDOKU_PUBLIC_URL:'https://other.example.com/'});
  const second=await generateOfflineBundle(root);
  assert.notEqual(first.version,second.version);
  const html=await readFile(path.join(root,'index.html'),'utf8');
  assert.equal((html.match(/content="https:\/\/other.example.com/g)||[]).length,3);
  assert.equal(await readFile('index.html','utf8'),original);
  const destination=root+'-android';t.after(()=>rm(destination,{recursive:true,force:true}));
  stage(root,destination);
  assert.match(await readFile(path.join(destination,'index.html'),'utf8'),/https:\/\/other.example.com\/public\/social-sudoku.png/);
  await writeFile(path.join(root,'index.html'),'<head></head>');
  await assert.rejects(configureSocialMetadata(root,{}),/three public URL metadata tags/);
});
