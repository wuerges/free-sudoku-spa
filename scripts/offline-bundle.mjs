import { createHash } from 'node:crypto';
import { readdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export async function generateOfflineBundle(directory, templatePath = new URL('../sw.js', import.meta.url)) {
  const files = [];
  async function visit(relative = '') {
    for (const entry of await readdir(path.join(directory, relative), { withFileTypes: true })) {
      const name = path.posix.join(relative, entry.name);
      if (entry.isDirectory()) await visit(name);
      else if (entry.isFile() && name !== 'sw.js') files.push(name);
    }
  }
  await visit();
  files.sort();
  for (const required of ['index.html', 'manifest.json']) {
    if (!files.includes(required)) throw new Error(`Missing required offline asset: ${required}`);
  }
  for (const extension of ['.js', '.wasm', '.css']) {
    if (!files.some(name => name.endsWith(extension))) throw new Error(`Missing ${extension} bundle`);
  }
  const template = await readFile(templatePath, 'utf8');
  const hash = createHash('sha256').update(template);
  for (const name of files) {
    const bytes = await readFile(path.join(directory, name));
    hash.update(JSON.stringify([name, bytes.length])).update(bytes);
  }
  const version = hash.digest('hex');
  const assets = files.map(name => '/' + name.split('/').map(encodeURIComponent).join('/'));
  const output = template.replace('/* OFFLINE_BUNDLE */', `const VERSION = ${JSON.stringify(version)};\nconst ASSETS = ${JSON.stringify(assets)};`);
  if (output === template) throw new Error('Missing offline bundle template marker');
  await writeFile(path.join(directory, 'sw.js'), output);
  return { version, assets };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const directory = process.env.TRUNK_STAGING_DIR || process.argv[2];
  if (!directory) throw new Error('TRUNK_STAGING_DIR or output directory argument is required');
  await generateOfflineBundle(directory);
}
