# Shelfmark Core Collections Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a testable Rust `steam-core` that can discover Steam on Windows, import a library snapshot, read/write modern Steam Collections via a fixture-backed backend, and export only after backup + “Steam not running” checks.

**Architecture:** Thick `steam-core` crate (no Tauri). Domain types + `CollectionChangeset` + `SteamPlatform` trait with Windows impl. Collections I/O behind `CollectionStore` trait; MVP implements JSON cloud-storage namespace first (fixture-friendly), then LevelDB-compatible read/write matching upstream behavior. No UI in this plan.

**Tech Stack:** Rust 2021, Cargo workspace, `serde`/`serde_json`, `thiserror`, `directories` (optional), `sysinfo` or Win32 process check, `rusty-leveldb` or `leveldb` crate only when Task 6 lands; unit tests with `tempfile` + `fixtures/steam/`.

## Global Constraints

- Spec: `docs/superpowers/specs/2026-10-04-shelfmark-design.md` (Approved).
- Spiritual successor only; MIT; **no copying** GPL code from `reference/depressurizer/`.
- Windows-first; `SteamPlatform` trait for future OS.
- Steam **Collections** model; write-back only after backup + Steam closed (hard reject).
- No Depressurizer profile import; no AutoCat; no Tauri/React in this plan.
- Reference behavior (read-only): `reference/README.md` and upstream files listed there.
- Every task ends with tests green + a commit.

## Scope split (why this plan is not the whole MVP)

| Plan | Delivers |
|------|----------|
| **This plan** | `steam-core`: platform, library import, collections R/W, backup, safe export |
| **Follow-up** | AutoCat engine (Genre/Tags/Year/DevPub/Manual/Flags/UserScore) |
| **Follow-up** | Tauri 2 + React UI + Windows portable zip |

---

## File structure (locked for this plan)

```text
/
  Cargo.toml                          # workspace
  LICENSE                             # MIT
  README.md                           # project blurb + reference note
  crates/steam-core/
    Cargo.toml
    src/
      lib.rs                          # re-exports
      error.rs                        # Error / ErrorKind
      types.rs                        # AppId, Game, Collection, Changeset
      platform/
        mod.rs
        traits.rs                     # SteamPlatform
        windows.rs                    # WindowsSteamPlatform
      library/
        mod.rs                        # load owned games from fixture/local paths
      collections/
        mod.rs
        types.rs                      # raw collection value shapes (serde)
        changeset.rs                  # apply changeset in memory
        store.rs                      # CollectionStore trait
        json_store.rs                 # cloud-storage-namespace-1.json
        leveldb_store.rs              # Task 6
      backup/
        mod.rs
      session/
        mod.rs                        # Session: import / pending / export
    tests/
      collections_json_roundtrip.rs
      export_safety.rs
  fixtures/steam/
    userdata/
      123456/
        config/
          cloudstorage/
            cloud-storage-namespace-1.json
          localconfig.vdf             # minimal stub if needed
    README.md                         # how fixtures were derived (synthetic)
```

---

### Task 1: Workspace bootstrap

**Files:**
- Create: `Cargo.toml`, `LICENSE`, `README.md`, `crates/steam-core/Cargo.toml`, `crates/steam-core/src/lib.rs`, `crates/steam-core/src/error.rs`
- Modify: none
- Test: `crates/steam-core` compiles; `cargo test -p steam-core` runs empty suite

**Interfaces:**
- Consumes: nothing
- Produces: crate `steam-core` with `Error` / `Result<T>`

- [ ] **Step 1: Create workspace and crate skeleton**

Root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["crates/steam-core"]

[workspace.package]
edition = "2021"
license = "MIT"
version = "0.1.0"
```

`crates/steam-core/Cargo.toml`:

```toml
[package]
name = "steam-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tempfile = "3"

[dev-dependencies]
pretty_assertions = "1"
```

`crates/steam-core/src/error.rs`:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("user action required: {0}")]
    UserAction(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("partial metadata: {0}")]
    PartialMeta(String),
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

`crates/steam-core/src/lib.rs`:

```rust
pub mod error;

pub use error::{Error, Result};

