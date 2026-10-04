use std::path::PathBuf;

use crate::autocat::AutoCatResult;
use crate::backup::BackupManager;
use crate::collections::changeset::apply_changeset;
use crate::collections::store::CollectionStore;
use crate::platform::SteamPlatform;
use crate::types::{AppId, Collection, CollectionChangeset, Game};
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

    /// Collections as they would look after applying pending (display helper).
    pub fn effective_collections(&self) -> Result<Vec<Collection>> {
        let mut cols = self.collections.clone();
        apply_changeset(&mut cols, &self.pending)?;
        Ok(cols)
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

    pub fn create_collection(&mut self, name: impl Into<String>) -> Result<String> {
        let name = name.into();
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err(Error::UserAction("collection name required".into()));
        }
        let effective = self.effective_collections()?;
        if effective.iter().any(|c| c.name.eq_ignore_ascii_case(&name)) {
            return Err(Error::UserAction(format!(
                "collection already exists: {name}"
            )));
        }
        let base = format!(
            "manual-{}",
            name.chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect::<String>()
        );
        let mut id = base.clone();
        let mut n = 1u32;
        while effective.iter().any(|c| c.id == id)
            || self.pending.create.iter().any(|c| c.id == id)
        {
            id = format!("{base}-{n}");
            n += 1;
        }
        self.pending.create.push(Collection {
            id: id.clone(),
            name,
            app_ids: vec![],
        });
        Ok(id)
    }

    pub fn add_games_to_collection(&mut self, collection_id: &str, app_ids: &[AppId]) -> Result<()> {
        if app_ids.is_empty() {
            return Err(Error::UserAction("no games selected".into()));
        }
        let effective = self.effective_collections()?;
        if !effective.iter().any(|c| c.id == collection_id) {
            return Err(Error::UserAction(format!(
                "collection not found: {collection_id}"
            )));
        }
        self.pending
            .add_members
            .push((collection_id.to_string(), app_ids.to_vec()));
        Ok(())
    }

    pub fn remove_games_from_collection(
        &mut self,
        collection_id: &str,
        app_ids: &[AppId],
    ) -> Result<()> {
        if app_ids.is_empty() {
            return Err(Error::UserAction("no games selected".into()));
        }
        let effective = self.effective_collections()?;
        if !effective.iter().any(|c| c.id == collection_id) {
            return Err(Error::UserAction(format!(
                "collection not found: {collection_id}"
            )));
        }
        self.pending
            .remove_members
            .push((collection_id.to_string(), app_ids.to_vec()));
        Ok(())
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

    #[test]
    fn manual_create_and_add_shows_in_effective_collections() {
        let mut session = Session::new(
            vec![Game {
                app_id: 10,
                name: "CS".into(),
            }],
            vec![],
            vec![],
        );
        let id = session.create_collection("Keepers").unwrap();
        session.add_games_to_collection(&id, &[10]).unwrap();
        let cols = session.effective_collections().unwrap();
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0].name, "Keepers");
        assert_eq!(cols[0].app_ids, vec![10]);
    }
}
