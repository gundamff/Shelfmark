pub mod traits;
#[cfg(windows)]
pub mod windows;

pub use traits::{scan_userdata_ids3, MockSteamPlatform, SteamPlatform};

#[cfg(windows)]
pub use windows::{WindowsSteamPlatform, DEFAULT_STEAM_PATH};
