# Task 02 — Live library: human-readable game names

Repo: D:\gits\Depressurizer. Scope: naming only.

## Symptom
762 apps load, but only the 11 installed ones (with `appmanifest_*.acf`) have names.
Everything else renders as `App 7`, `App 10`, `App 4688680`.

## Data available locally
- `C:\Program Files (x86)\Steam\appcache\appinfo.vdf` contains the full appid -> name table.
- Existing code has an appinfo.vdf parser (see commit 9974c90 "appinfo.vdf parser"); check whether it
  already exposes names and just is not wired into the live loader.

## Required
1. Live loader resolves a display name per app: appmanifest name -> appinfo.vdf name -> Store metadata
   cache -> `App <id>` fallback.
2. Missing or corrupt appinfo.vdf must not break the live session; degrade to the fallback only.
3. Do not ship the real appinfo.vdf into the repo. Add a fixture-backed test with a synthetic snippet.

## Acceptance
- `cargo test -p steam-core` green
- one commit: `feat(live): resolve app names from appinfo.vdf`
- Do not push. Do not touch src-tauri/tauri.conf.json.
