use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::collections::store::CollectionStore;
use crate::types::{AppId, Collection};
use crate::{Error, Result};

/// Steam `cloud-storage-namespace-1.json` catalog store.
///
/// On-disk shape (observed): JSON array of `[key, element]` pairs where
/// `element.value` is a stringified JSON object. Collection keys start with
/// `user-collections`. Entries with `filterSpec` (dynamic) are skipped on load.
pub struct SteamCloudCatalogStore {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CatalogElement {
    key: String,
    #[serde(default)]
    timestamp: i64,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    is_deleted: bool,
    #[serde(default)]
    value: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct CollectionValue {
    id: String,
    name: String,
    #[serde(default)]
    added: Vec<AppId>,
    #[serde(default)]
    removed: Vec<AppId>,
    #[serde(default)]
    #[serde(rename = "filterSpec")]
    filter_spec: Option<Value>,
}

impl SteamCloudCatalogStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn now_ts() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    fn read_catalog(&self) -> Result<Vec<(String, CatalogElement)>> {
        if !self.path.is_file() {
            return Err(Error::UserAction(format!(
                "steam catalog not found: {}",
                self.path.display()
            )));
        }
        let raw = std::fs::read_to_string(&self.path)?;
        let arr: Value = serde_json::from_str(&raw)
            .map_err(|e| Error::Parse(format!("steam catalog: {e}")))?;
        let Value::Array(items) = arr else {
            return Err(Error::Parse("steam catalog root must be array".into()));
        };

        let mut out = Vec::new();
        for item in items {
            let Value::Array(pair) = item else {
                continue;
            };
            if pair.len() < 2 {
                continue;
            }
            let key = pair[0].as_str().unwrap_or("").to_string();
            let elem: CatalogElement = serde_json::from_value(pair[1].clone())
                .map_err(|e| Error::Parse(format!("catalog element {key}: {e}")))?;
            out.push((key, elem));
        }
        Ok(out)
    }

    fn write_catalog(&self, entries: &[(String, CatalogElement)]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let arr: Vec<Value> = entries
            .iter()
            .map(|(key, elem)| json!([key, elem]))
            .collect();
        let raw = serde_json::to_string(&arr)
            .map_err(|e| Error::Internal(format!("serialize catalog: {e}")))?;
        std::fs::write(&self.path, raw)?;
        Ok(())
    }
}

impl CollectionStore for SteamCloudCatalogStore {
    fn load(&self) -> Result<Vec<Collection>> {
        let entries = self.read_catalog()?;
        let mut collections = Vec::new();
        for (key, elem) in entries {
            if elem.is_deleted || !key.starts_with("user-collections") {
                continue;
            }
            if elem.value.is_empty() {
                continue;
            }
            let value: CollectionValue = serde_json::from_str(&elem.value)
                .map_err(|e| Error::Parse(format!("collection value {key}: {e}")))?;
            if value.filter_spec.is_some() {
                continue;
            }
            let id = if value.id.is_empty() {
                key.trim_start_matches("user-collections.").to_string()
            } else {
                value.id
            };
            collections.push(Collection {
                id,
                name: value.name,
                app_ids: value.added,
            });
        }
        Ok(collections)
    }

    fn save(&self, collections: &[Collection]) -> Result<()> {
        let mut entries = self.read_catalog()?;
        let wanted: std::collections::HashMap<&str, &Collection> =
            collections.iter().map(|c| (c.id.as_str(), c)).collect();

        // Update existing user-collections entries; drop ones we manage that are gone.
        let mut seen = std::collections::HashSet::new();
        let mut next = Vec::new();
        for (key, mut elem) in entries.drain(..) {
            if !key.starts_with("user-collections") {
                next.push((key, elem));
                continue;
            }
            if elem.is_deleted {
                next.push((key, elem));
                continue;
            }
            let Ok(value) = serde_json::from_str::<CollectionValue>(&elem.value) else {
                next.push((key, elem));
                continue;
            };
            if value.filter_spec.is_some() {
                next.push((key, elem));
                continue;
            }
            let id = if value.id.is_empty() {
                key.trim_start_matches("user-collections.").to_string()
            } else {
                value.id.clone()
            };
            if let Some(col) = wanted.get(id.as_str()) {
                let updated = CollectionValue {
                    id: col.id.clone(),
                    name: col.name.clone(),
                    added: col.app_ids.clone(),
                    removed: vec![],
                    filter_spec: None,
                };
                elem.value = serde_json::to_string(&updated)
                    .map_err(|e| Error::Internal(format!("serialize collection: {e}")))?;
                elem.timestamp = Self::now_ts();
                elem.key = key.clone();
                seen.insert(col.id.clone());
                next.push((key, elem));
            }
            // else: drop managed collection that is no longer present
        }

        for col in collections {
            if seen.contains(&col.id) {
                continue;
            }
            let key = format!("user-collections.{}", col.id);
            let value = CollectionValue {
                id: col.id.clone(),
                name: col.name.clone(),
                added: col.app_ids.clone(),
                removed: vec![],
                filter_spec: None,
            };
            let value_s = serde_json::to_string(&value)
                .map_err(|e| Error::Internal(format!("serialize collection: {e}")))?;
            next.push((
                key.clone(),
                CatalogElement {
                    key: key.clone(),
                    timestamp: Self::now_ts(),
                    version: Some("1".into()),
                    is_deleted: false,
                    value: value_s,
                },
            ));
        }

        self.write_catalog(&next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::changeset::apply_changeset;
    use crate::types::CollectionChangeset;
    use tempfile::tempdir;

    fn fixture_catalog() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/steam/userdata/123456/config/cloudstorage/steam-catalog.json")
    }

    #[test]
    fn loads_static_collections_skips_dynamic() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("catalog.json");
        std::fs::copy(fixture_catalog(), &path).unwrap();
        let store = SteamCloudCatalogStore::new(path);
        let cols = store.load().unwrap();
        assert_eq!(cols.len(), 2);
        assert!(cols.iter().any(|c| c.name == "Indie"));
        assert!(cols.iter().any(|c| c.name == "FPS"));
        assert!(!cols.iter().any(|c| c.name == "Dynamic"));
    }

    #[test]
    fn save_roundtrip_preserves_non_collection_keys() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("catalog.json");
        std::fs::copy(fixture_catalog(), &path).unwrap();
        let store = SteamCloudCatalogStore::new(path.clone());
        let mut cols = store.load().unwrap();
        apply_changeset(
            &mut cols,
            &CollectionChangeset {
                add_members: vec![("c-indie".into(), vec![440])],
                ..Default::default()
            },
        )
        .unwrap();
        store.save(&cols).unwrap();

        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(raw.contains("GameReleased"));
        let again = store.load().unwrap();
        let indie = again.iter().find(|c| c.id == "c-indie").unwrap();
        assert!(indie.app_ids.contains(&440));
        assert!(indie.app_ids.contains(&220));
    }
}
