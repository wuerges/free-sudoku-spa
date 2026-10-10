import test from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
const css = await readFile('style/input.css', 'utf8');
const palettes = Object.fromEntries([':root', '.dark'].map(selector => {
  const body = css.slice(css.indexOf(selector + ' {')).split('}')[0];
  return [selector, Object.fromEntries([...body.matchAll(/--ui-([\w-]+):\s*(#[0-9a-f]{6})/g)].map(m => [m[1], m[2]]))];
}));
function luminance(hex) {
  return [1, 3, 5].map(i => parseInt(hex.slice(i, i + 2), 16) / 255)
    .map(v => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4)
    .reduce((sum, v, i) => sum + v * [.2126, .7152, .0722][i], 0);
}
function contrast(a, b) { const [lo, hi] = [luminance(a), luminance(b)].sort((a,b) => a-b); return (hi + .05) / (lo + .05); }
for (const [theme, p] of Object.entries(palettes)) {
  test(theme + ': readable numbers, notes, hints, errors, and control labels', () => {
    for (const fg of ['text', 'user', 'notes', 'accent']) {
      for (const bg of ['available','blocked','matching-blocked','blocked-stripe','cell','cell-hover','peer','matching','selected','secondary','error','hint']) {
        assert.ok(contrast(p[fg],p[bg]) >= 5.5, `${fg}/${bg}: ${contrast(p[fg],p[bg])}`);
      }
    }
    for (const [fg,bg] of [['error-text','error'],['hint-text','hint'],['success-text','success'],['on-primary','primary'],['on-primary','primary-hover'],['text','control'],['text','control-hover'],['muted','page'],['muted','control'],['accent','page'],['accent','control'],['error-text','selected'],['hint-text','selected'],['success-text','page']]) {
      assert.ok(contrast(p[fg],p[bg]) >= 4.5, `${fg}/${bg}`);
    }
  });
  test(theme + ': slider blends retain readable notes and grid lines', () => {
    for (const source of ['selected','peer','matching','blocked','matching-blocked','available']) {
      for (const percent of [0,25,50,75,100]) {
        const mixed='#'+[1,3,5].map(i=>Math.round(parseInt(p[source].slice(i,i+2),16)*percent/100+parseInt(p.cell.slice(i,i+2),16)*(1-percent/100)).toString(16).padStart(2,'0')).join('');
        for(const fg of ['text','user','notes','accent']) assert.ok(contrast(p[fg],mixed)>=5.5, `${fg}/${source} ${percent}%`);
        for(const fg of ['grid-thin','grid-strong','accent']) assert.ok(contrast(p[fg],mixed)>=3, `${fg}/${source} ${percent}%`);
      }
    }
  });
  test(theme + ': completed keypad slider preserves readable digits and checkmarks', () => {
    for (let percent = 0; percent <= 100; percent++) {
      const mixed = '#'+[1,3,5].map(i=>Math.round(parseInt(p.completed.slice(i,i+2),16)*percent/100+parseInt(p.control.slice(i,i+2),16)*(1-percent/100)).toString(16).padStart(2,'0')).join('');
      assert.ok(contrast(p.text,mixed)>=5.5, `completed ${percent}%: ${contrast(p.text,mixed)}`);
    }
    assert.ok(contrast(p.completed,p.control)>=1.9, 'completed fill differs from remaining digits');
    assert.ok(contrast(p.text,p.completed)>=3, 'completed outline remains visible');
  });
  test(theme + ': distinct peer, matching, and selected backgrounds', () => {
    // Product hierarchy targets, not WCAG thresholds for every tinted fill.
    for (const [a,b,min] of [['selected','cell',1.5],['peer','cell',1.5],['matching','cell',1.5],['available','cell',1.5],['blocked-stripe','blocked',1.3],['blocked-stripe','matching-blocked',1.3]]) {
      assert.ok(contrast(p[a],p[b]) >= min, `${a}/${b}: ${contrast(p[a],p[b])}`);
    }
  });
  test(theme + ': visible grid boundaries and selection/focus indicators', () => {
    for (const fg of ['grid-thin','grid-strong','accent']) {
      for(const bg of ['available','blocked','matching-blocked','blocked-stripe','cell','cell-hover','peer','matching','selected','secondary','error','hint']) {
        assert.ok(contrast(p[fg],p[bg]) >= 3, `${fg}/${bg}`);
      }
    }
  });
}
