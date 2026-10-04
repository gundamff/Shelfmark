# Shelfmark AutoCat Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Add a dry-run AutoCat engine in `steam-core` that turns rule presets + local game metadata into a `CollectionChangeset` (Genre, Tags, Year, DevPub, Manual, Flags, UserScore; no HLTB).

**Architecture:** Pure Rust module `autocat` over an in-memory `GameMetadata` map. Missing fields â†?skip + count (`PartialMeta` stats), never fail the whole run. Output merges into `CollectionChangeset` (create collections + add members). No Steam writes, no Tauri, no Depressurizer import.

**Tech Stack:** Existing `steam-core` (serde, thiserror). Unit tests with hand-built metadata fixtures.

## Global Constraints

- Spec: `docs/superpowers/specs/2026-10-04-shelfmark-design.md` (Approved).
- MIT greenfield; do not copy GPL from `reference/depressurizer/`.
- Dry-run only: AutoCat produces changeset; Session.export remains the write path.
- MVP types: Genre, Tags, Year, DevPub, Manual, Flags, UserScore. No HLTB.
- Missing metadata â†?skip game for that rule + increment skip counter.
- Every task ends with `cargo test -p steam-core` green + commit.

## File structure

```text
crates/steam-core/src/
  types.rs                 # extend Game metadata fields (optional)
  autocat/
    mod.rs
    meta.rs                # GameMetadata
    rules.rs               # AutoCatRule / AutoCatPreset enums
    engine.rs              # run_preset â†?AutoCatResult
  lib.rs                   # re-exports
```

---

### Task 1: Metadata model + AutoCatResult

**Files:**
- Create: `autocat/mod.rs`, `autocat/meta.rs`, `autocat/rules.rs`
- Modify: `lib.rs`, optionally `types.rs`
- Test: unit tests in `meta.rs` / `rules.rs`

**Interfaces:**
- `GameMetadata { genres, tags, flags, developers, publishers, release_year, user_score_percent }`
- `MetadataIndex = HashMap<AppId, GameMetadata>`
- `AutoCatRule` enum variants for the seven types
- `AutoCatPreset { name, rules: Vec<AutoCatRule> }`
- `AutoCatResult { changeset: CollectionChangeset, matched_games: u32, skipped_missing_meta: u32 }`

- [x] **Step 1:** Add failing test that `GameMetadata::default()` has empty vectors
- [x] **Step 2:** Implement types; tests pass
- [x] **Step 3:** Commit `feat(core): add AutoCat metadata and rule types`

---

### Task 2: Engine skeleton + Genre rule

**Files:**
- Create: `autocat/engine.rs`
- Modify: `autocat/mod.rs`
- Test: genre assigns games into collections named with optional prefix

**Interfaces:**
- `pub fn run_preset(games: &[Game], meta: &MetadataIndex, existing: &[Collection], preset: &AutoCatPreset) -> Result<AutoCatResult>`
- Genre rule: `AutoCatRule::Genre { prefix: String, max_categories: u32 /* 0 = unlimited */, remove_existing: bool }`
- For each game with genres: ensure collection `prefix+genre` exists (create), add app_id; if no genres in meta â†?skip++

- [x] **Step 1:** RED test â€?two games, Action/Indie genres â†?changeset creates/adds
- [x] **Step 2:** Implement `run_preset` + Genre
- [x] **Step 3:** GREEN + commit `feat(core): AutoCat Genre rule dry-run`

---

### Task 3: Tags + Year + Flags

**Files:** Modify `rules.rs`, `engine.rs`

**Behavior:**
- Tags: `{ prefix, max_tags_per_game, included: Option<Vec<String>> }` â€?if `included` is Some, only those tags; else all tags up to max
- Year: `{ mode: Individual | Decade, prefix }` â€?collection name from year or `2010s`
- Flags: `{ prefix, included: Vec<String> }` â€?only listed flags

- [x] **Step 1:** RED tests for each
- [x] **Step 2:** Implement
- [x] **Step 3:** Commit `feat(core): AutoCat Tags, Year, Flags rules`

---

### Task 4: DevPub + UserScore + Manual

**Files:** Modify `rules.rs`, `engine.rs`

**Behavior:**
- DevPub: `{ prefix, use_developers: bool, use_publishers: bool }`
- UserScore: `{ rules: Vec<{ name, min_inclusive, max_inclusive }> }` â€?first match wins; score missing â†?skip
- Manual: `{ add: Vec<String> /* collection names */, remove: Vec<String>, app_ids: Vec<AppId> }` â€?only listed apps (or empty app_ids = all games in `games`)

- [x] **Step 1:** RED tests
- [x] **Step 2:** Implement
- [x] **Step 3:** Commit `feat(core): AutoCat DevPub, UserScore, Manual rules`

---

### Task 5: Wire Session helper + docs gate

**Files:**
- Modify: `session/mod.rs` â€?`Session::queue_autocat(&mut self, result: AutoCatResult)` merges changeset into pending (create + add_members only for MVP merge)
- Modify: `README.md` â€?how to run AutoCat tests
- Test: session merges pending after autocat

- [x] **Step 1:** RED merge test
- [x] **Step 2:** Implement merge helper
- [x] **Step 3:** `cargo test -p steam-core` all green; commit `feat(core): queue AutoCat result into session pending`

---

## Spec coverage

| Spec | Task |
|------|------|
| AutoCat Genre/Tags/Year/DevPub/Manual/Flags/UserScore | 2â€? |
| Dry-run â†?changeset | 2 |
| PartialMeta skip | 2â€? |
| No HLTB / no Depressurizer import | all |
| Session pending | 5 |
