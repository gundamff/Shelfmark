use std::path::Path;

use serde::Serialize;

/// Human-readable layout Shelfmark supports (update when maintainers re-test).
pub const SUPPORTED_STEAM_LAYOUT: &str =
    "Windows Steam · userdata/.../cloudstorage/cloud-storage-namespace-1.json";

/// Notes which client builds were last verified (FileVersion major family).
pub const SUPPORTED_STEAM_NOTE: &str = "tested with steam.exe FileVersion 10.x (as of 2026-10)";

/// Major FileVersion family considered in-support when detection works.
pub const SUPPORTED_FILEVERSION_MAJOR: u32 = 10;

/// Why a version probe did or did not produce a version string.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VersionDetectionState {
    /// No Steam root was provided, so nothing was checked.
    NotProbed,
    /// A FileVersion was read successfully.
    Ok,
    /// `steam.exe` was not found at the checked path.
    ExeMissing,
    /// `steam.exe` exists but its version resource could not be read.
    Unreadable,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SteamCompatInfo {
    pub supported_label: String,
    pub supported_note: String,
    pub detected_version: Option<String>,
    pub within_support: Option<bool>,
    /// Absolute-ish path to the `steam.exe` that was checked, when a root was known.
    pub checked_exe_path: Option<String>,
    pub detection_state: VersionDetectionState,
}

impl SteamCompatInfo {
    pub fn probe(steam_root: &Path) -> Self {
        let exe = steam_root.join("steam.exe");
        let checked_exe_path = Some(exe.display().to_string());
        let (detected_version, detection_state) = probe_steam_version_detailed(steam_root);
        let within_support = detected_version.as_deref().map(version_within_support);
        Self {
            supported_label: SUPPORTED_STEAM_LAYOUT.to_string(),
            supported_note: SUPPORTED_STEAM_NOTE.to_string(),
            detected_version,
            within_support,
            checked_exe_path,
            detection_state,
        }
    }

    /// Placeholder used before any Steam root has been detected.
    pub fn not_probed() -> Self {
        Self {
            supported_label: SUPPORTED_STEAM_LAYOUT.to_string(),
            supported_note: SUPPORTED_STEAM_NOTE.to_string(),
            detected_version: None,
            within_support: None,
            checked_exe_path: None,
            detection_state: VersionDetectionState::NotProbed,
        }
    }
}

pub fn version_within_support(file_version: &str) -> bool {
    parse_major(file_version) == Some(SUPPORTED_FILEVERSION_MAJOR)
}

fn parse_major(file_version: &str) -> Option<u32> {
    file_version
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
}

/// Format a PE `VS_FIXEDFILEINFO` file version (MS/LS DWORDs) as `major.minor.build.rev`.
pub fn format_file_version(file_version_ms: u32, file_version_ls: u32) -> String {
    let major = (file_version_ms >> 16) & 0xffff;
    let minor = file_version_ms & 0xffff;
    let build = (file_version_ls >> 16) & 0xffff;
    let rev = file_version_ls & 0xffff;
    format!("{major}.{minor}.{build}.{rev}")
}

/// Read `steam.exe` FileVersion (Windows PE version resource).
pub fn probe_steam_version(steam_root: &Path) -> Option<String> {
    probe_steam_version_detailed(steam_root).0
}

fn probe_steam_version_detailed(steam_root: &Path) -> (Option<String>, VersionDetectionState) {
    let exe = steam_root.join("steam.exe");
    if !exe.is_file() {
        return (None, VersionDetectionState::ExeMissing);
    }
    #[cfg(windows)]
    {
        match read_pe_file_version(&exe) {
            Some(v) => (Some(v), VersionDetectionState::Ok),
            None => (None, VersionDetectionState::Unreadable),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = exe;
        (None, VersionDetectionState::Unreadable)
    }
}

#[cfg(windows)]
fn read_pe_file_version(path: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;

    #[repr(C)]
    struct VsFixedFileInfo {
        signature: u32,
        struc_version: u32,
        file_version_ms: u32,
        file_version_ls: u32,
        product_version_ms: u32,
        product_version_ls: u32,
        file_flags_mask: u32,
        file_flags: u32,
        file_os: u32,
        file_type: u32,
        file_subtype: u32,
        file_date_ms: u32,
        file_date_ls: u32,
    }

    #[link(name = "version")]
    unsafe extern "system" {
        fn GetFileVersionInfoSizeW(filename: *const u16, handle: *mut u32) -> u32;
        fn GetFileVersionInfoW(
            filename: *const u16,
            handle: u32,
            len: u32,
            data: *mut core::ffi::c_void,
        ) -> i32;
        fn VerQueryValueW(
            block: *const core::ffi::c_void,
            sub_block: *const u16,
            buffer: *mut *mut core::ffi::c_void,
            len: *mut u32,
        ) -> i32;
    }

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        let mut handle = 0u32;
        let size = GetFileVersionInfoSizeW(wide.as_ptr(), &mut handle);
        if size == 0 {
            return None;
        }
        let mut buf = vec![0u8; size as usize];
        if GetFileVersionInfoW(
            wide.as_ptr(),
            0,
            size,
            buf.as_mut_ptr().cast(),
        ) == 0
        {
            return None;
        }
        let sub: Vec<u16> = "\\".encode_utf16().chain(Some(0)).collect();
        let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut len = 0u32;
        if VerQueryValueW(buf.as_ptr().cast(), sub.as_ptr(), &mut ptr, &mut len) == 0
            || ptr.is_null()
            || (len as usize) < std::mem::size_of::<VsFixedFileInfo>()
        {
            return None;
        }
        let info = &*(ptr as *const VsFixedFileInfo);
        if info.signature != 0xFEEF04BD {
            return None;
        }
        Some(format_file_version(info.file_version_ms, info.file_version_ls))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn supported_label_is_non_empty() {
        assert!(!SUPPORTED_STEAM_LAYOUT.is_empty());
        assert!(!SUPPORTED_STEAM_NOTE.is_empty());
    }

    #[test]
    fn probe_missing_root_reports_exe_missing() {
        let root = Path::new("C:\\no-such-steam-shelfmark");
        assert!(probe_steam_version(root).is_none());
        let compat = SteamCompatInfo::probe(root);
        assert_eq!(compat.detected_version, None);
        assert_eq!(compat.detection_state, VersionDetectionState::ExeMissing);
        assert!(compat
            .checked_exe_path
            .as_deref()
            .unwrap_or_default()
            .ends_with("steam.exe"));
    }

    #[test]
    fn major_10_is_within_support() {
        assert!(version_within_support("10.96.30.42"));
        assert!(!version_within_support("9.0.0.0"));
    }

    #[test]
    fn parses_file_version_fields_from_fixture() {
        // 10.96.30.42 => ms = (10<<16)|96, ls = (30<<16)|42
        let ms = (10u32 << 16) | 96;
        let ls = (30u32 << 16) | 42;
        assert_eq!(format_file_version(ms, ls), "10.96.30.42");
    }

    #[test]
    fn not_probed_state_has_no_checked_path() {
        let c = SteamCompatInfo::not_probed();
        assert_eq!(c.detection_state, VersionDetectionState::NotProbed);
        assert_eq!(c.checked_exe_path, None);
        assert_eq!(c.detected_version, None);
    }
}
