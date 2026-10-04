use std::path::PathBuf;

use crate::autocat::AutoCatResult;
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

    /// Merge an AutoCat dry-run into pending (create / rename / delete / member ops).
    pub fn queue_autocat(&mut self, result: AutoCatResult) {
        self.pending.create.extend(result.changeset.create);
        self.pending.rename.extend(result.changeset.rename);
        self.pending.delete.extend(result.changeset.delete);
        self.pending.add_members.extend(result.changeset.add_members);
        self.pending
            .remove_members
            .extend(result.changeset.remove_members);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autocat::{run_preset, AutoCatPreset, AutoCatRule, GameMetadata};
    use std::collections::HashMap;

    #[test]
    fn queue_autocat_merges_into_pending() {
        let games = vec![Game {
            app_id: 10,
            name: "CS".into(),
        }];
        let mut meta = HashMap::new();
        meta.insert(
            10,
            GameMetadata {
                genres: vec!["Action".into()],
                ..Default::default()
            },
        );
        let preset = AutoCatPreset {
            name: "g".into(),
            rules: vec![AutoCatRule::Genre {
                prefix: "g_".into(),
                max_categories: 0,
                remove_existing: false,
            }],
        };
        let result = run_preset(&games, &meta, &[], &preset).unwrap();
        let mut session = Session::new(games, vec![], vec![]);
        session.queue_autocat(result);
        assert!(!session.is_pending_empty());
        assert!(session.pending.create.iter().any(|c| c.name == "g_Action"));
    }
}
