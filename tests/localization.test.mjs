import test from 'node:test';
import assert from 'node:assert/strict';
import vm from 'node:vm';
import { readFile, mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { readCatalogs, validateCatalogs, configureLocalization, generateRust } from '../scripts/localization.mjs';
import { generateOfflineBundle } from '../scripts/offline-bundle.mjs';
import { stage } from '../scripts/android.mjs';

const catalogs = await readCatalogs();
const runtime = await readFile('public/localization.js', 'utf8');
function boot({ saved = null, languages, language, unavailable = false, store = new Map(), denied = false } = {}) {
  if (saved !== null) store.set('sudoku_locale', saved);
  const nodes = [{dataset:{i18n:'LoadingIntro'},textContent:''}];
  const status = {dataset:{},textContent:''};
  const document = {documentElement:{}, title:'', querySelectorAll:()=>nodes, getElementById:()=>status, addEventListener:(_name, fn)=>fn()};
  const localStorage = {getItem:key=>{if(unavailable) throw Error('unavailable');return store.get(key) ?? null;},setItem:(key,value)=>{if(unavailable || denied) throw Error('quota');store.set(key,value);}};
  const window = {__sudokuCatalogs:catalogs};
  vm.runInNewContext(runtime, {window,document,localStorage,navigator:{languages,language},Intl,Date});
  return {api:window.__sudokuI18n,document,store,status,nodes};
}

test('catalogs contain complete matching keys and named placeholders', () => {
  assert.equal(Object.keys(catalogs).length,4);
  assert.ok(Object.keys(catalogs['en-US']).length >= 200);
  assert.equal(new Set(Object.values(catalogs).map(messages=>messages.Settings)).size,4);
});
test('validation rejects missing keys, missing languages and broken placeholders', () => {
  const missingKey=structuredClone(catalogs);delete missingKey['gn-PY'].Settings;
  assert.throws(()=>validateCatalogs(missingKey),/catalog keys differ/);
  const missingLocale=structuredClone(catalogs);delete missingLocale['gn-PY'];
  assert.throws(()=>validateCatalogs(missingLocale),/Supported locale catalogs differ/);
  const missingArgument=structuredClone(catalogs);missingArgument['es-AR'].GameTime='Tiempo';
  assert.throws(()=>validateCatalogs(missingArgument),/placeholders differ/);
  const malformed=structuredClone(catalogs);malformed['pt-BR'].GameTime='Tempo {time';
  assert.throws(()=>validateCatalogs(malformed),/malformed placeholders/);
});
test('browser preference order, regional matching and language fallback', () => {
  for(const [languages, language, expected] of [
    [['es-AR','en-US'],null,'es-AR'], [['gn-PY'],null,'gn-PY'], [['pt-BR'],null,'pt-BR'], [['en-US'],null,'en-US'],
    [['fr-FR','pt-PT'],null,'pt-BR'], [['en-GB','es-AR'],null,'en-US'], [['ES-ar'],null,'es-AR'],
    [['gn'],null,'gn-PY'], [['es-MX'],null,'es-AR'], [[], 'pt-BR','pt-BR'],
    [undefined,'gn-PY','gn-PY'], [undefined,undefined,'en-US'], [['fr-FR'],'fr-FR','en-US'],
  ]) {
    const result=boot({languages,language});
    assert.equal(result.api.locale,expected);
    assert.equal(result.store.get('sudoku_locale'),expected,'initial selection must be saved');
    assert.equal(result.document.documentElement.lang,expected);
    assert.equal(result.document.title,catalogs[expected].Title);
    assert.equal(result.nodes[0].textContent,catalogs[expected].LoadingIntro);
  }
});
test('saved locale wins over changed browser settings and invalid saves are replaced', () => {
  assert.equal(boot({saved:'gn-PY',languages:['en-US']}).api.locale,'gn-PY');
  const result=boot({saved:'automatic',languages:['es-AR']});
  assert.equal(result.api.locale,'es-AR');
  assert.equal(result.store.get('sudoku_locale'),'es-AR');
});
test('manual choice and English fallback are saved across launches without touching games', () => {
  const store=new Map([['sudoku_state','progress'],['other_app','unchanged']]);
  const first=boot({store});
  assert.equal(store.get('sudoku_locale'),'en-US');
  first.api.setLocale('pt-BR');
  assert.equal(boot({store,languages:['es-AR']}).api.locale,'pt-BR');
  assert.equal(store.get('sudoku_state'),'progress');
  assert.equal(store.get('other_app'),'unchanged');
  first.api.setLocale('invalid');
  assert.equal(first.api.locale,'pt-BR');
});
test('denied or unavailable storage keeps a usable session with failure feedback', () => {
  for(const failure of [{unavailable:true},{denied:true}]) {
    const result=boot({...failure,languages:['es-AR']});
    assert.equal(result.api.locale,'es-AR');
    assert.equal(result.api.saveFailed,true);
    result.api.setLocale('gn-PY');
    assert.equal(result.api.locale,'gn-PY');
    assert.equal(result.api.saveFailed,true);
    assert.equal(result.document.documentElement.lang,'gn-PY');
  }
});
test('sharing notices change language and Guarani dates remain usable', () => {
  const result=boot({languages:['es-AR']});
  result.api.shareStatus('LinkCopied');
  assert.equal(result.status.textContent,catalogs['es-AR'].LinkCopied);
  result.api.setLocale('gn-PY');
  assert.equal(result.status.textContent,catalogs['gn-PY'].LinkCopied);
  assert.match(result.api.playedDate(Date.UTC(2026,9,12)),/2026/);
  result.api.shareStatus('');
  assert.equal(result.status.textContent,'');
});
test('generated catalogs join offline assets, change the cache version, and survive APK staging', async t => {
  const directory=await mkdtemp(path.join(tmpdir(),'sudoku-i18n-'));
  t.after(()=>rm(directory,{recursive:true,force:true}));
  const web=path.join(directory,'web');
  await mkdir(web);
  for(const [name,body] of Object.entries({'index.html':'<html><head></head></html>','manifest.json':'{}','app.js':'init()','app.wasm':'wasm','app.css':'body{}'})) await writeFile(path.join(web,name),body);
  await configureLocalization(web);
  const before=await generateOfflineBundle(web);
  assert.ok(before.assets.includes('/public/localization.js'));
  const output=await readFile(path.join(web,'public/localization.js'),'utf8');
  for(const locale of Object.keys(catalogs)) assert.ok(output.includes(locale));
  await writeFile(path.join(web,'public/localization.js'),output+'\n// changed translation');
  assert.notEqual((await generateOfflineBundle(web)).version,before.version);
  stage(web,path.join(directory,'native'));
  assert.equal(await readFile(path.join(directory,'native/public/localization.js'),'utf8'),output+'\n// changed translation');
  await generateRust(path.join(directory,'messages.rs'));
  assert.match(await readFile(path.join(directory,'messages.rs'),'utf8'),/pub enum Message/);
});
