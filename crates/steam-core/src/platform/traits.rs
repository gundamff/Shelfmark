use std::path::{Path, PathBuf};

use crate::Result;

pub trait SteamPlatform {
    fn steam_root(&self) -> Result<PathBuf>;
    fn list_user_ids3(&self, steam_root: &Path) -> Result<Vec<String>>;
    fn is_steam_running(&self) -> Result<bool>;
}

/// Test double with injectable paths and running flag.
#[derive(Debug, Clone)]
pub struct MockSteamPlatform {
    pub root: PathBuf,
    pub users: Vec<String>,
    pub running: bool,
}

impl SteamPlatform for MockSteamPlatform {
    fn steam_root(&self) -> Result<PathBuf> {
        Ok(self.root.clone())
    }

    fn list_user_ids3(&self, _steam_root: &Path) -> Result<Vec<String>> {
        Ok(self.users.clone())
    }

    fn is_steam_running(&self) -> Result<bool> {
        Ok(self.running)
    }
}

/// Scan `{steam_root}/userdata/*` for numeric directory names (SteamID3).
pub fn scan_userdata_ids3(steam_root: &Path) -> Result<Vec<String>> {
    let userdata = steam_root.join("userdata");
    if !userdata.is_dir() {
        return Ok(Vec::new());
    }

    let mut ids = Vec::new();
    for entry in std::fs::read_dir(&userdata)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_digit()) {
            ids.push(name);
        }
    }
    ids.sort();
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn mock_lists_injected_users() {
        let mock = MockSteamPlatform {
            root: PathBuf::from("C:\\fake\\Steam"),
            users: vec!["111".into(), "222".into()],
            running: false,
        };
        assert_eq!(
            mock.list_user_ids3(Path::new(".")).unwrap(),
            vec!["111", "222"]
        );
        assert!(!mock.is_steam_running().unwrap());
    }

    #[test]
    fn scan_userdata_finds_numeric_dirs() {
        let dir = tempdir().unwrap();
        let userdata = dir.path().join("userdata");
        std::fs::create_dir_all(userdata.join("111")).unwrap();
        std::fs::create_dir_all(userdata.join("222")).unwrap();
        std::fs::create_dir_all(userdata.join("not-a-user")).unwrap();
        let ids = scan_userdata_ids3(dir.path()).unwrap();
        assert_eq!(ids, vec!["111".to_string(), "222".to_string()]);
    }
}
