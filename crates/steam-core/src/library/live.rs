use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::types::{AppId, Game};
use crate::Result;

/// Load installed games from Steam library folders + appmanifest files.
pub fn load_installed_games(steam_root: &Path) -> Result<Vec<Game>> {
    let mut by_id: HashMap<AppId, String> = HashMap::new();

    let mut roots = vec![steam_root.to_path_buf()];
    roots.extend(parse_library_folders(steam_root)?);

    for root in roots {
        let steamapps = if root.join("steamapps").is_dir() {
            root.join("steamapps")
        } else if root.ends_with("steamapps") {
            root
        } else {
            root.join("steamapps")
        };
        if !steamapps.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(&steamapps)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with("appmanifest_") || !name.ends_with(".acf") {
                continue;
            }
            if let Some(game) = parse_appmanifest(&entry.path())? {
                by_id.insert(game.app_id, game.name);
            }
        }
    }

    let mut games: Vec<Game> = by_id
        .into_iter()
        .map(|(app_id, name)| Game { app_id, name })
        .collect();
    games.sort_by_key(|g| g.app_id);
    Ok(games)
}

/// Union installed games with any app ids (e.g. from collections / localconfig).
pub fn merge_library_with_app_ids(installed: Vec<Game>, extra_ids: &[AppId]) -> Vec<Game> {
    let mut by_id: HashMap<AppId, String> = installed
        .into_iter()
        .map(|g| (g.app_id, g.name))
        .collect();
    for id in extra_ids {
        by_id
            .entry(*id)
            .or_insert_with(|| format!("App {id}"));
    }
    let mut games: Vec<Game> = by_id
        .into_iter()
        .map(|(app_id, name)| Game { app_id, name })
        .collect();
    games.sort_by_key(|g| g.app_id);
    games
}

/// Best-effort live library for a user: installed manifests ∪ collection members ∪ localconfig apps.
pub fn load_live_library(
    steam_root: &Path,
    user_id3: &str,
    collection_app_ids: &[AppId],
) -> Result<Vec<Game>> {
    let installed = load_installed_games(steam_root).unwrap_or_default();
    let mut extras: HashSet<AppId> = collection_app_ids.iter().copied().collect();
    if let Ok(ids) = localconfig_app_ids(steam_root, user_id3) {
        extras.extend(ids);
    }
    let extra_vec: Vec<AppId> = extras.into_iter().collect();
    Ok(merge_library_with_app_ids(installed, &extra_vec))
}

fn parse_library_folders(steam_root: &Path) -> Result<Vec<PathBuf>> {
    let path = steam_root.join("steamapps").join("libraryfolders.vdf");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)?;
    let mut roots = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        // "path"		"D:\\SteamLibrary"
        if let Some(rest) = line.strip_prefix("\"path\"") {
            let rest = rest.trim();
            if let Some(p) = unquote(rest) {
                let normalized = p.replace("\\\\", "\\");
                roots.push(PathBuf::from(normalized));
            }
        }
    }
    Ok(roots)
}

fn parse_appmanifest(path: &Path) -> Result<Option<Game>> {
    let text = std::fs::read_to_string(path)?;
    let mut app_id: Option<AppId> = None;
    let mut name: Option<String> = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("\"appid\"") {
            if let Some(v) = unquote(rest.trim()) {
                app_id = v.parse().ok();
            }
        } else if let Some(rest) = line.strip_prefix("\"name\"") {
            name = unquote(rest.trim());
        }
    }
    match (app_id, name) {
        (Some(app_id), Some(name)) => Ok(Some(Game { app_id, name })),
        _ => Ok(None),
    }
}

