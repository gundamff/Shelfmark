#!/usr/bin/env node
// 从 CHANGELOG.md 提取指定版本的段落，输出到 stdout
// 用法: node scripts/changelog-for-version.mjs 0.1.0 > release-notes.md
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const ver = (process.argv[2] || '').trim().replace(/^v/, '');
if (!ver) {
  console.error('用法: node scripts/changelog-for-version.mjs <x.y.z>');
  process.exit(1);
}

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const lines = readFileSync(resolve(root, 'CHANGELOG.md'), 'utf8').split(/\r?\n/);

const esc = ver.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const startRe = new RegExp(`^##\\s+v?${esc}(\\s|$)`);
const start = lines.findIndex((l) => startRe.test(l));
if (start < 0) {
  console.error(`CHANGELOG.md 里找不到 ${ver} 的段落（期望形如 "## v${ver} ..."）`);
  process.exit(1);
}

let end = lines.length;
for (let i = start + 1; i < lines.length; i += 1) {
  if (/^##\s/.test(lines[i])) {
    end = i;
    break;
  }
}

process.stdout.write(`${lines.slice(start, end).join('\n').trim()}\n`);
