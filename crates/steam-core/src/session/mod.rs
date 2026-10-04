use std::path::PathBuf;

use crate::backup::BackupManager;
use crate::collections::changeset::apply_changeset;
use crate::collections::store::CollectionStore;
use crate::platform::SteamPlatform;
use crate::types::{Collection, CollectionChangeset, Game};
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct Session {
    pub games: Vec<Game>,
    pub collections: Vec<Collection>,
    pub pending: CollectionChangeset,
    /// Files that must be backed up before export (e.g. JSON store path).
    pub store_paths: Vec<PathBuf>,
}

impl Session {
    pub fn new(games: Vec<Game>, collections: Vec<Collection>, store_paths: Vec<PathBuf>) -> Self {
        Self {
            games,
            collections,
            pending: CollectionChangeset::default(),
            store_paths,
        }
    }

    pub fn is_pending_empty(&self) -> bool {
        self.pending.create.is_empty()
            && self.pending.rename.is_empty()
            && self.pending.delete.is_empty()
            && self.pending.add_members.is_empty()
            && self.pending.remove_members.is_empty()
    }

    pub fn export(
        &mut self,
        platform: &dyn SteamPlatform,
        store: &dyn CollectionStore,
        backups: &BackupManager,
    ) -> Result<()> {
        if platform.is_steam_running()? {
            return Err(Error::UserAction(
                "close Steam before export".into(),
            ));
        }
        if self.is_pending_empty() {
            return Err(Error::UserAction("nothing to export".into()));
        }

        backups.backup_files(&self.store_paths)?;
        apply_changeset(&mut self.collections, &self.pending)?;
        store.save(&self.collections)?;
        self.pending = CollectionChangeset::default();
        Ok(())
    }
}
