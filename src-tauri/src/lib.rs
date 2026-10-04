use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use steam_core::{
    load_live_library, load_owned_games, run_preset, AutoCatPreset, AutoCatResult, AutoCatRule,
    BackupManager, Collection, CollectionChangeset, CollectionStore, Game, GameMetadata,
    JsonCollectionStore, MetadataIndex, Session, SteamCloudCatalogStore, SteamPlatform,
};
#[cfg(windows)]
use steam_core::WindowsSteamPlatform;

struct AppState {
    inner: Mutex<Option<SessionBundle>>,
}

#[derive(Clone, Copy)]
enum StoreKind {
    DemoJson,
    SteamCloud,
}

struct SessionBundle {
    session: Session,
    store_path: PathBuf,
    store_kind: StoreKind,
    meta: MetadataIndex,
    last_autocat: Option<AutoCatResult>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SteamInfo {
    root: String,
    users: Vec<String>,
    steam_running: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionSummary {
    mode: String,
    game_count: usize,
    collection_count: usize,
    store_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AutoCatSummary {
    matched_games: u32,
    skipped_missing_meta: u32,
    create_count: usize,
    add_member_ops: usize,
}

fn map_err(e: steam_core::Error) -> String {
    e.to_string()
}

fn fixtures_steam_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures/steam")
}

fn shelfmark_data_dir() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or_else(|| "could not resolve AppData".to_string())?;
    let dir = base.join("Shelfmark");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[tauri::command]
fn detect_steam() -> Result<SteamInfo, String> {
    #[cfg(windows)]
    {
        let platform = WindowsSteamPlatform::new();
        let root = platform.steam_root().map_err(map_err)?;
        let users = platform.list_user_ids3(&root).map_err(map_err)?;
        let steam_running = platform.is_steam_running().map_err(map_err)?;
        return Ok(SteamInfo {
            root: root.display().to_string(),
            users,
            steam_running,
        });
    }
    #[cfg(not(windows))]
    {
        Err("Steam detection is Windows-only in this MVP".into())
    }
}

#[tauri::command]
fn is_steam_running() -> Result<bool, String> {
    #[cfg(windows)]
    {
        WindowsSteamPlatform::new()
            .is_steam_running()
            .map_err(map_err)
    }
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

#[tauri::command]
fn open_demo_session(state: tauri::State<'_, AppState>) -> Result<SessionSummary, String> {
    let fixture = fixtures_steam_root();
    let games = load_owned_games(&fixture, "123456").map_err(map_err)?;

    let data = shelfmark_data_dir()?;
    let demo_dir = data.join("demo");
    std::fs::create_dir_all(demo_dir.join("cloudstorage")).map_err(|e| e.to_string())?;
    let store_path = demo_dir
        .join("cloudstorage")
        .join("cloud-storage-namespace-1.json");
    let fixture_store = fixture
        .join("userdata/123456/config/cloudstorage/cloud-storage-namespace-1.json");
    std::fs::copy(&fixture_store, &store_path).map_err(|e| e.to_string())?;

    let store = JsonCollectionStore::new(store_path.clone());
    let collections = store.load().map_err(map_err)?;

    let mut meta = MetadataIndex::new();
    for g in &games {
        meta.insert(
            g.app_id,
            GameMetadata {
                genres: vec!["Action".into(), "Indie".into()],
                tags: vec!["Classic".into()],
                flags: vec!["Single-player".into()],
                developers: vec!["Valve".into()],
                publishers: vec!["Valve".into()],
                release_year: Some(2004),
                user_score_percent: Some(96),
            },
        );
    }

    let session = Session::new(games.clone(), collections.clone(), vec![store_path.clone()]);
    let summary = SessionSummary {
        mode: "demo".into(),
        game_count: games.len(),
        collection_count: collections.len(),
        store_path: store_path.display().to_string(),
    };

    *state.inner.lock().map_err(|e| e.to_string())? = Some(SessionBundle {
        session,
        store_path,
        store_kind: StoreKind::DemoJson,
        meta,
        last_autocat: None,
    });

    Ok(summary)
}

#[tauri::command]
fn open_live_session(
    user_id3: String,
    state: tauri::State<'_, AppState>,
) -> Result<SessionSummary, String> {
    #[cfg(not(windows))]
    {
        let _ = (user_id3, state);
        return Err("live session is Windows-only in this MVP".into());
    }

    #[cfg(windows)]
    {
        let platform = WindowsSteamPlatform::new();
        let root = platform.steam_root().map_err(map_err)?;
        let store_path = root
            .join("userdata")
            .join(&user_id3)
            .join("config")
            .join("cloudstorage")
            .join("cloud-storage-namespace-1.json");
        if !store_path.is_file() {
            return Err(format!(
                "Steam catalog not found for user {user_id3}: {}",
                store_path.display()
            ));
        }

        let store = SteamCloudCatalogStore::new(store_path.clone());
        let collections = store.load().map_err(map_err)?;
        let collection_ids: Vec<_> = collections
            .iter()
            .flat_map(|c| c.app_ids.iter().copied())
            .collect();
        let games = load_live_library(&root, &user_id3, &collection_ids).map_err(map_err)?;

        // Live AutoCat metadata is empty for now — Genre will skip until enrichment lands.
        let meta = MetadataIndex::new();

        let session = Session::new(games.clone(), collections.clone(), vec![store_path.clone()]);
        let summary = SessionSummary {
            mode: "live".into(),
            game_count: games.len(),
            collection_count: collections.len(),
            store_path: store_path.display().to_string(),
        };

        *state.inner.lock().map_err(|e| e.to_string())? = Some(SessionBundle {
            session,
            store_path,
            store_kind: StoreKind::SteamCloud,
            meta,
            last_autocat: None,
        });

        Ok(summary)
    }
}

#[tauri::command]
fn list_games(state: tauri::State<'_, AppState>) -> Result<Vec<Game>, String> {
    let guard = state.inner.lock().map_err(|e| e.to_string())?;
    let bundle = guard.as_ref().ok_or_else(|| "no session open".to_string())?;
    Ok(bundle.session.games.clone())
}

#[tauri::command]
fn list_collections(state: tauri::State<'_, AppState>) -> Result<Vec<Collection>, String> {
    let guard = state.inner.lock().map_err(|e| e.to_string())?;
    let bundle = guard.as_ref().ok_or_else(|| "no session open".to_string())?;
    Ok(bundle.session.collections.clone())
}

#[tauri::command]
fn pending_changeset(state: tauri::State<'_, AppState>) -> Result<CollectionChangeset, String> {
    let guard = state.inner.lock().map_err(|e| e.to_string())?;
    let bundle = guard.as_ref().ok_or_else(|| "no session open".to_string())?;
    Ok(bundle.session.pending.clone())
}

#[tauri::command]
fn run_autocat_genre(state: tauri::State<'_, AppState>) -> Result<AutoCatSummary, String> {
    let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
    let bundle = guard.as_mut().ok_or_else(|| "no session open".to_string())?;
    let preset = AutoCatPreset {
        name: "demo-genre".into(),
        rules: vec![AutoCatRule::Genre {
            prefix: "g_".into(),
            max_categories: 0,
            remove_existing: false,
        }],
    };
    let result = run_preset(
        &bundle.session.games,
        &bundle.meta,
        &bundle.session.collections,
        &preset,
    )
    .map_err(map_err)?;
    let summary = AutoCatSummary {
        matched_games: result.matched_games,
        skipped_missing_meta: result.skipped_missing_meta,
        create_count: result.changeset.create.len(),
        add_member_ops: result.changeset.add_members.len(),
    };
    bundle.last_autocat = Some(result);
    Ok(summary)
}

#[tauri::command]
fn queue_last_autocat(state: tauri::State<'_, AppState>) -> Result<CollectionChangeset, String> {
    let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
    let bundle = guard.as_mut().ok_or_else(|| "no session open".to_string())?;
    let result = bundle
        .last_autocat
        .take()
        .ok_or_else(|| "run AutoCat first".to_string())?;
    bundle.session.queue_autocat(result);
    Ok(bundle.session.pending.clone())
}

#[tauri::command]
fn export_session(state: tauri::State<'_, AppState>) -> Result<String, String> {
    #[cfg(not(windows))]
    {
        let _ = state;
        return Err("export is Windows-only in this MVP".into());
    }

    #[cfg(windows)]
    {
        let mut guard = state.inner.lock().map_err(|e| e.to_string())?;
        let bundle = guard.as_mut().ok_or_else(|| "no session open".to_string())?;
        let platform = WindowsSteamPlatform::new();
        let data = shelfmark_data_dir()?;
        let backups = BackupManager::new(data.join("backups"));
        match bundle.store_kind {
            StoreKind::DemoJson => {
                let store = JsonCollectionStore::new(bundle.store_path.clone());
                bundle
                    .session
                    .export(&platform, &store, &backups)
                    .map_err(map_err)?;
                bundle.session.collections = store.load().map_err(map_err)?;
            }
            StoreKind::SteamCloud => {
                let store = SteamCloudCatalogStore::new(bundle.store_path.clone());
                bundle
                    .session
                    .export(&platform, &store, &backups)
                    .map_err(map_err)?;
                bundle.session.collections = store.load().map_err(map_err)?;
            }
        }
        Ok(format!("exported to {}", bundle.store_path.display()))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            inner: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            detect_steam,
            is_steam_running,
            open_demo_session,
            open_live_session,
            list_games,
            list_collections,
            pending_changeset,
            run_autocat_genre,
            queue_last_autocat,
            export_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running Shelfmark");
}
