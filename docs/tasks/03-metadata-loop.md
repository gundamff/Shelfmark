# Task 03 — Metadata: fetch until complete, with progress and cancel

Repo: D:\gits\Depressurizer. Scope: metadata fetch only.

## Symptom
- 拉取元数据（每批 40）: each click fetches 40 apps. 762 apps is impractical.
- Without a proxy configured the counter does not move and no error is shown.

## Required
1. Add a continuous mode: loop batches until the whole library is covered, or until cancelled.
2. Show progress: done / total, and the current app being fetched.
3. The existing Stop button must cancel promptly and leave the cache consistent.
4. On a failed batch, surface the real error text instead of stalling silently.
5. Keep the per-batch cap (40) as a safety limit inside the loop.

## Acceptance
- `cargo test -p steam-core` green (add tests for the loop state machine with a fake fetcher)
- `npm run build` green
- one commit: `feat(metadata): fetch all with progress and cancel`
- Do not push. Do not touch src-tauri/tauri.conf.json.
