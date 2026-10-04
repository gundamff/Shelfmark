use std::path::{Path, PathBuf};

use sysinfo::{ProcessesToUpdate, System};

use crate::platform::traits::{scan_userdata_ids3, SteamPlatform};
use crate::{Error, Result};

/// Default Steam install location on 64-bit Windows.
pub const DEFAULT_STEAM_PATH: &str = r"C:\Program Files (x86)\Steam";

#[derive(Debug, Default)]
pub struct WindowsSteamPlatform;

impl WindowsSteamPlatform {
    pub fn new() -> Self {
        Self
    }

    fn registry_steam_path() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            use winreg::enums::*;
            use winreg::RegKey;

            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            if let Ok(key) = hkcu.open_subkey("Software\\Valve\\Steam") {
                if let Ok(path) = key.get_value::<String, _>("SteamPath") {
                    let p = PathBuf::from(path.replace('/', "\\"));
                    if p.is_dir() {
                        return Some(p);
                    }
                }
            }

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            if let Ok(key) = hklm.open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam") {
                if let Ok(path) = key.get_value::<String, _>("InstallPath") {
                    let p = PathBuf::from(path);
                    if p.is_dir() {
                        return Some(p);
                    }
                }
            }
        }
        None
    }
}

impl SteamPlatform for WindowsSteamPlatform {
    fn steam_root(&self) -> Result<PathBuf> {
        if let Some(path) = Self::registry_steam_path() {
            return Ok(path);
        }

        let default = PathBuf::from(DEFAULT_STEAM_PATH);
        if default.is_dir() {
            return Ok(default);
        }

        Err(Error::UserAction(
            "Steam installation not found; set path manually".into(),
        ))
    }

    fn list_user_ids3(&self, steam_root: &Path) -> Result<Vec<String>> {
        scan_userdata_ids3(steam_root)
    }

    fn is_steam_running(&self) -> Result<bool> {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let running = sys.processes().values().any(|p| {
            let name = p.name().to_string_lossy().to_ascii_lowercase();
            name == "steam.exe" || name == "steam"
        });
        Ok(running)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn detects_local_steam_install() {
        let platform = WindowsSteamPlatform::new();
        let root = platform.steam_root().expect("local Steam should exist");
        assert!(root.is_dir(), "steam root missing: {}", root.display());
        // Known machine path from the developer environment.
        assert!(
            root.to_string_lossy()
                .to_ascii_lowercase()
                .contains("steam"),
            "unexpected root: {}",
            root.display()
        );
        let users = platform.list_user_ids3(&root).unwrap();
        assert!(
            !users.is_empty(),
            "expected at least one userdata id under {}",
            root.display()
        );
    }
}
