pub mod autocat;
pub mod backup;
pub mod collections;
pub mod config;
pub mod error;
pub mod library;
pub mod platform;
pub mod session;
pub mod types;

pub use autocat::{
    run_preset, AutoCatPreset, AutoCatResult, AutoCatRule, GameMetadata, MetadataIndex,
    UserScoreBand, YearMode,
};
pub use backup::BackupManager;
pub use collections::{
    apply_changeset, CollectionStore, JsonCollectionStore, LevelDbCollectionStore,
    SteamCloudCatalogStore,
};
pub use config::AppConfig;
pub use error::{Error, Result};
pub use library::{load_live_library, load_owned_games};
pub use platform::{
    probe_steam_version, MockSteamPlatform, SteamCompatInfo, SteamPlatform,
    SUPPORTED_STEAM_LAYOUT, SUPPORTED_STEAM_NOTE,
};
pub use session::Session;
pub use types::{AppId, Collection, CollectionChangeset, Game};

#[cfg(windows)]
pub use platform::{WindowsSteamPlatform, DEFAULT_STEAM_PATH};

pub fn crate_name() -> &'static str {
    "steam-core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_is_steam_core() {
        assert_eq!(crate_name(), "steam-core");
    }
}
