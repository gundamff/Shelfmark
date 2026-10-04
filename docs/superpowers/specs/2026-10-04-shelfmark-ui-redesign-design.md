# Shelfmark UI Redesign + Extensible AutoCat Design

**Status:** approved (brainstorm 2026-10-04)  
**Brand:** English name **Shelfmark** only for now (no 中文名). Anime-style mascot logo planned (top bar small + About large).  
**About UX reference:** SaveSmith `AboutDialog` (modal, not a primary tab).

## Goals

- Clean, simple, easy IA: **Settings** | **Collections (当前游戏分组)**; **About** via top-bar dialog.
- Settings: language, Steam session, Web API key (achievements), metadata loader with **progress**.
- Collections: list + edit members; **AutoCat wizard**; pending + export at bottom.
- AutoCat strategies are **data-driven rule packs** (JSON) so users/AI can extend without rebuilding the app.
- Metadata: **prefer local Steam caches**, fill gaps via Store API.
- Achievements: **Steam Web API**; Key in config; SteamID from current session user.
- **Supported Steam version is disclosed clearly** on startup / main shell (and repeated in About).

## Non-goals (this redesign cycle)

- Chinese dual brand name (deferred).
- HLTB or other third-party stores (unless later added as a new engine `kind` + pack).
- In-app undo stack beyond “don’t export yet”.
- Shipping a full visual redesign system unrelated to the IA above.

---

## §1 Information architecture

### Shell

- Top bar left: **Shelfmark** (+ optional tiny logo).
- Top bar nav: **Settings** | **Collections**.
- Top bar right: **About** (opens modal).
- Optional one-line session chip: mode + user (so user is not lost outside Settings).
- No session: Settings + About work; Collections shows empty state → “open a session in Settings”.
- **Steam compatibility line (always visible on main shell):**  
  - Declares the Steam client / data layout Shelfmark supports (human-readable, versioned with the app — e.g. Windows Steam using `cloud-storage-namespace-1.json`, tested against Steam package/build `X`).  
  - After Detect Steam (or on launch if path known): show **detected** Steam version beside it.  
  - If detected is outside the declared support window → warning style (still usable, but explicit).  
  - Same text appears in the About modal.

### About modal (SaveSmith-like)

- Logo, brand, short tagline  
- Version from host (`CARGO_PKG_VERSION`)  
- **Supported Steam version** (same declaration as shell) + detected version when available  
- Short disclaimer (unofficial; exit Steam before export; use at own risk)  
- GitHub link if configured; Close; Donate optional/hidden if unset  

### Visual tone

- Clean, airy, single main column; not a dense dashboard of equal panels.  
- Keep dark green family; primary actions (open session, export, start AutoCat) visually stronger than secondary.

---

## §2 Settings

Sections top → bottom:

1. **Language** — system / en / zh (existing `config.locale`).
2. **Steam session** — detect Steam; user select; open live / demo; show summary (mode, counts, catalog path); running indicator.
3. **Steam Web API** — API Key (masked, saved in `config.json`); SteamID from **current session** only; status configured/missing.
4. **Game metadata** — coverage bar `with_meta / total`; Continue / Stop; batch size (default 40); live counters (cache hit / fetched / failed / remaining). Disabled without session.
5. **Achievements data** (may share the metadata card or sit beside it) — fetch/refresh using Web API into local cache; progress UX similar to metadata.

Not in Settings: collection editing, AutoCat wizard, export.

---

## §3 Collections + AutoCat wizard

### Main view

- **Left:** collections (name + count); create; select; delete → pending.  
- **Right:** members of selection; library search; add/remove selected.  
- Empty states for no session / no collections.

### AutoCat wizard

Entry: **自动分组…**

1. **Strategy:** Recommended pack (multi-kind preset) **or** pick a single rule pack / dimension.  
2. **Options** for that pack (prefixes, achievement bands, etc.; advanced collapsed).  
3. **Conflict policy (required):**  
   - Skip same-named collections  
   - Merge / append members  
   - Replace members  
4. **Preview** (creates / member ops / skips) → **Write to pending** (never direct Steam write).

