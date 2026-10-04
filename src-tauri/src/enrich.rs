use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use steam_core::{AppId, GameMetadata, MetadataIndex};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaCache {
    pub apps: HashMap<AppId, GameMetadata>,
}

impl MetaCache {
    pub fn path() -> Result<PathBuf, String> {
        let base = dirs::data_dir().ok_or_else(|| "AppData missing".to_string())?;
        Ok(base.join("Shelfmark").join("meta-cache.json"))
    }

    pub fn load() -> Self {
        let Ok(path) = Self::path() else {
            return Self::default();
        };
        let Ok(raw) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str(&raw).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, raw).map_err(|e| e.to_string())
    }
}

/// Parse one Steam Store `appdetails` response object (`data` node).
pub fn metadata_from_appdetails_data(data: &Value) -> Option<GameMetadata> {
    if !data.is_object() {
        return None;
    }
    let genres: Vec<String> = data
        .get("genres")
        .and_then(|g| g.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.get("description")?.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let flags: Vec<String> = data
        .get("categories")
        .and_then(|g| g.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.get("description")?.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    let developers = string_list(data.get("developers"));
    let publishers = string_list(data.get("publishers"));
    let release_year = data
        .get("release_date")
        .and_then(|r| r.get("date"))
        .and_then(|d| d.as_str())
        .and_then(parse_year);
    let user_score_percent = data
        .get("metacritic")
        .and_then(|m| m.get("score"))
        .and_then(|s| s.as_u64())
        .map(|s| s.min(100) as u8);
    // Store appdetails has no community tags here; reuse genres as light tag signal.
    let tags = genres.clone();

    Some(GameMetadata {
        genres,
        tags,
        flags,
        developers,
        publishers,
        release_year,
        user_score_percent,
    })
}

fn string_list(v: Option<&Value>) -> Vec<String> {
    v.and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_year(date: &str) -> Option<u16> {
    // "16 Nov, 2004" / "Nov 16, 2004" / "2004"
    for token in date.split(|c: char| !c.is_ascii_digit()) {
        if token.len() == 4 {
            if let Ok(y) = token.parse::<u16>() {
                if (1970..=2100).contains(&y) {
                    return Some(y);
                }
            }
        }
    }
    None
}

pub fn fetch_appdetails(app_id: AppId) -> Result<Option<GameMetadata>, String> {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={app_id}");
    let body: Value = ureq::get(&url)
        .header("User-Agent", "Shelfmark/0.1")
        .call()
        .map_err(|e| format!("store fetch {app_id}: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("store json {app_id}: {e}"))?;

    let entry = body
        .get(app_id.to_string())
        .ok_or_else(|| format!("missing appdetails key for {app_id}"))?;
    let success = entry.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
    if !success {
        return Ok(None);
    }
    let data = entry.get("data").ok_or_else(|| "missing data".to_string())?;
    Ok(metadata_from_appdetails_data(data))
}

/// Enrich missing entries in `meta` for `app_ids`, using disk cache + store API.
pub fn enrich_metadata_index(
    meta: &mut MetadataIndex,
    app_ids: &[AppId],
    max_fetch: usize,
) -> Result<EnrichReport, String> {
    let mut cache = MetaCache::load();
    let mut report = EnrichReport::default();

    for &app_id in app_ids {
        if let Some(cached) = cache.apps.get(&app_id) {
            if !cached.genres.is_empty() || cached.release_year.is_some() {
                meta.insert(app_id, cached.clone());
                report.from_cache += 1;
                continue;
            }
        }
        if meta.get(&app_id).is_some_and(|m| !m.genres.is_empty()) {
            report.already_present += 1;
            continue;
        }
        if report.fetched >= max_fetch as u32 {
            report.skipped_budget += 1;
            continue;
        }

        match fetch_appdetails(app_id) {
            Ok(Some(m)) => {
                cache.apps.insert(app_id, m.clone());
                meta.insert(app_id, m);
                report.fetched += 1;
            }
            Ok(None) => {
                report.failed += 1;
            }
            Err(_) => {
                report.failed += 1;
            }
        }
        // Be polite to Steam Store.
        thread::sleep(Duration::from_millis(300));
    }

    let _ = cache.save();
    Ok(report)
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichReport {
    pub from_cache: u32,
    pub fetched: u32,
    pub failed: u32,
    pub already_present: u32,
    pub skipped_budget: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_appdetails_sample() {
        let data = json!({
            "genres": [{"id":"1","description":"Action"},{"id":"23","description":"Indie"}],
            "categories": [{"id":2,"description":"Single-player"}],
            "developers": ["Valve"],
            "publishers": ["Valve"],
            "release_date": {"date": "16 Nov, 2004"},
            "metacritic": {"score": 96}
        });
        let m = metadata_from_appdetails_data(&data).unwrap();
        assert_eq!(m.genres, vec!["Action", "Indie"]);
        assert_eq!(m.flags, vec!["Single-player"]);
        assert_eq!(m.developers, vec!["Valve"]);
        assert_eq!(m.release_year, Some(2004));
        assert_eq!(m.user_score_percent, Some(96));
        assert_eq!(m.tags, m.genres);
    }
}
