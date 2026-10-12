import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

export const locales = ['en-US', 'es-AR', 'gn-PY', 'pt-BR'];
export async function readCatalogs() {
  const catalogs = {};
  for (const locale of locales) {
    catalogs[locale] = JSON.parse(await readFile(new URL(`../locales/${locale}.json`, import.meta.url), 'utf8'));
  }
  validateCatalogs(catalogs);
  return catalogs;
}

export function validateCatalogs(catalogs) {
  if (JSON.stringify(Object.keys(catalogs).sort()) !== JSON.stringify([...locales].sort())) throw Error('Supported locale catalogs differ');
  const keys = Object.keys(catalogs['en-US']).sort();
  const placeholders = text => [...text.matchAll(/\{([a-z]+)\}/g)].map(match => match[1]).sort();
  for (const locale of locales) {
    const messages = catalogs[locale];
    if (JSON.stringify(Object.keys(messages).sort()) !== JSON.stringify(keys)) throw Error(`${locale}: catalog keys differ`);
    for (const key of keys) {
      if (!/^[A-Z][A-Za-z]+$/.test(key) || typeof messages[key] !== 'string' || !messages[key].trim()) throw Error(`${locale}: invalid message ${key}`);
      if (/[{}]/.test(messages[key].replace(/\{[a-z]+\}/g, ''))) throw Error(`${locale}: malformed placeholders for ${key}`);
      if (JSON.stringify(placeholders(messages[key])) !== JSON.stringify(placeholders(catalogs['en-US'][key]))) throw Error(`${locale}: placeholders differ for ${key}`);
    }
  }
}

export async function generateRust(destination) {
  const catalogs = await readCatalogs();
  const keys = Object.keys(catalogs['en-US']);
  const output = `// Generated from locales/*.json; do not edit.\n// Some keys are consumed only by the JavaScript bootstrap or WASM storage.\n#[allow(dead_code)]\n#[derive(Clone, Copy, Debug, PartialEq, Eq)]\npub enum Message {\n${keys.map(key => `    ${key},`).join('\n')}\n}\n` +
    `pub const MESSAGES: [[&str; ${keys.length}]; 4] = [\n${locales.map(locale => `    [${keys.map(key => JSON.stringify(catalogs[locale][key])).join(',')}],`).join('\n')}\n];\n`;
  await writeFile(destination, output);
}

export async function configureLocalization(directory) {
  const catalogs = await readCatalogs();
  const runtime = await readFile(new URL('../public/localization.js', import.meta.url), 'utf8');
  await mkdir(path.join(directory, 'public'), { recursive: true });
  await writeFile(path.join(directory, 'public/localization.js'), `window.__sudokuCatalogs = ${JSON.stringify(catalogs)};\n${runtime}`);
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === '--rust' && process.argv[3]) await generateRust(process.argv[3]);
  else if (process.argv[2] === '--check') await readCatalogs();
  else throw Error('Use --rust <output> or --check');
}