fn localconfig_app_ids(steam_root: &Path, user_id3: &str) -> Result<Vec<AppId>> {
    let path = steam_root
        .join("userdata")
        .join(user_id3)
        .join("config")
        .join("localconfig.vdf");
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(&path)?;
    // Heuristic line/brace parser: collect numeric keys directly inside any
    // `"apps"` block. Nested per-app blocks are skipped by depth tracking.
    let mut ids = HashSet::new();
    let mut depth: i32 = 0;
    let mut apps_depth: Option<i32> = None;
    for line in text.lines() {
        let t = line.trim();
        if apps_depth.is_none() && t.starts_with("\"apps\"") {
            apps_depth = Some(depth);
        } else if let Some(target) = apps_depth {
            if depth == target + 1 {
                if let Some(id) = unquote(t) {
                    if id.chars().all(|c| c.is_ascii_digit()) {
                        if let Ok(n) = id.parse::<AppId>() {
                            ids.insert(n);
                        }
                    }
                }
            }
        }
        depth += brace_delta(t);
        if let Some(target) = apps_depth {
            // Close only when a closing brace brings us back to the level the
            // `"apps"` key itself sits at. (The key's own line has delta 0.)
            if brace_delta(t) < 0 && depth <= target {
                apps_depth = None;
            }
        }
    }
    Ok(ids.into_iter().collect())
}

fn brace_delta(line: &str) -> i32 {
    line.chars().fold(0, |acc, c| match c {
        '{' => acc + 1,
        '}' => acc - 1,
        _ => acc,
    })
}

fn unquote(s: &str) -> Option<String> {
    let s = s.trim();
    let start = s.find('"')?;
    let rest = &s[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn parses_appmanifest_and_merges_extras() {
        let dir = tempdir().unwrap();
        let steamapps = dir.path().join("steamapps");
        std::fs::create_dir_all(&steamapps).unwrap();
        std::fs::write(
            steamapps.join("appmanifest_220.acf"),
            r#""AppState"
{
	"appid"		"220"
	"name"		"Half-Life 2"
}
"#,
        )
        .unwrap();
        let installed = load_installed_games(dir.path()).unwrap();
        assert_eq!(installed.len(), 1);
        assert_eq!(installed[0].name, "Half-Life 2");
        let merged = merge_library_with_app_ids(installed, &[10]);
        assert_eq!(merged.len(), 2);
        assert!(merged.iter().any(|g| g.app_id == 10 && g.name == "App 10"));
    }

    fn write_manifest(steamapps: &Path, app_id: AppId, name: &str) {
        std::fs::create_dir_all(steamapps).unwrap();
        std::fs::write(
            steamapps.join(format!("appmanifest_{app_id}.acf")),
            format!(
                r#""AppState"
{{
	"appid"		"{app_id}"
	"name"		"{name}"
}}
"#
            ),
        )
        .unwrap();
    }

    #[test]
    fn loads_games_from_library_folders_vdf() {
        let dir = tempdir().unwrap();
        let main = dir.path().join("Steam");
        let library = dir.path().join("SteamLibrary");
        write_manifest(&main.join("steamapps"), 220, "Half-Life 2");
        write_manifest(&library.join("steamapps"), 440, "Team Fortress 2");

        let vdf_path = main.join("steamapps").join("libraryfolders.vdf");
        let library_vdf = library.display().to_string().replace('\\', "\\\\");
        std::fs::write(
            &vdf_path,
            format!(
                r#""libraryfolders"
{{
	"0"
	{{
		"path"		"{library_vdf}"
	}}
}}
"#
            ),
        )
        .unwrap();

        let installed = load_installed_games(&main).unwrap();
        assert_eq!(installed.len(), 2);
        assert_eq!(installed[0].app_id, 220);
        assert_eq!(installed[1].app_id, 440);
        assert_eq!(installed[1].name, "Team Fortress 2");
    }

    #[test]
    fn live_library_unions_installed_collections_and_localconfig() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        write_manifest(&root.join("steamapps"), 220, "Half-Life 2");

        let config = root.join("userdata").join("123456").join("config");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(
            config.join("localconfig.vdf"),
            r#""UserLocalConfigStore"
{
	"apps"
	{
		"440"
		{
			"LastPlayed"		"1"
		}
		"570"
		{
			"LastPlayed"		"2"
		}
	}
}
"#,
        )
        .unwrap();

        let games = load_live_library(root, "123456", &[4000]).unwrap();
        let ids: Vec<AppId> = games.iter().map(|g| g.app_id).collect();
        assert_eq!(ids, vec![220, 440, 570, 4000]);
        assert!(games.iter().any(|g| g.app_id == 440 && g.name == "App 440"));
        assert!(games.iter().any(|g| g.app_id == 570 && g.name == "App 570"));
        assert!(games
            .iter()
            .any(|g| g.app_id == 4000 && g.name == "App 4000"));
    }
}
