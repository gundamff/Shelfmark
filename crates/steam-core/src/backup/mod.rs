use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct BackupManager {
    pub root: PathBuf,
}

impl BackupManager {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Copy `paths` into a new timestamped directory under `root`.
    /// Returns the backup directory path.
    pub fn backup_files(&self, paths: &[PathBuf]) -> Result<PathBuf> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::Internal(format!("clock error: {e}")))?
            .as_secs();
        let dir = self.root.join(format!("{stamp}"));
        std::fs::create_dir_all(&dir)?;

        for path in paths {
            if !path.is_file() {
                return Err(Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("backup source missing: {}", path.display()),
                )));
            }
            let name = path
                .file_name()
                .ok_or_else(|| Error::Internal("backup path has no file name".into()))?;
            std::fs::copy(path, dir.join(name))?;
        }
        Ok(dir)
    }

    pub fn list_backups(&self) -> Result<Vec<PathBuf>> {
        if !self.root.is_dir() {
            return Ok(Vec::new());
        }
        let mut dirs = Vec::new();
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                dirs.push(entry.path());
            }
        }
        dirs.sort();
        Ok(dirs)
    }

    pub fn restore_file(&self, backup_dir: &Path, file_name: &str, dest: &Path) -> Result<()> {
        let src = backup_dir.join(file_name);
        if !src.is_file() {
            return Err(Error::UserAction(format!(
                "backup file missing: {}",
                src.display()
            )));
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(src, dest)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn backup_copies_file() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("data.json");
        std::fs::write(&src, b"{\"ok\":true}").unwrap();
        let mgr = BackupManager::new(tmp.path().join("backups"));
        let dir = mgr.backup_files(&[src.clone()]).unwrap();
        assert!(dir.join("data.json").is_file());
    }
}