### Bottom bar

- Pending summary + **Export to Steam** (disabled if Steam running, with reason).  
- Success refreshes lists; failure keeps pending.

### Soft gates

- Low metadata coverage when pack needs meta → CTA to Settings.  
- Achievement packs without Key/cache → disabled + CTA.

---

## §4 Data, extensibility, metadata providers, phasing

### On-disk layout (`%AppData%/Shelfmark/`)

| Artifact | Purpose |
|---|---|
| `config.json` | locale, last user, API key, steam path override |
| `meta-cache.json` | merged GameMetadata by appid |
| `achievement-cache.json` | achievement progress by SteamID + appid |
| `rule-packs/*.json` | **user/AI extensible** AutoCat packs |
| `backups/` | pre-export backups |
| `demo/` | demo session store |

Built-in packs also ship in app resources; **user packs override/extend** (user same id wins).

### Extensible rule packs

**Engine kinds** (require app update to add new kinds):  
`genre`, `tags`, `devpub`, `year`, `flags`, `user_score`, `achievements`, `manual`, …

**Rule pack JSON** (user-extensible): id, name/description zh+en, kinds + params, default conflict policy, flags (`recommended`, `needs_metadata`, `needs_achievements`).

- Document **JSON Schema + examples** in repo for AI/user authoring.  
- Load-time validation; bad pack surfaces error, others still work.  
- Unknown `kind` → clear “update Shelfmark” message (no silent ignore).  
- Cap file size / rule count for safety.

Users/AI can add strategies **without modifying the main program** as long as they only compose existing kinds.

### Metadata provider chain

Steam **does** cache rich-ish app data locally (notably under `appcache/` such as `appinfo.vdf`), plus thin `appmanifest_*.acf` (name/install). Formats are proprietary/binary and version-sensitive.

**Design:**

1. **Local Steam provider** — parse what we can (name always from manifests/library; genres/categories/etc. from appinfo when parser supports).  
2. **Store API provider** — fill missing fields (existing enrich path) with batching, progress, stop.  
3. Unified write into `meta-cache.json` / in-session `MetadataIndex`.

UI progress aggregates both sources (“from local cache / from store / failed”).

### Achievements

- Web API with saved Key; identity = session user.  
- Cache before rule evaluation.  
- Packs declare achievement predicates (e.g. 100%, 0%, partial).

### Risks

- API Key at rest in config — document; mask in UI.  
- Rate limits — batch + stop.  
- Local appinfo parser breakage on Steam updates — Store fallback must remain.  
- Export safety unchanged: backup + refuse if Steam running.

### Phased delivery

1. Shell (nav, Settings/Collections, About, logo placeholder/asset).  
2. Settings session + metadata progress (local-first provider stub OK if parser not ready).  
3. Collections manual edit + pending/export bar.  
4. Rule-pack loader + schema + migrate built-in Genre/etc. to JSON.  
5. AutoCat wizard bound to packs + conflict policy.  
6. Web API achievements + achievement packs.  
7. Harden local appinfo parser as primary metadata source.

---

## Decisions log

| Topic | Choice |
|---|---|
| Scope | Full (incl. achievements) |
| Achievements source | Steam Web API |
| API credentials | Key in config; SteamID from session |
| AutoCat UX | Recommended pack vs per-pack wizard + preview |
| About | Top-bar modal (not a tab) |
| Pending/export | Bottom of Collections |
| Conflict policy | Skip / merge / replace |
| Brand | Shelfmark EN only for now; anime logo later |
| Rule extension | JSON rule packs in AppData |
| Metadata | Local Steam cache first, Store API fill |
| Steam version UX | Always show supported range on shell + About; show detected when known; warn if outside support |

## Open implementation details (plan, not product ambiguity)

- Exact SteamID3 → SteamID64 conversion method.  
- First version of appinfo parser field coverage.  
- Canonical rule-pack JSON schema field names.  
- Logo asset production pipeline.  
- Concrete initial “supported Steam” string and how to read FileVersion / package from install (plan pins one method).
