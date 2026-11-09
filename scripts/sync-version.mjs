#!/usr/bin/env node
// 把 tag 版本同步进 package.json / Cargo.toml([workspace.package]) / src-tauri/tauri.conf.json
// 用法: node scripts/sync-version.mjs 0.2.0
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const ver = (process.argv[2] || '').trim();
if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(ver)) {
  console.error(`用法: node scripts/sync-version.mjs <x.y.z>，收到: ${ver || '(空)'}`);
  process.exit(1);
}

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');

function patch(rel, replacer) {
  const p = resolve(root, rel);
  const before = readFileSync(p, 'utf8');
  const after = replacer(before);
  if (after === before) {
    console.log(`= ${rel} 已是 ${ver}`);
    return;
  }
  writeFileSync(p, after, 'utf8');
  console.log(`+ ${rel} -> ${ver}`);
}

// package.json 顶层 version
patch('package.json', (s) => s.replace(/("version"\s*:\s*")[^"]*(")/, `$1${ver}$2`));

// Cargo.toml 只改 [workspace.package] 里的 version
patch('Cargo.toml', (s) => {
  const m = s.match(/\[workspace\.package\][\s\S]*?(?=\n\[|$)/);
  if (!m) throw new Error('Cargo.toml 缺少 [workspace.package]');
  const block = m[0].replace(/^version\s*=\s*"[^"]*"/m, `version = "${ver}"`);
  return s.replace(m[0], block);
});

// tauri.conf.json 顶层 version
patch('src-tauri/tauri.conf.json', (s) => s.replace(/("version"\s*:\s*")[^"]*(")/, `$1${ver}$2`));
