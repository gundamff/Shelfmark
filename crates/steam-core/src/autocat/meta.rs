use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::AppId;

/// Local metadata used by AutoCat. Missing fields mean the rule may skip the game.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GameMetadata {
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub flags: Vec<String>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    pub release_year: Option<u16>,
    /// Steam-style percent 0–100 when known.
    pub user_score_percent: Option<u8>,
}

pub type MetadataIndex = HashMap<AppId, GameMetadata>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_metadata_is_empty() {
        let m = GameMetadata::default();
        assert!(m.genres.is_empty());
        assert!(m.tags.is_empty());
        assert!(m.flags.is_empty());
        assert!(m.developers.is_empty());
        assert!(m.publishers.is_empty());
        assert!(m.release_year.is_none());
        assert!(m.user_score_percent.is_none());
    }
}
