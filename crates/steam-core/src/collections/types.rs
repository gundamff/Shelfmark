use serde::{Deserialize, Serialize};

use crate::types::{AppId, Collection};

/// Shelfmark-normalized on-disk schema for the JSON collection backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonCollectionsFile {
    pub collections: Vec<JsonCollectionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonCollectionEntry {
    pub id: String,
    pub name: String,
    pub added: Vec<AppId>,
}

impl From<JsonCollectionEntry> for Collection {
    fn from(value: JsonCollectionEntry) -> Self {
        Collection {
            id: value.id,
            name: value.name,
            app_ids: value.added,
        }
    }
}

impl From<&Collection> for JsonCollectionEntry {
    fn from(value: &Collection) -> Self {
        JsonCollectionEntry {
            id: value.id.clone(),
            name: value.name.clone(),
            added: value.app_ids.clone(),
        }
    }
}
