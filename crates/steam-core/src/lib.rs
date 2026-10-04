pub mod backup;
pub mod collections;
pub mod error;
pub mod library;
pub mod platform;
pub mod session;
pub mod types;

pub use backup::BackupManager;
pub use collections::{
    apply_changeset, CollectionStore, JsonCollectionStore, LevelDbCollectionStore,
};
pub use error::{Error, Result};
pub use library::load_owned_games;
pub use platform::{MockSteamPlatform, SteamPlatform};
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
