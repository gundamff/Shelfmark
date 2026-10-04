# Shelfmark Metadata Enrichment + Library Search Plan

> **For agentic workers:** executing-plans.

**Goal:** Enrich `MetadataIndex` for live sessions via Steam Store `appdetails` (Tauri layer, cached under AppData), so AutoCat Genre/Tags/etc. work; add client-side game search filter in the UI.

**Architecture:** Keep `steam-core` HTTP-free. New module in `src-tauri` fetches/parses store JSON → `GameMetadata`. Persist cache `%AppData%/Shelfmark/meta-cache.json`. UI: Enrich button + search box.

## Tasks

- [ ] Task 1: meta cache load/save + store appdetails parser (unit-testable pure fn)
- [ ] Task 2: `enrich_metadata` command (batch, skip cached, polite delay)
- [ ] Task 3: UI search filter + Enrich button
- [ ] Task 4: verify + merge
