import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import vm from 'node:vm';
import { versionCode, requireSigning, stage } from '../scripts/android.mjs';
test('version mapping is bounded and monotonic', () => {
  assert.equal(versionCode('0.17.0'), 17000);
  assert.ok(versionCode('1.0.0') > versionCode('0.999.999'));
  for (const v of ['0.1000.0','1.0.1000','2101.0.0','01.0.0','0.0.0','1.2.3-beta']) assert.throws(() => versionCode(v));
});
test('release signing cannot silently use debug credentials', () => {
  const credentials = Object.fromEntries(['ANDROID_KEYSTORE_BASE64', 'ANDROID_KEYSTORE_PASSWORD', 'ANDROID_KEY_ALIAS', 'ANDROID_KEY_PASSWORD'].map(key=>[key,'private-test-value']));
  requireSigning(credentials);
  for (const key of Object.keys(credentials)) {
    const missing = {...credentials}; delete missing[key];
    assert.throws(() => requireSigning(missing), new RegExp(`Missing release signing credential: ${key}`));
  }
});
test('staging preserves web output and suppresses native worker and prompts', () => {
  const root = mkdtempSync(`${tmpdir()}/android-test-`);
  try {
    mkdirSync(`${root}/web`);
    const html = readFileSync('index.html','utf8');
    writeFileSync(`${root}/web/index.html`,html);
    writeFileSync(`${root}/web/sw.js`,'worker');
    stage(`${root}/web`,`${root}/native`);
    assert.equal(readFileSync(`${root}/web/index.html`,'utf8'),html);
    assert.ok(existsSync(`${root}/web/sw.js`));
    assert.ok(!existsSync(`${root}/native/sw.js`));
    const scripts = [...readFileSync(`${root}/native/index.html`,'utf8').matchAll(/<script>([\s\S]*?)<\/script>/g)].map(m=>m[1]);
    const window = { matchMedia:()=>({matches:false}), addEventListener:()=>{throw Error('native prompt registered');} };
    const navigator = { serviceWorker:{register:()=>{throw Error('native worker registered');}} };
    for (const script of scripts) vm.runInNewContext(script,{window,navigator,document:{addEventListener(){},documentElement:{classList:{toggle(){}}}}});
  } finally { rmSync(root,{recursive:true,force:true}); }
});
