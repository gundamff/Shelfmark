# Shelfmark UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild Shelfmark’s UI into Settings | Collections + About modal, with Steam version disclosure, metadata progress, extensible JSON AutoCat rule packs, and Web API achievements.

**Architecture:** React shell owns tabs/modals; `steam-core` gains Steam version probe, rule-pack types/loader, metadata provider trait (local appinfo → store), achievement cache; Tauri commands stay thin. User rule packs live under `%AppData%/Shelfmark/rule-packs/`.

**Tech Stack:** Tauri 2, React/TS, Vitest, Rust `steam-core`, existing i18n dictionaries (no i18next).

## Global Constraints

- Brand: **Shelfmark** English only (no 中文名 this cycle).
- About: top-bar **modal** (SaveSmith-like), not a primary tab.
- Pending/export: bottom of **Collections** only.
- Conflict policy: skip / merge / replace.
- Achievements: Steam Web API; API Key in `config.json`; SteamID from session.
- Metadata: local Steam cache first, Store API fill; progress + stop.
- Rule extension: JSON packs; unknown `kind` fails that pack clearly.
- Steam support: always show supported declaration on shell + About; show detected; warn if outside support.
- Export: refuse if Steam running; backup first.
- Windows-first MVP.

**Spec:** `docs/superpowers/specs/2026-10-04-shelfmark-ui-redesign-design.md`

---

### Task 1: Steam version compatibility (core + command)

**Files:**
- Create: `crates/steam-core/src/platform/steam_version.rs`
- Modify: `crates/steam-core/src/platform/mod.rs`, `crates/steam-core/src/lib.rs`
- Modify: `src-tauri/src/lib.rs` (expose in `detect_steam` / new fields)
- Test: unit tests in `steam_version.rs`

**Interfaces:**
- Produces:
  - `pub struct SteamCompatInfo { pub supported_label: String, pub detected_version: Option<String>, pub within_support: Option<bool> }`
  - `pub const SUPPORTED_STEAM_LAYOUT: &str` — human label, e.g. `"Windows Steam · cloud-storage-namespace-1 catalog"`
  - `pub fn probe_steam_version(steam_root: &Path) -> Option<String>` — read `steam.exe` FileVersion via Windows API or parse `Package/steam_client_win32.manifest` / similar; pin **one** method in implementation (prefer `steam.exe` version resource on Windows).
  - Initial support window: document in constant `SUPPORTED_STEAM_NOTE` (update when maintainers re-test), e.g. tested package date/build string; `within_support` may be `None` if detection fails.

- [ ] **Step 1: Write failing test** for label non-empty and probe returns `None` on fake empty dir

```rust
#[test]
fn supported_label_is_non_empty() {
    assert!(!SUPPORTED_STEAM_LAYOUT.is_empty());
}

#[test]
fn probe_missing_root_is_none() {
    assert!(probe_steam_version(Path::new("C:\\no-such-steam-shelfmark")).is_none());
}
```

- [ ] **Step 2: Run** `cargo test -p steam-core steam_version -- --nocapture` — expect FAIL (module missing)

- [ ] **Step 3: Implement** `steam_version.rs` + wire into `SteamInfo` / detect response as:
  - `supportedSteam: string`
  - `detectedSteamVersion: string | null`
  - `steamWithinSupport: boolean | null`

- [ ] **Step 4: Run tests** — PASS

- [ ] **Step 5: Commit** `feat: probe Steam version and declare supported layout`

---

### Task 2: App shell (tabs, About, compat line, i18n)

**Files:**
- Create: `src/components/AppShell.tsx`, `src/components/AboutDialog.tsx`, `src/components/SteamCompatBar.tsx`
- Create: `src/assets/brand/logo-placeholder.svg` (until anime logo exists)
- Modify: `src/App.tsx` (compose shell; remove single-scroll panels), `src/App.css`, `src/i18n/en.ts`, `src/i18n/zh.ts`, `src/i18n/types.ts`
- Test: `src/components/steamCompatBar.test.ts` (pure format helper if extracted)

