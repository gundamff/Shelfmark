use crate::types::{Collection, CollectionChangeset};
use crate::{Error, Result};

/// Apply a changeset to an in-memory collection list.
///
/// Order: delete → create → rename → remove_members → add_members.
pub fn apply_changeset(collections: &mut Vec<Collection>, cs: &CollectionChangeset) -> Result<()> {
    for id in &cs.delete {
        collections.retain(|c| &c.id != id);
    }

    for created in &cs.create {
        if collections.iter().any(|c| c.id == created.id) {
            return Err(Error::UserAction(format!(
                "collection id already exists: {}",
                created.id
            )));
        }
        collections.push(created.clone());
    }

    for (id, new_name) in &cs.rename {
        let col = collections
            .iter_mut()
            .find(|c| &c.id == id)
            .ok_or_else(|| Error::UserAction(format!("collection not found for rename: {id}")))?;
        col.name = new_name.clone();
    }

    for (id, remove) in &cs.remove_members {
        let col = collections
            .iter_mut()
            .find(|c| &c.id == id)
            .ok_or_else(|| Error::UserAction(format!("collection not found for remove: {id}")))?;
        col.app_ids.retain(|app| !remove.contains(app));
    }

    for (id, add) in &cs.add_members {
        let col = collections
            .iter_mut()
            .find(|c| &c.id == id)
            .ok_or_else(|| Error::UserAction(format!("collection not found for add: {id}")))?;
        for app in add {
            if !col.app_ids.contains(app) {
                col.app_ids.push(*app);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Collection, CollectionChangeset};

    #[test]
    fn adds_members_to_existing_collection() {
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

    #[test]
    fn delete_create_rename_and_remove_members() {
        let mut cols = vec![
            Collection {
                id: "old".into(),
                name: "Old".into(),
                app_ids: vec![1, 2, 3],
            },
            Collection {
                id: "gone".into(),
                name: "Gone".into(),
                app_ids: vec![9],
            },
        ];
        let cs = CollectionChangeset {
            create: vec![Collection {
                id: "new".into(),
                name: "New".into(),
                app_ids: vec![5],
            }],
            rename: vec![("old".into(), "Renamed".into())],
            delete: vec!["gone".into()],
            add_members: vec![],
            remove_members: vec![("old".into(), vec![2])],
        };
        apply_changeset(&mut cols, &cs).unwrap();
        assert_eq!(cols.len(), 2);
        let old = cols.iter().find(|c| c.id == "old").unwrap();
        assert_eq!(old.name, "Renamed");
        assert_eq!(old.app_ids, vec![1, 3]);
        let new = cols.iter().find(|c| c.id == "new").unwrap();
        assert_eq!(new.app_ids, vec![5]);
    }

    #[test]
    fn create_duplicate_id_errors() {
        let mut cols = vec![Collection {
            id: "c1".into(),
            name: "A".into(),
            app_ids: vec![],
        }];
        let cs = CollectionChangeset {
            create: vec![Collection {
                id: "c1".into(),
                name: "B".into(),
                app_ids: vec![],
            }],
            ..Default::default()
        };
        assert!(apply_changeset(&mut cols, &cs).is_err());
    }
}