pub fn crate_name() -> &'static str {
    "steam-core"
}
```

- [ ] **Step 2: Add MIT LICENSE and minimal README**

README must state: Shelfmark, MIT, Windows-first, reference clone instructions pointing at `reference/README.md`, no GPL code copied.

- [ ] **Step 3: Run tests**

Run: `cargo test -p steam-core`
Expected: PASS (0 or trivial tests)

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml LICENSE README.md crates/steam-core .gitignore reference/README.md
git commit -m "chore: bootstrap steam-core workspace"
```

---

### Task 2: Domain types and in-memory changeset

**Files:**
- Create: `crates/steam-core/src/types.rs`, `crates/steam-core/src/collections/mod.rs`, `crates/steam-core/src/collections/changeset.rs`
- Modify: `crates/steam-core/src/lib.rs`
- Test: unit tests inside `changeset.rs` or `tests/changeset_apply.rs`

**Interfaces:**
- Consumes: `Error` / `Result`
- Produces:
  - `pub type AppId = u32;`
  - `pub struct Game { pub app_id: AppId, pub name: String }`
  - `pub struct Collection { pub id: String, pub name: String, pub app_ids: Vec<AppId> }`
  - `pub struct CollectionChangeset { pub create: Vec<Collection>, pub rename: Vec<(String /*id*/, String /*new_name*/)>, pub delete: Vec<String>, pub add_members: Vec<(String, Vec<AppId>)>, pub remove_members: Vec<(String, Vec<AppId>)> }`
  - `pub fn apply_changeset(collections: &mut Vec<Collection>, cs: &CollectionChangeset) -> Result<()>`

- [ ] **Step 1: Write failing test for apply_changeset**

```rust
#[test]
fn adds_members_to_existing_collection() {
    use steam_core::collections::changeset::{apply_changeset, CollectionChangeset};
    use steam_core::types::{AppId, Collection};

    let mut cols = vec![Collection {
        id: "c1".into(),
        name: "Indie".into(),
        app_ids: vec![10],
    }];
    let cs = CollectionChangeset {
        create: vec![],
        rename: vec![],
        delete: vec![],
        add_members: vec![("c1".into(), vec![20, 30])],
        remove_members: vec![],
    };
    apply_changeset(&mut cols, &cs).unwrap();
    assert_eq!(cols[0].app_ids, vec![10u32, 20, 30]);
}
```

- [ ] **Step 2: Run test — expect FAIL**

Run: `cargo test -p steam-core adds_members_to_existing_collection -- --nocapture`
Expected: FAIL (module/type missing)

- [ ] **Step 3: Implement types + apply_changeset**

Rules for `apply_changeset`:
- `create`: push if id absent; error if id exists
- `rename`: update name by id; error if missing
- `delete`: remove by id (idempotent OK)
- `add_members`: append unique app_ids
- `remove_members`: retain those not listed
- Order: delete → create → rename → remove_members → add_members

- [ ] **Step 4: Run test — expect PASS**

Run: `cargo test -p steam-core`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/steam-core
git commit -m "feat(core): add collection domain types and changeset apply"
```

---

### Task 3: Windows SteamPlatform (detect path + users + running)

**Files:**
- Create: `crates/steam-core/src/platform/mod.rs`, `traits.rs`, `windows.rs`
- Modify: `crates/steam-core/Cargo.toml` (add `winreg` with target cfg), `lib.rs`
- Test: unit tests with injectable paths; optional `#[cfg(windows)]` integration ignored by default

**Interfaces:**
- Consumes: `Error`
- Produces:
  - `pub trait SteamPlatform { fn steam_root(&self) -> Result<PathBuf>; fn list_user_ids3(&self, steam_root: &Path) -> Result<Vec<String>>; fn is_steam_running(&self) -> Result<bool>; }`
  - `pub struct WindowsSteamPlatform;`
  - `pub struct MockSteamPlatform { pub root: PathBuf, pub users: Vec<String>, pub running: bool }` for tests

