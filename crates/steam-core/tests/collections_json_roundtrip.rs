use std::path::PathBuf;

use steam_core::{
    apply_changeset, CollectionChangeset, CollectionStore, JsonCollectionStore,
};
use tempfile::tempdir;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/steam/userdata/123456/config/cloudstorage/cloud-storage-namespace-1.json")
}

#[test]
fn json_store_roundtrip_after_changeset() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("cloud-storage-namespace-1.json");
    std::fs::copy(fixture_path(), &path).unwrap();

    let store = JsonCollectionStore::new(path);
    let mut cols = store.load().unwrap();
    assert_eq!(cols.len(), 2);

    let cs = CollectionChangeset {
        add_members: vec![("c-indie".into(), vec![440])],
        ..Default::default()
    };
    apply_changeset(&mut cols, &cs).unwrap();
    store.save(&cols).unwrap();

    let again = store.load().unwrap();
    let indie = again.iter().find(|c| c.id == "c-indie").unwrap();
    assert!(indie.app_ids.contains(&220));
    assert!(indie.app_ids.contains(&440));
}
