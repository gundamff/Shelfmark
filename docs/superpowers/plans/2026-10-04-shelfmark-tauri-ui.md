# Shelfmark Tauri UI Vertical Slice Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans or subagent-driven-development. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a Windows Tauri 2 + React + TypeScript shell that can detect Steam, open a **demo session** (fixtures), run AutoCat dry-run, preview pending changeset, and export only when Steam is closed (writes demo JSON store under AppData — not live Steam LevelDB yet).

**Architecture:** Thin `src-tauri` commands over `steam-core`. React holds view state; session lives in Rust `AppState`. Live Steam collection write deferred until real parsers land; UI still enforces export safety via `SteamPlatform::is_steam_running`.

**Tech Stack:** Tauri 2, React 18, TypeScript, Vite, Vitest; workspace member `src-tauri` depending on `steam-core`.

## Global Constraints

- Spec approved; MIT; no GPL copy.
- Windows-first; no Linux/macOS packaging in this plan.
- Export hard-refuses if Steam is running.
- Demo session uses `fixtures/steam`; label UI clearly as Demo.
- Every task: tests/typecheck green + commit.

## File structure

```text
package.json
vite.config.ts
tsconfig.json
index.html
src/                 # React
src-tauri/           # Tauri app crate
crates/steam-core/   # existing
```

---

### Task 1: Scaffold Tauri 2 + React + Vite

- [ ] Workspace + npm + `src-tauri` compile
- [ ] `npm run build` / `cargo check -p shelfmark` green
- [ ] Commit

### Task 2: Rust commands + AppState

Commands: `detect_steam`, `open_demo_session`, `list_games`, `list_collections`, `pending_changeset`, `run_autocat_genre`, `queue_last_autocat`, `export_session`, `is_steam_running`

- [ ] Unit-testable state helpers where possible
- [ ] Commit

### Task 3: React UI vertical slice

Screens: Steam status, Demo open, game/collection lists, AutoCat Genre button, pending JSON preview, Export button with confirm.

- [ ] Vitest for a pure UI helper (e.g. formatPending)
- [ ] Commit

### Task 4: Docs + gate

- [ ] README run instructions (`npm install`, `npm run tauri dev`)
- [ ] `cargo test -p steam-core` + `npm test` + `npm run build` green
- [ ] Commit; merge when done
