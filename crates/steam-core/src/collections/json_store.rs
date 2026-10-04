use std::path::PathBuf;

use crate::collections::store::CollectionStore;
use crate::collections::types::JsonCollectionsFile;
use crate::types::Collection;
use crate::{Error, Result};

/// JSON collection store using Shelfmark-normalized schema at
/// `{steam}/userdata/{id3}/config/cloudstorage/cloud-storage-namespace-1.json`
/// (fixture-friendly; not byte-identical to live Steam files).
pub struct JsonCollectionStore {
    pub path: PathBuf,
}

impl JsonCollectionStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl CollectionStore for JsonCollectionStore {
    fn load(&self) -> Result<Vec<Collection>> {
        if !self.path.is_file() {
            return Err(Error::UserAction(format!(
                "collections file not found: {}",
                self.path.display()
            )));
        }
        let raw = std::fs::read_to_string(&self.path)?;
        let file: JsonCollectionsFile = serde_json::from_str(&raw)
            .map_err(|e| Error::Parse(format!("collections json: {e}")))?;
        Ok(file.collections.into_iter().map(Collection::from).collect())
    }

    fn save(&self, collections: &[Collection]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = JsonCollectionsFile {
            collections: collections.iter().map(Into::into).collect(),
        };
        let raw = serde_json::to_string_pretty(&file)
            .map_err(|e| Error::Internal(format!("serialize collections: {e}")))?;
        std::fs::write(&self.path, raw)?;
        Ok(())
    }
}
