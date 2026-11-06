# Task: Fix Shelfmark AutoCat pipeline and live-library naming (2026-11-06)

Repo: D:\gits\Depressurizer (Rust workspace + Tauri 2 + React)
Do NOT change data on the user's machine outside the repo. Do not write to the Steam cloud-storage file.

## Background (measured on a real machine)

Live session opened with Steam user id 136330193:
- 762 app entries loaded (11 with appmanifest name, the rest show as `App <id>` because only `appmanifest_*.acf` provides a name)
- Metadata cache: 120 apps after 5 manual batches; each batch = 40 apps and needs ~50 s (network)
- AutoCat "genre" pack preview prints: `匹配 120，跳过 642，新建 18，添加操作 0`
- After clicking "加入待导出", the export queue JSON is completely empty:
  `{"create":0,"rename":0,"delete":0,"add_members":0,"remove_members":0}`
- UI banner always shows "尚未检测版本 — 请先『检测 Steam』" even though `steam.exe` exists and has a FileVersion

## Issue 1 (blocker): AutoCat preview result never reaches the export queue

Symptom: preview reports N new collections but 0 add operations, and "加入待导出" produces an empty changeset.

Required:
- Trace the AutoCat pipeline end to end: rule pack -> match -> plan (create/rename/add/remove) -> preview -> apply-to-queue.
- Fix the plan generation so that matched games actually produce `add_members` operations, and new collections are created with their members. A collection with zero members must not be created.
- Make preview counts and queue counts consistent by construction: whatever preview reports must be exactly what appears in the queue after "加入待导出" (same numbers).
- Respect the existing coverage policies (skip-same-name / merge-append / replace-members).
- Add unit tests at the plan level that lock this behaviour: (a) matched games produce add ops, (b) empty collections are dropped, (c) preview totals equal queue totals.

## Issue 2: Live library game names

Symptom: hundreds of entries render as `App <id>`.

Required:
- Resolve display names for apps that have no `appmanifest_*.acf`: parse `%PROGRAMFILES(X86)%\Steam\appcache\appinfo.vdf` (local file, no network) and use it as a name source; fall back to the Store metadata cache if present; only fall back to `App <id>` when neither has the name.
- Keep the loader tolerant: a missing/corrupt appinfo.vdf must not break the live session, only degrade names.
- Add a fixture-backed test using a synthetic appinfo.vdf snippet (do not copy the real file into the repo).

## Issue 3: Metadata fetch is manual and slow

Symptom: each click fetches only 40 apps and the counter does not advance without a proxy configured; 762 apps is impractical.

Required:
- Add a "fetch until complete or cancelled" mode: loop batches automatically, show progress (done / total), keep the existing Stop button working, and stop cleanly on cancellation or error.
- Keep the existing per-batch cap as a safety limit but allow the loop to run through the whole library.
- Surface the real error text when a batch fails (currently it silently stalls).

## Issue 4 (minor): Steam version detection

- Investigate why the FileVersion probe fails on this machine (Steam installed at `C:\Program Files (x86)\Steam`). Either fix the probe or make the banner state the actual reason (not detected vs. unsupported).

## Constraints

- No behaviour change to the safe-export guard: exporting must still refuse while Steam is running, and must still back up before writing.
- Rust: `cargo test -p steam-core` must stay green (currently 42 + 1 + 2 passing). Add tests for new logic.
- Frontend: `npm run build` must pass.
- Commit per issue with a clear message. Do not push.
- Do not touch `src-tauri/tauri.conf.json` window options.

## Acceptance

1. `cargo test -p steam-core` green, `npm run build` green.
2. AutoCat preview numbers == queue numbers after "加入待导出" (verified by a new test).
3. Live library shows human-readable names for apps present in appinfo.vdf.
4. Metadata "fetch all" loops to completion with progress and can be cancelled.
