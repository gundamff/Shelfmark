# Task 01 — AutoCat: preview result must reach the export queue

Repo: D:\gits\Depressurizer. Scope: this issue only. Do not touch other modules.

## Symptom (measured on the real machine)
- Live session, 762 apps, metadata for 120 apps
- AutoCat "genre" pack preview prints: `匹配 120，跳过 642，新建 18，添加操作 0`
- After clicking 加入待导出, queue JSON is empty:
  `{"create":0,"rename":0,"delete":0,"add_members":0,"remove_members":0}`

## What is already in the working tree
A previous session added `crates/steam-core/tests/autocat_plan.rs` (6 tests, passing) and changes in
`src/autocat/engine.rs` (+76). Treat that as a starting point, verify it is correct, keep or fix it.

## Required
1. Plan generation must emit `add_members` for matched games, and a new collection must be created
   together with its members. Never create an empty collection.
2. Preview counters and queue counters must be consistent by construction. Whatever preview prints
   must equal what the queue contains after 加入待导出.
3. Respect coverage policies: skip-same-name / merge-append / replace-members.
4. Keep the 6 existing tests green; add a test that asserts preview totals == queue totals end to end
   (same code path the Tauri command uses, not a re-implementation).

## Acceptance
- `cargo test -p steam-core` green
- `git commit` one commit, message: `fix(autocat): emit add ops so preview matches export queue`
- Do not push. Do not touch src-tauri/tauri.conf.json.
