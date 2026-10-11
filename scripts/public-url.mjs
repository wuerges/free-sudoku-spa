// Shared build-time resolver for Rust and staged social metadata.
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const DEFAULT_PUBLIC_URL = 'https://free-sudoku-spa.vercel.app';

export function resolvePublicUrl(env = process.env) {
  let candidate = DEFAULT_PUBLIC_URL;
  if (env.SUDOKU_PUBLIC_URL !== undefined) candidate = env.SUDOKU_PUBLIC_URL;
  else if (env.VERCEL_ENV === 'production' && env.VERCEL_PROJECT_PRODUCTION_URL) candidate = `https://${env.VERCEL_PROJECT_PRODUCTION_URL}`;
  else if (env.VERCEL_URL) candidate = `https://${env.VERCEL_URL}`;
  let url;
  try { url = new URL(candidate); }
  catch { throw new Error('Public URL must be an absolute HTTP/HTTPS origin'); }
  if (!['http:', 'https:'].includes(url.protocol) || !url.hostname || url.username || url.password || url.pathname !== '/' || /[?#]/.test(candidate)) {
    throw new Error('Public URL must be an HTTP/HTTPS origin without credentials, path, query or fragment');
  }
  return url.origin;
}

export async function configureSocialMetadata(directory, env = process.env) {
  const origin = resolvePublicUrl(env);
  const filename = path.join(directory, 'index.html');
  const html = await readFile(filename, 'utf8');
  let count = 0;
  const output = html.replace(/<meta\b[^>]*\bdata-public-path="([^"]*)"[^>]*>/g, (tag, suffix) => {
    if (!['/', '/public/social-sudoku.png'].includes(suffix)) throw new Error('Invalid public metadata path');
    if (!/\bcontent="[^"]*"/.test(tag)) throw new Error('Missing public metadata content');
    count++;
    return tag.replace(/\bcontent="[^"]*"/, `content="${origin}${suffix}"`);
  });
  if (count !== 3) throw new Error('Expected three public URL metadata tags');
  await writeFile(filename, output);
  return origin;
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  console.log(resolvePublicUrl());
}
