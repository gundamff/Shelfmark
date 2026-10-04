# Shelfmark Design Spec

**Date:** 2026-10-04  
**Status:** Approved  
**Working title:** Shelfmark (user may rename; paths and crate names follow the final name at bootstrap)

## 0. Reference material

Clone Depressurizer for **read-only** format/behavior research (GPL-3.0). See `reference/README.md`.  
Do **not** copy upstream source into Shelfmark; reimplement clean-room under MIT.

## 1. Intent

Shelfmark is a **new** desktop app — a spiritual successor to [Depressurizer](https://github.com/julianxhokaxhiu/Depressurizer), not a fork and not a continuation of that codebase or brand.

**Problem:** Large Steam libraries are hard to organize. Users need reliable local tools to manage Steam **Collections**, apply rule-based auto-organization, and write changes back safely.

**Non-goals (MVP):**

- Cross-platform shipping (Linux/macOS installers). Architecture may leave a thin platform trait for later.
- Compatibility with Depressurizer profiles or AutoCat presets.
- Store scraping / HLTB / rich metadata enrichment pipelines.
- Competing with [Repressurizer](https://github.com/Crimsab/Repressurizer) as a fork; this is an independent project.

## 2. Decisions (locked)

| Topic | Decision |
|-------|----------|
| Relation to Depressurizer | Spiritual successor only; greenfield |
| Primary OS | **Windows-first** |
| Stack | **Tauri 2 + Rust + React + TypeScript** |
| Steam model | **Modern Collections** (not legacy Categories) |
| MVP scope | Library import + manual collection edit + AutoCat (near Depressurizer set, **no HLTB**) |
| Legacy import | **None** — new config format only |
| Architecture | **Thick `steam-core` crate + thin Tauri shell + React UI** |

### AutoCat types in MVP

- Genre  
- Tags  
- Year  
- DevPub  
- Manual  
- Flags  
- UserScore  

**Deferred:** HLTB, full store enrichment, Depressurizer preset import.

## 3. Architecture

```text
React UI (TS)
    │ Tauri commands (thin)
tauri-app (Rust)
    │
steam-core (pure Rust, no Tauri dependency)
    │
Local Steam files (read / backup / write-back)
```

### Layer rules

| Layer | May | Must not |
|-------|-----|----------|
| `steam-core` | Steam paths, library/collections parse, backup, AutoCat, app config I/O | UI, Tauri APIs, dialogs; MVP store HTTP |
| `tauri-app` | Call core, expose commands, folder pickers | Embed complex business rules |
| React | Presentation, edit intent, diff confirmation | Touch Steam files directly |

### Windows-first platform boundary

- Trait: `SteamPlatform` (locate install, userdata, “is Steam running?”).
- MVP implementation: `WindowsSteamPlatform` only.
- CI and release artifacts: Windows x64 only for MVP.

## 4. Core modules (`steam-core`)

| Module | Responsibility |
|--------|----------------|
| `platform` | Steam root, `userdata/<id>`, process check |
| `library` | Owned game list from stable local sources |
| `collections` | Read/write modern Collections; `CollectionId` + `AppId` members |
| `backup` | Timestamped copies before write-back; list/restore |
| `autocat` | Rules → `CollectionChangeset` (dry-run by default) |
| `config` | App settings + AutoCat presets (JSON under AppData) |
| `session` | Loaded library, collections, pending changeset |

### Conceptual types

- `AppId` (`u32`), `SteamUserId` (`u64`)
- `Game { app_id, name, ... }` — minimal fields required by AutoCat
- `Collection { id, name, app_ids[] }`
- `CollectionChangeset` — create/rename/delete collections; add/remove members (serializable for UI diff)
- `AutoCatRule` / `AutoCatPreset`

### Source of truth

- **Collections:** Steam local storage → memory → changeset → confirmed export.
- **Library list:** Local Steam user data (usable without Web API).
- **Genre/Tags/Flags/Year/DevPub/UserScore:** Use locally available parseable data only in MVP. Missing fields → skip game + count (`PartialMeta`); do not fail the whole AutoCat run.
- **No second Steam on disk:** AppData holds config/backups/logs only; Steam files change only on import/export paths as designed.

### Config locations (Windows)

- Config: `%AppData%/Shelfmark/config.json`
- Backups: `%AppData%/Shelfmark/backups/<timestamp>/`
- Logs: `%AppData%/Shelfmark/logs/`

(Name segment follows final product name.)

## 5. Main flows

### 5.1 First run / open

Detect Steam → select user → import library + collections → session.  
On failure: clear errors + optional manual path; **no Steam writes**.

### 5.2 Manual collection edits

UI updates in-memory view and pending `CollectionChangeset`. Disk unchanged until export.

### 5.3 AutoCat

Select preset → dry-run → changeset + report (matched / skipped) → UI preview → optional manual tweak → merge into pending. **No auto write to Steam.**

### 5.4 Export (write-back) — safety sequence

1. Refuse if Steam is running (default hard reject).  
2. Require non-empty changeset.  
3. Backup target files.  
4. Apply changeset.  
5. Re-read sanity check.  
6. On success, clear pending; on failure, keep pending and offer restore from backup.

### 5.5 Restore backup

List recent backups → restore Steam files → re-import session.

## 6. Error handling

| Class | Examples | Behavior |
|-------|----------|----------|
| `UserAction` | Steam running, no user selected | Retryable; no write |
| `Io` | Permissions, locks, disk full | Abort write; keep backup if already made |
| `Parse` | Unknown/changed formats | Fail read; **never write after parse failure** |
| `PartialMeta` | Missing genre/tags | Skip + count; AutoCat still succeeds |
| `Internal` | Invariant bugs | Log; never claim successful write |

Logging: rolling files under AppData; structured for backup/export paths. Prefer `app_id` over dumping entire libraries into UI error dialogs.

## 7. Repository layout

```text
/
  crates/steam-core/
  src-tauri/
  src/                    # React
  fixtures/steam/         # Fake Steam trees for tests
  docs/superpowers/specs/
```

## 8. Testing

| Layer | Focus |
|-------|--------|
| `steam-core` | Parse/serialize collections, apply changeset, AutoCat rules, backup layout — against **fixtures**, not the developer’s real Steam dir |
| Safety | Running Steam blocks write; backup failure blocks write; failed write keeps pending |
| Tauri | Thin command mapping / error propagation |
| React | Diff preview, collection edit, AutoCat forms (Vitest + Testing Library) |
| Manual | Import → edit → exit Steam → export → verify in Steam client |

## 9. Release (MVP)

- Windows x64: ship a **portable zip** for MVP; optional installer is post-MVP.
- CI: `cargo test` (core) + frontend typecheck/test + Windows build artifact.
- Out of MVP: Linux/macOS packages, auto-update, code signing, installer.
- License: **MIT** (greenfield; do not copy GPL Depressurizer source).

## 10. MVP definition of done

1. Windows: discover Steam/user; import library and collections.  
2. Manual collection edit + diff preview + backup + write-back.  
3. AutoCat: Genre, Tags, Year, DevPub, Manual, Flags, UserScore (skip on missing metadata).  
4. Persist/load app config; no Depressurizer compatibility.  
5. Core tests green in CI; written manual write-back checklist.

## 11. Risks

| Risk | Mitigation |
|------|------------|
| Steam collection on-disk format changes | Isolate parse/serialize; fixtures; refuse write on parse failure |
| Incomplete local metadata for AutoCat | `PartialMeta` skip + clear UI report; enrichment later |
| Write-back corruption | Backup-first, Steam-must-be-closed, post-write re-read |
| Scope creep toward full metadata client | Enforce non-goals until post-MVP |

## 12. Next step

After this spec is approved: write an implementation plan (`docs/superpowers/plans/...`) via the writing-plans skill, then implement behind that plan.
```
