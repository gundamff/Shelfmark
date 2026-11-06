# Shelfmark Live Steam Collections Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: executing-plans / subagent-driven-development.

**Goal:** Read/write real Steam `cloud-storage-namespace-1.json` collections; open a live session from a Steam user id; export with backup + Steam-closed guard. LevelDB optional later if present.

**Architecture:** New `SteamCloudCatalogStore` understands Steam’s `[ [key, element], ... ]` catalog. Library live loader uses `libraryfolders.vdf` + `appmanifest_*.acf` plus app ids seen in collections/localconfig. Tauri gains `open_live_session(user_id3)`.

**Tech Stack:** Existing `steam-core` + Tauri commands. No GPL code copy — reimplement from observed format.

## Global Constraints

- Never write if Steam is running.
- Backup before write (AppData + optional sidecar next to Steam file).
- Preserve non-`user-collections*` catalog entries on save.
- Skip dynamic collections (`filterSpec`).
- Fixture tests use synthetic catalog shaped like Steam’s array format (not the user’s private file).

---

### Task 1: Steam catalog parse/load

- [x] `collections/steam_cloud_store.rs` load → `Vec<Collection>`
- [x] Fixture + tests
- [x] Commit

### Task 2: Steam catalog save

- [x] Save merges collections back into catalog JSON
- [x] Roundtrip test
- [x] Commit

### Task 3: Live library loader

- [x] `library/live.rs` — libraryfolders + appmanifest + union collection app ids
- [x] Tests with fixture tree
- [x] Commit

### Task 4: Tauri live session + UI

- [x] `open_live_session`, export uses real store path
- [x] UI: pick user + Open live session
- [x] Commit + merge
