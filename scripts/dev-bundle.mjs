// Development never precaches assets: reload must always see the current build.
import { copyFile } from 'node:fs/promises';
import path from 'node:path';
const stage = process.env.TRUNK_STAGING_DIR;
if (!stage) throw new Error('TRUNK_STAGING_DIR is required');
await copyFile('scripts/dev-worker.js', path.join(stage, 'sw.js'));