**Interfaces:**
- Consumes: `detect_steam` fields from Task 1
- Produces: `activeTab: "settings" | "collections"`; About open state; always-visible `SteamCompatBar`

- [ ] **Step 1: Add i18n keys** — `nav.settings`, `nav.collections`, `nav.about`, `compat.supported`, `compat.detected`, `compat.unknown`, `compat.warnOutside`, about.* (tagline, disclaimer, version, close, github)

- [ ] **Step 2: Implement shell** — top bar brand+logo, tab buttons, About button; main renders Settings or Collections placeholder; `SteamCompatBar` under top bar showing supported + detected + warn

- [ ] **Step 3: AboutDialog** — mask+card; version via new command `app_version` → `env!("CARGO_PKG_VERSION")`; show compat strings; close on mask

- [ ] **Step 4:** `npm test` + `npm run build` PASS

- [ ] **Step 5: Commit** `feat: add Settings/Collections shell and About dialog`

---

### Task 3: Settings page (language, session, metadata progress UI)

**Files:**
- Create: `src/pages/SettingsPage.tsx`, `src/components/MetadataProgress.tsx`
- Modify: `src-tauri/src/enrich.rs`, `src-tauri/src/lib.rs` — add progress callback **or** polling command `enrich_status` + `enrich_start` / `enrich_stop` (prefer: start background enrich in mutex state; UI polls `enrich_status` every 500ms)
- Modify: i18n keys for settings sections
- Test: `src/components/metadataProgress.test.ts` — pure `%` math

**Interfaces:**
- `EnrichStatus { running: bool, total: u32, done: u32, fromLocal: u32, fromStore: u32, failed: u32, remaining: u32 }`
- Settings still uses `save_locale`, `open_*_session`, `save_last_user`; add `save_api_key` (writes `AppConfig.steam_web_api_key`)

- [ ] **Step 1: Extend `AppConfig`** with `steam_web_api_key: Option<String>` + serde default; unit test roundtrip

- [ ] **Step 2: Enrich status machinery** — non-blocking batch loop; Stop flag in `AppState`

- [ ] **Step 3: SettingsPage UI** — sections per spec §2; `MetadataProgress` bar = `done/total`

- [ ] **Step 4:** `cargo test -p steam-core --lib config` + `npm test` PASS

- [ ] **Step 5: Commit** `feat: settings page with metadata progress and API key field`

---

### Task 4: Collections page (list, edit, pending bar)

**Files:**
- Create: `src/pages/CollectionsPage.tsx`
- Reuse: existing invoke commands `list_*`, `create_collection`, `add/remove_games_*`, `export_session`, `pending_changeset`
- Test: keep `src/lib/pending.test.ts`; add empty-state copy keys

**Interfaces:**
- Empty state if no session; left list / right members; bottom `PendingBar` with export

- [ ] **Step 1: Build CollectionsPage** wiring current commands (no AutoCat wizard yet)

- [ ] **Step 2: PendingBar** — summary via `formatPendingSummary` + `t()`; export disabled when `steamRunning`

- [ ] **Step 3:** Manual add/remove/create flows smoke via `npm test` + `cargo check -p shelfmark`

- [ ] **Step 4: Commit** `feat: collections tab with manual edit and export bar`

---

### Task 5: Rule-pack schema + loader

**Files:**
- Create: `crates/steam-core/src/autocat/pack.rs`
- Create: `crates/steam-core/src/autocat/schema/rule-pack.schema.json` (also copy/docs under `docs/rule-packs/`)
- Create: `rule-packs/recommended-basic.json`, `rule-packs/genre.json`, … (built-in)
- Create: `%` loader reads `AppData/Shelfmark/rule-packs` + built-in dir from resources
- Modify: `autocat/mod.rs`, Tauri `list_rule_packs`, `validate_rule_pack`
- Test: `pack.rs` tests — unknown kind errors; valid genre pack loads

**Interfaces:**
```rust
pub struct RulePack {
  pub id: String,
  pub name: I18nStr,       // { en, zh }
  pub description: I18nStr,
  pub recommended: bool,
  pub needs_metadata: bool,
  pub needs_achievements: bool,
  pub default_conflict: ConflictPolicy, // skip | merge | replace
  pub rules: Vec<PackRule>, // kind + params JSON
}
pub enum ConflictPolicy { Skip, Merge, Replace }
```

