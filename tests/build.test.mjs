import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, mkdir, cp, writeFile, readFile, rm, chmod } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

async function fixture(t, { tools = true, platform = 'Linux', curlFailure = false } = {}) {
  const root = await mkdtemp(path.join(tmpdir(), 'sudoku-build-'));
  t.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(path.join(root, 'scripts'));
  for (const name of ['build.sh', 'rust-toolchain.toml', 'scripts/setup-build.sh', 'scripts/build-env.sh', 'scripts/with-build-env.sh', 'Justfile']) {
    await cp(name, path.join(root, name));
  }
  const fake = path.join(root, 'fake-bin');
  await mkdir(fake);
  async function executable(name, body) {
    const target = path.join(root, name);
    await mkdir(path.dirname(target), { recursive: true });
    await writeFile(target, '#!/bin/sh\nset -eu\n' + body + '\n');
    await chmod(target, 0o755);
  }
  await executable('fake-bin/node', 'exit 0');
  await executable('fake-bin/npm', 'echo "npm $*" >> "$TEST_LOG"');
  await executable('fake-bin/uname', `case "$1" in -s) echo '${platform}' ;; -m) echo x86_64 ;; esac`);
  await executable('fake-bin/curl', curlFailure ? 'echo "download failed" >&2; exit 22' :
    'while [ "$#" -gt 0 ]; do if [ "$1" = "-o" ]; then shift; echo corrupted > "$1"; exit 0; fi; shift; done; exit 1');
  if (tools) {
    await executable('.build-tools/cargo/bin/rustup', 'echo "rustup $*" >> "$TEST_LOG"');
    await executable('.build-tools/cargo/bin/rustc', 'echo "rustc 1.98.1"');
    await executable('.build-tools/bin/trunk', 'if [ "$1" = "--version" ]; then echo "trunk 0.21.14"; else echo "trunk $*" >> "$TEST_LOG"; echo "$CARGO_HOME|$RUSTUP_HOME|${NO_COLOR:-}" >> "$TEST_LOG"; fi');
  }
  return {
    root,
    executable,
    run: file => spawnSync('sh', [path.join(root, file)], {
      cwd: tmpdir(), encoding: 'utf8',
      env: { ...process.env, PATH: fake + ':/usr/bin:/bin', TEST_LOG: path.join(root, 'commands'), NO_COLOR: '1' },
    }),
    log: () => readFile(path.join(root, 'commands'), 'utf8'),
  };
}

test('cached setup uses pinned local tools and locked npm dependencies', async t => {
  const f = await fixture(t);
  const result = f.run('scripts/setup-build.sh');
  assert.equal(result.status, 0, result.stderr);
  const log = await f.log();
  assert.match(log, /npm ci/);
  assert.match(log, /rustup toolchain install 1\.98\.1/);
  assert.match(log, /rustup target add --toolchain 1\.98\.1 wasm32-unknown-unknown/);
  assert.match(log, /rustup component add --toolchain 1\.98\.1 rustfmt clippy/);
});

test('build uses the same local environment and locks Cargo dependencies', async t => {
  const f = await fixture(t);
  const result = f.run('build.sh');
  assert.equal(result.status, 0, result.stderr);
  const log = await f.log();
  assert.match(log, /npm run test:offline\nnpm run css\ntrunk build --release --locked/);
  assert.ok(log.includes(`${f.root}/.build-tools/cargo|${f.root}/.build-tools/rustup|true`));
});

test('build fails clearly without project-local tools', async t => {
  const f = await fixture(t, { tools: false });
  const result = f.run('build.sh');
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /run just setup/);
});

test('setup rejects unsupported platforms before installing dependencies', async t => {
  const f = await fixture(t, { platform: 'Darwin' });
  const result = f.run('scripts/setup-build.sh');
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /supports Linux/);
  await assert.rejects(f.log(), /ENOENT/);
});

test('setup rejects corrupt Trunk downloads without installing them', async t => {
  const f = await fixture(t);
  await rm(path.join(f.root, '.build-tools/bin/trunk'));
  const result = f.run('scripts/setup-build.sh');
  assert.notEqual(result.status, 0);
  assert.match(result.stdout + result.stderr, /FAILED|did NOT match/);
  await assert.rejects(readFile(path.join(f.root, '.build-tools/bin/trunk')), /ENOENT/);
});

test('setup propagates download failure and does not install a partial tool', async t => {
  const f = await fixture(t, { curlFailure: true });
  await rm(path.join(f.root, '.build-tools/bin/trunk'));
  const result = f.run('scripts/setup-build.sh');
  assert.equal(result.status, 22);
  assert.match(result.stderr, /download failed/);
  await assert.rejects(readFile(path.join(f.root, '.build-tools/bin/trunk')), /ENOENT/);
});

test('build rejects a different Node major before running npm or Trunk', async t => {
  const f = await fixture(t);
  await f.executable('fake-bin/node', 'exit 1');
  const result = f.run('build.sh');
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Node.js 24 is required/);
  await assert.rejects(f.log(), /ENOENT/);
});

test('development wrapper uses pinned tools and propagates command failures', async t => {
  const f = await fixture(t);
  await f.executable('fake-bin/dev-test', 'echo "$CARGO_HOME|$RUSTUP_HOME|$NO_COLOR" >> "$TEST_LOG"; exit 7');
  // Pass a command explicitly, as the Just recipes do.
  const result = spawnSync('sh', [path.join(f.root, 'scripts/with-build-env.sh'), 'dev-test'], {
    cwd: tmpdir(), encoding: 'utf8',
    env: {...process.env, PATH:path.join(f.root,'fake-bin')+':/usr/bin:/bin',TEST_LOG:path.join(f.root,'commands'),NO_COLOR:'1'},
  });
  assert.equal(result.status, 7);
  assert.ok((await f.log()).includes(`${f.root}/.build-tools/cargo|${f.root}/.build-tools/rustup|true`));
});
