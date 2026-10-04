use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use steam_core::backup::BackupManager;
use steam_core::platform::MockSteamPlatform;
use steam_core::session::Session;
use steam_core::types::{Collection, CollectionChangeset, Game};
use steam_core::{CollectionStore, Error};
use tempfile::tempdir;

#[derive(Default)]
struct RecordingStore {
    collections: RefCell<Vec<Collection>>,
    save_calls: Rc<RefCell<u32>>,
}

impl CollectionStore for RecordingStore {
    fn load(&self) -> steam_core::Result<Vec<Collection>> {
        Ok(self.collections.borrow().clone())
    }

    fn save(&self, collections: &[Collection]) -> steam_core::Result<()> {
        *self.save_calls.borrow_mut() += 1;
        *self.collections.borrow_mut() = collections.to_vec();
        Ok(())
    }
}

#[test]
fn export_refuses_when_steam_running() {
    let platform = MockSteamPlatform {
        root: PathBuf::from("C:\\fake"),
        users: vec![],
        running: true,
    };
    let store = RecordingStore::default();
    *store.collections.borrow_mut() = vec![Collection {
        id: "c1".into(),
        name: "A".into(),
        app_ids: vec![1],
    }];
    let tmp = tempdir().unwrap();
    let file = tmp.path().join("cols.json");
    std::fs::write(&file, "{}").unwrap();
    let backups = BackupManager::new(tmp.path().join("backups"));
    let mut session = Session::new(
        vec![Game {
            app_id: 1,
            name: "G".into(),
        }],
        store.load().unwrap(),
        vec![file],
    );
    session.pending.add_members = vec![("c1".into(), vec![2])];

    let err = session.export(&platform, &store, &backups).unwrap_err();
    assert!(matches!(err, Error::UserAction(_)));
    assert_eq!(*store.save_calls.borrow(), 0);
}

#[test]
fn export_writes_backup_then_store() {
    let platform = MockSteamPlatform {
        root: PathBuf::from("C:\\fake"),
        users: vec![],
        running: false,
    };
    let save_calls = Rc::new(RefCell::new(0u32));
    let store = RecordingStore {
        collections: RefCell::new(vec![Collection {
            id: "c1".into(),
            name: "A".into(),
            app_ids: vec![1],
        }]),
        save_calls: save_calls.clone(),
    };
    let tmp = tempdir().unwrap();
    let file = tmp.path().join("cols.json");
    std::fs::write(&file, b"original").unwrap();
    let backups = BackupManager::new(tmp.path().join("backups"));
    let mut session = Session::new(vec![], store.load().unwrap(), vec![file.clone()]);
    session.pending = CollectionChangeset {
        add_members: vec![("c1".into(), vec![2])],
        ..Default::default()
    };

    session.export(&platform, &store, &backups).unwrap();
    assert_eq!(*save_calls.borrow(), 1);
    assert!(session.is_pending_empty());
    let listed = backups.list_backups().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(
        std::fs::read(listed[0].join("cols.json")).unwrap(),
        b"original"
    );
    assert_eq!(store.collections.borrow()[0].app_ids, vec![1, 2]);
}
