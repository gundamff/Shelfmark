pub mod steam_version;
pub mod traits;
#[cfg(windows)]
pub mod windows;

pub use steam_version::{
    format_file_version, probe_steam_version, SteamCompatInfo, VersionDetectionState,
    SUPPORTED_STEAM_LAYOUT, SUPPORTED_STEAM_NOTE,
};
pub use traits::{scan_userdata_ids3, MockSteamPlatform, SteamPlatform};

#[cfg(windows)]
pub use windows::{WindowsSteamPlatform, DEFAULT_STEAM_PATH};