- [ ] **Step 1: Failing test** — parse sample JSON into `RulePack`

- [ ] **Step 2: Implement** loader + schema doc + 2–3 built-in packs mapping to existing `AutoCatRule`

- [ ] **Step 3:** `run_pack(pack, games, meta, achievements, conflict) -> AutoCatResult` adapter

- [ ] **Step 4: Commit** `feat: JSON AutoCat rule packs with loader and schema`

---

### Task 6: AutoCat wizard UI

**Files:**
- Create: `src/components/AutocatWizard.tsx`
- Modify: `CollectionsPage.tsx`, i18n, Tauri `preview_rule_pack` / `queue_rule_pack`

**Interfaces:**
- Steps: strategy → options → conflict → preview → queue pending
- Disable packs when `needs_metadata` and coverage low / `needs_achievements` and no cache

- [ ] **Step 1: Wizard component** with step state machine

- [ ] **Step 2: Wire preview/queue** commands

- [ ] **Step 3:** `npm test` + `cargo check -p shelfmark`

- [ ] **Step 4: Commit** `feat: AutoCat wizard driven by rule packs`

---

### Task 7: Achievements via Steam Web API

**Files:**
- Create: `crates/steam-core/src/achievements/mod.rs` (HTTP **not** in steam-core if policy is HTTP-free — **put HTTP in `src-tauri/src/achievements.rs`**, cache types in steam-core)
- Note: keep `steam-core` HTTP-free; Tauri fetches; steam-core stores `AchievementIndex`
- Modify: config key already in Task 3; Settings “Fetch achievements”
- Packs: `rule-packs/achievements-100.json`, `achievements-0.json`

**Interfaces:**
- `SteamID64` from id3: `id64 = id3 + 76561197960265728`
- `fetch_achievements(key, id64, app_ids) -> cache`
- Predicates: `all_unlocked` | `none_unlocked` | `partial`

- [ ] **Step 1: Unit test** id3→id64 helper

- [ ] **Step 2: Tauri fetch + cache file `achievement-cache.json`

- [ ] **Step 3: Wire** wizard achievement packs

- [ ] **Step 4: Commit** `feat: Steam Web API achievement cache for AutoCat`

---

### Task 8: Local metadata provider (appinfo) + Store fallback

**Files:**
- Create: `crates/steam-core/src/library/appinfo.rs` (parser MVP — best-effort fields)
- Create: `src-tauri/src/meta_providers.rs` — `enrich_from_local` then store
- Modify: enrich status counters `fromLocal` / `fromStore`

- [ ] **Step 1: Test** parser against a **fixture** snippet checked into `fixtures/steam/` (minimal)

- [ ] **Step 2: Integrate** into enrich loop before network

- [ ] **Step 3: Commit** `feat: prefer local Steam appinfo for metadata enrichment`

---

### Task 9: Anime logo asset + polish placeholder

**Files:**
- Create: `src/assets/brand/shelfmark-logo.png` (or svg) — generate/approve with user
- Modify: AboutDialog + top bar `<img>`

- [ ] **Step 1: Add asset**; wire paths

- [ ] **Step 2: Commit** `feat: add Shelfmark brand logo`

---

### Task 10: Verify gate + merge

- [ ] **Step 1:** `cargo test -p steam-core`；`cargo check -p shelfmark`；`npm test`；`npm run build`

- [ ] **Step 2:** Mark plan checkboxes; update README status blurb (Steam support line)

- [ ] **Step 3:** Merge `feature/shelfmark-ui-redesign` → `master`

---

## Spec coverage check

| Spec item | Task |
|---|---|
| Shell tabs + About modal | 2 |
| Steam version disclosure | 1, 2 |
| Settings language/session/API/metadata progress | 3 |
| Collections edit + pending/export | 4 |
| Extensible JSON rule packs | 5, 6 |
| Achievements Web API | 7 |
| Local metadata first | 8 |
| Anime logo | 9 |
| Brand EN only | 2 (copy) |
