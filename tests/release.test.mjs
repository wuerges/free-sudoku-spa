import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { validateVersion, assertTagTarget, publishRelease, selectMergedPR } from '../scripts/release-policy.mjs';
const files = Object.fromEntries(['Cargo.toml','Cargo.lock','package.json','package-lock.json','CHANGELOG.md'].map(n => [n, readFileSync(n,'utf8')]));
const body = 'Game-state compatibility: compatible';
test('initial aligned release and compatible minor bump', () => {
  assert.equal(validateVersion(files, undefined, body, true).version, '0.9.0');
  assert.equal(validateVersion(files, '0.8.0', body).version, '0.9.0');
});
test('reject missing declaration, wrong bump, missing notes, and divergent manifests', () => {
  assert.throws(() => validateVersion(files, '0.8.0', ''));
  assert.throws(() => validateVersion(files, '0.9.0', body));
  assert.throws(() => validateVersion(files, '0.8.0', 'Game-state compatibility: breaking'));
  assert.throws(() => validateVersion({...files, 'CHANGELOG.md': ''}, '0.8.0', body));
  assert.throws(() => validateVersion({...files, 'Cargo.toml': files['Cargo.toml'].replace('0.9.0','0.8.0')}, '0.8.0', body));
});
test('breaking changes require a major bump', () => {
  const major = Object.fromEntries(Object.entries(files).map(([k,v]) => [k,v.replaceAll('0.9.0','1.0.0')]));
  assert.equal(validateVersion(major, '0.9.0', 'Game-state compatibility: breaking').version, '1.0.0');
});
test('release reruns must never move an existing tag', () => {
  assert.doesNotThrow(() => assertTagTarget({type:'commit',sha:'abc'}, 'abc'));
  assert.throws(() => assertTagTarget({type:'commit',sha:'other'}, 'abc'));
});

test('release creation, recovery, and idempotent reruns use the verified commit', () => {
  const metadata={version:'0.9.0',notes:'Release notes'};
  let calls=0;
  const create=(tag,notes)=>{calls++;assert.equal(tag,'v0.9.0');assert.equal(notes,metadata.notes);};
  const ref={ref:'refs/tags/v0.9.0',object:{type:'commit',sha:'abc'}};
  publishRelease(metadata,'abc','owner/repo',p=>p.includes('matching-refs')?[]:null,create);
  publishRelease(metadata,'abc','owner/repo',p=>p.includes('matching-refs')?[ref]:null,create);
  assert.equal(calls,2);
  publishRelease(metadata,'abc','owner/repo',p=>p.includes('matching-refs')?[ref]:{draft:false},create);
  assert.equal(calls,2);
  assert.throws(()=>publishRelease(metadata,'other','owner/repo',()=>[ref],create));
  assert.equal(calls,2);
});

test('publish only an exact merged main-branch PR, never a feature merge or direct push', () => {
  const pr={merged_at:'2026-10-09',base:{ref:'main'},merge_commit_sha:'abc'};
  assert.equal(selectMergedPR([pr],'abc'),pr);
  assert.equal(selectMergedPR([],'abc'),undefined);
  assert.equal(selectMergedPR([pr],'other'),undefined);
  assert.equal(selectMergedPR([{...pr,merged_at:null}],'abc'),undefined);
  assert.equal(selectMergedPR([{...pr,base:{ref:'feature'}}],'abc'),undefined);
});
