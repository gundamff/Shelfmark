use std::path::Path;

use serde::Serialize;

/// Human-readable layout Shelfmark supports (update when maintainers re-test).
pub const SUPPORTED_STEAM_LAYOUT: &str =
    "Windows Steam · userdata/.../cloudstorage/cloud-storage-namespace-1.json";

/// Notes which client builds were last verified (FileVersion major family).
pub const SUPPORTED_STEAM_NOTE: &str = "tested with steam.exe FileVersion 10.x (as of 2026-10)";

/// Major FileVersion family considered in-support when detection works.
pub const SUPPORTED_FILEVERSION_MAJOR: u32 = 10;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SteamCompatInfo {
    pub supported_label: String,
    pub supported_note: String,
    pub detected_version: Option<String>,
    pub within_support: Option<bool>,
}

impl SteamCompatInfo {
    pub fn probe(steam_root: &Path) -> Self {
        let detected_version = probe_steam_version(steam_root);
        let within_support = detected_version
            .as_deref()
            .map(version_within_support);
        Self {
            supported_label: SUPPORTED_STEAM_LAYOUT.to_string(),
            supported_note: SUPPORTED_STEAM_NOTE.to_string(),
            detected_version,
            within_support,
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

/// Read `steam.exe` FileVersion (Windows PE version resource).
pub fn probe_steam_version(steam_root: &Path) -> Option<String> {
    let exe = steam_root.join("steam.exe");
    if !exe.is_file() {
        return None;
    }
    #[cfg(windows)]
    {
        return read_pe_file_version(&exe);
    }
    #[cfg(not(windows))]
    {
        let _ = exe;
        None
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
        let major = (info.file_version_ms >> 16) & 0xffff;
        let minor = info.file_version_ms & 0xffff;
        let build = (info.file_version_ls >> 16) & 0xffff;
        let rev = info.file_version_ls & 0xffff;
        Some(format!("{major}.{minor}.{build}.{rev}"))
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
    fn probe_missing_root_is_none() {
        assert!(probe_steam_version(Path::new("C:\\no-such-steam-shelfmark")).is_none());
    }

    #[test]
    fn major_10_is_within_support() {
        assert!(version_within_support("10.96.30.42"));
        assert!(!version_within_support("9.0.0.0"));
    }
}