Reference (behavior only): registry / install path ideas in upstream `DlgSteamPath` / settings — do not copy code. Typical Windows install: `C:\Program Files (x86)\Steam`; userdata: `{root}\userdata\{id3}\`.

- [ ] **Step 1: Failing test using MockSteamPlatform**

```rust
#[test]
fn mock_lists_users_from_userdata_dirs() {
    // create temp dir userdata/111 and userdata/222
    // MockSteamPlatform { root: temp, ... } or a small helper scan_userdata
}
```

Implement `list_user_ids3` as scanning `{steam_root}/userdata/*` numeric directory names — testable without real Steam.

- [ ] **Step 2: Run — FAIL then implement trait + mock + windows registry lookup**

Windows `steam_root`: try registry `HKCU\Software\Valve\Steam\SteamPath` then `HKLM\SOFTWARE\WOW6432Node\Valve\Steam\InstallPath`, normalize to `PathBuf`.

`is_steam_running`: check for process name `steam.exe` (sysinfo crate).

- [ ] **Step 3: `cargo test -p steam-core` PASS**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(core): add SteamPlatform trait and Windows detection"
```

---

### Task 4: Minimal library import from fixture

**Files:**
- Create: `crates/steam-core/src/library/mod.rs`, `fixtures/steam/README.md`, synthetic owned-games fixture
- Modify: `lib.rs`
- Test: `library::load_games_from_fixture`-style test

**Interfaces:**
- Consumes: `Game`, `AppId`, paths
- Produces: `pub fn load_owned_games(steam_root: &Path, user_id3: &str) -> Result<Vec<Game>>`

MVP strategy (keep simple):
- Prefer parsing a **Shelfmark fixture JSON** `fixtures/steam/userdata/{id}/shelfmark/owned_games.json` for unit tests.
- Additionally attempt reading names from a minimal stub; full `localconfig.vdf` / licensecache parity is best-effort and can grow later.
- Document in fixture README that production will expand parsers; tests must not require a live Steam install.

Reference for future expansion only: `GameList.cs` licensecache / localconfig paths in `Constants.cs`.

- [ ] **Step 1: Write fixture + failing test that loads 2 games**

```json
[
  {"app_id": 10, "name": "Counter-Strike"},
  {"app_id": 220, "name": "Half-Life 2"}
]
```

- [ ] **Step 2: Implement loader; tests PASS**

- [ ] **Step 3: Commit**

```bash
git commit -m "feat(core): load owned games from userdata fixture"
```

---

### Task 5: JSON CollectionStore (cloud-storage-namespace-1)

**Files:**
- Create: `collections/store.rs`, `collections/types.rs`, `collections/json_store.rs`, `fixtures/steam/userdata/123456/config/cloudstorage/cloud-storage-namespace-1.json`, `tests/collections_json_roundtrip.rs`
- Modify: `collections/mod.rs`
- Test: read → mutate via changeset → write → read back

**Interfaces:**
- Consumes: `Collection`, `CollectionChangeset`, `apply_changeset`
- Produces:
  - `pub trait CollectionStore { fn load(&self) -> Result<Vec<Collection>>; fn save(&self, collections: &[Collection]) -> Result<()>; }`
  - `pub struct JsonCollectionStore { pub path: PathBuf }`

**Format notes (from upstream behavior — reimplement, do not copy):**
- File: `{steam}/userdata/{id3}/config/cloudstorage/cloud-storage-namespace-1.json`
- Logical keys: `user-collections.<id>`, `user-collections.hidden`, `user-collections.favorite`
- Value JSON includes at least `name`, `added` (array of app ids). Ignore dynamic `filterSpec` collections on import (skip), matching upstream intent in `GameList.ImportSteamLevelDB`.

Fixture may be a simplified JSON array or object that `JsonCollectionStore` understands; if upstream file shape is a large wrapped structure, define a **clear Shelfmark-normalized on-disk schema for the JSON backend** documented in `collections/json_store.rs` module docs, and map to/from `Collection`. Goal of this task: stable roundtrip for static user collections.

- [ ] **Step 1: Add synthetic fixture with two collections**

- [ ] **Step 2: Failing roundtrip test**

```rust
#[test]
fn json_store_roundtrip_after_changeset() {
    // copy fixture to temp path
    // load, apply add_members, save, load again, assert
}
```

- [ ] **Step 3: Implement `JsonCollectionStore` until PASS**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(core): JSON collection store with fixture roundtrip"
```

---

### Task 6: LevelDB CollectionStore (Windows path parity)

**Files:**
- Create: `collections/leveldb_store.rs`
- Modify: `collections/mod.rs`, `Cargo.toml` (leveldb dependency)
- Test: skip on CI without LevelDB libs if needed; provide `#[ignore]` integration test + unit test for key prefix / payload framing using bytes fixtures

**Interfaces:**
- Produces: `pub struct LevelDbCollectionStore { pub database_dir: PathBuf, pub user_id3: String }` implementing `CollectionStore`
- Key prefix behavior must match upstream *observable* format:
  - DB dir default: `%LocalAppData%\Steam\htmlcache\Local Storage\leveldb`
  - Key: `_https://steamloopback.host` + `\0` + `\x01` + `U{id3}-cloud-storage-namespace-1`
  - Value: encoding tag byte + JSON text of catalog array (see `SteamLevelDB.setParsedCatalog`)

**Safety:** Never write to the developer’s real LevelDB in unit tests. Use temp dirs + byte fixtures.

- [ ] **Step 1: Byte-level fixture test for decode/encode of catalog payload**

- [ ] **Step 2: Implement encode/decode helpers + store**

- [ ] **Step 3: Tests PASS (ignored tests documented in module docs)**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(core): LevelDB collection store with framed catalog codec"
```

---

### Task 7: Backup + Session export safety

**Files:**
- Create: `backup/mod.rs`, `session/mod.rs`, `tests/export_safety.rs`
- Modify: `lib.rs`
- Test: export refused when running; backup created before save; failed save keeps pending

**Interfaces:**
- Consumes: `SteamPlatform`, `CollectionStore`, `CollectionChangeset`
- Produces:
  - `pub struct BackupManager { pub root: PathBuf }` with `backup_files(paths: &[PathBuf]) -> Result<PathBuf>` returning timestamped dir
  - `pub struct Session { pub games: Vec<Game>, pub collections: Vec<Collection>, pub pending: CollectionChangeset, ... }`
  - `impl Session { pub fn export(&mut self, platform: &dyn SteamPlatform, store: &dyn CollectionStore, backups: &BackupManager) -> Result<()>; }`

Export algorithm (spec §5.4):
1. If `platform.is_steam_running()?` → `Err(Error::UserAction("close Steam before export".into()))`
2. If pending empty → `Err(Error::UserAction(...))` or no-op error
3. Backup store’s underlying file(s)
4. `apply_changeset` on in-memory collections
5. `store.save`
6. Clear pending on success

- [ ] **Step 1: Write `export_refuses_when_steam_running` with MockSteamPlatform `{ running: true }`**

- [ ] **Step 2: FAIL → implement Session/Backup → PASS**

- [ ] **Step 3: Write `export_writes_backup_then_store` with mock store recording calls**

- [ ] **Step 4: PASS + commit**

```bash
git commit -m "feat(core): session export with backup and steam-running guard"
```

---

### Task 8: Plan verification gate

**Files:**
- Modify: `README.md` (how to run `cargo test -p steam-core`)
- Test: full suite

- [ ] **Step 1: Run** `cargo test -p steam-core` **— all non-ignored tests PASS**

- [ ] **Step 2: Manual checklist (document in README)**

1. (Optional on a Windows Steam machine) Point a small bin/example at real paths in read-only mode — **read only** in this plan; live write is deferred until UI plan + explicit user confirmation.

- [ ] **Step 3: Commit** docs updates if any

```bash
git commit -m "docs: document steam-core test and safety gate"
```

---

## Spec coverage (self-review)

| Spec item | Task |
|-----------|------|
| Thick steam-core | 1–8 |
| SteamPlatform / Windows-first | 3 |
| Collections model + changeset | 2, 5, 6 |
| Backup + Steam closed + export | 7 |
| Library import | 4 |
| Fixtures / no real Steam in CI | 4–7 |
| AutoCat | **Out of plan** (follow-up) |
| Tauri/React | **Out of plan** (follow-up) |
| MIT / no GPL copy | 1 + reference/README |

## Placeholder scan

None intentional. LevelDB crate choice may be adjusted at Task 6 if the first crate fails to build on Windows — swap only the dependency, keep the `CollectionStore` interface.
```
