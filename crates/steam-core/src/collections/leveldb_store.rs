use std::path::PathBuf;

use rusty_leveldb::{Options, DB};

use crate::collections::store::CollectionStore;
use crate::collections::types::JsonCollectionsFile;
use crate::types::Collection;
use crate::{Error, Result};

/// Build the LevelDB key used for a user's cloud-storage namespace catalog.
///
/// Observable shape (reimplemented from public behavior, not copied source):
/// `_https://steamloopback.host` + NUL + SOH + `U{id3}-cloud-storage-namespace-1`
pub fn catalog_key(user_id3: &str) -> Vec<u8> {
    let mut key = b"_https://steamloopback.host".to_vec();
    key.push(0x00);
    key.push(0x01);
    key.extend_from_slice(format!("U{user_id3}-cloud-storage-namespace-1").as_bytes());
    key
}

/// Frame catalog JSON as: encoding tag byte + payload.
/// Tag `0` = UTF-16LE text; any other tag = UTF-8 (we write tag `1`).
pub fn encode_catalog_utf8(json: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(1 + json.len());
    out.push(1);
    out.extend_from_slice(json.as_bytes());
    out
}

pub fn decode_catalog_payload(bytes: &[u8]) -> Result<String> {
    if bytes.is_empty() {
        return Err(Error::Parse("empty leveldb catalog value".into()));
    }
    let tag = bytes[0];
    let payload = &bytes[1..];
    if tag == 0 {
        if payload.len() % 2 != 0 {
            return Err(Error::Parse("utf-16 catalog payload odd length".into()));
        }
        let units: Vec<u16> = payload
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&units).map_err(|e| Error::Parse(format!("utf-16 catalog: {e}")))
    } else {
        String::from_utf8(payload.to_vec())
            .map_err(|e| Error::Parse(format!("utf-8 catalog: {e}")))
    }
}

/// LevelDB-backed collection store. Uses Shelfmark-normalized JSON inside the framed value.
pub struct LevelDbCollectionStore {
    pub database_dir: PathBuf,
    pub user_id3: String,
}

impl LevelDbCollectionStore {
    pub fn new(database_dir: PathBuf, user_id3: impl Into<String>) -> Self {
        Self {
            database_dir,
            user_id3: user_id3.into(),
        }
    }

    fn open_db(&self) -> Result<DB> {
        std::fs::create_dir_all(&self.database_dir)?;
        let mut opts = Options::default();
        opts.create_if_missing = true;
        DB::open(&self.database_dir, opts)
            .map_err(|e| Error::Io(std::io::Error::other(format!("leveldb open: {e}"))))
    }
}

impl CollectionStore for LevelDbCollectionStore {
    fn load(&self) -> Result<Vec<Collection>> {
        let mut db = self.open_db()?;
        let key = catalog_key(&self.user_id3);
        let bytes = db.get(&key).ok_or_else(|| {
            Error::UserAction(format!("catalog key missing for user {}", self.user_id3))
        })?;
        let json = decode_catalog_payload(&bytes)?;
        let file: JsonCollectionsFile = serde_json::from_str(&json)
            .map_err(|e| Error::Parse(format!("leveldb catalog json: {e}")))?;
        Ok(file.collections.into_iter().map(Collection::from).collect())
    }

    fn save(&self, collections: &[Collection]) -> Result<()> {
        let file = JsonCollectionsFile {
            collections: collections.iter().map(Into::into).collect(),
        };
        let json = serde_json::to_string(&file)
            .map_err(|e| Error::Internal(format!("serialize catalog: {e}")))?;
        let value = encode_catalog_utf8(&json);
        let mut db = self.open_db()?;
        let key = catalog_key(&self.user_id3);
        db.put(&key, &value)
            .map_err(|e| Error::Io(std::io::Error::other(format!("leveldb put: {e}"))))?;
        db.flush()
            .map_err(|e| Error::Io(std::io::Error::other(format!("leveldb flush: {e}"))))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::changeset::apply_changeset;
    use crate::types::CollectionChangeset;
    use tempfile::tempdir;

    #[test]
    fn catalog_key_has_expected_prefix_and_user() {
        let key = catalog_key("123456");
        assert!(key.starts_with(b"_https://steamloopback.host"));
        assert_eq!(key[b"_https://steamloopback.host".len()], 0);
        assert_eq!(key[b"_https://steamloopback.host".len() + 1], 1);
        let suffix = String::from_utf8(key[b"_https://steamloopback.host".len() + 2..].to_vec())
            .unwrap();
        assert_eq!(suffix, "U123456-cloud-storage-namespace-1");
    }

    #[test]
    fn encode_decode_utf8_roundtrip() {
        let json = r#"{"collections":[]}"#;
        let framed = encode_catalog_utf8(json);
        assert_eq!(framed[0], 1);
        assert_eq!(decode_catalog_payload(&framed).unwrap(), json);
    }

    #[test]
    fn leveldb_store_roundtrip() {
        let dir = tempdir().unwrap();
        let store = LevelDbCollectionStore::new(dir.path().to_path_buf(), "123456");
        let cols = vec![Collection {
            id: "c1".into(),
            name: "Indie".into(),
            app_ids: vec![220],
        }];
        store.save(&cols).unwrap();

        let mut loaded = store.load().unwrap();
        let cs = CollectionChangeset {
            add_members: vec![("c1".into(), vec![440])],
            ..Default::default()
        };
        apply_changeset(&mut loaded, &cs).unwrap();
        store.save(&loaded).unwrap();
        let again = store.load().unwrap();
        assert_eq!(again[0].app_ids, vec![220, 440]);
    }
}
