use std::path::Path;

use serde::Deserialize;

use crate::types::Game;
use crate::{Error, Result};

#[derive(Debug, Deserialize)]
struct OwnedGameDto {
    app_id: u32,
    name: String,
}

/// Load owned games for a user.
///
/// MVP: reads Shelfmark fixture JSON at
/// `{steam_root}/userdata/{user_id3}/shelfmark/owned_games.json`.
pub fn load_owned_games(steam_root: &Path, user_id3: &str) -> Result<Vec<Game>> {
    let path = steam_root
        .join("userdata")
        .join(user_id3)
        .join("shelfmark")
        .join("owned_games.json");

    if !path.is_file() {
        return Err(Error::UserAction(format!(
            "owned games fixture not found: {}",
            path.display()
        )));
    }

    let raw = std::fs::read_to_string(&path)?;
    let dtos: Vec<OwnedGameDto> = serde_json::from_str(&raw)
        .map_err(|e| Error::Parse(format!("owned_games.json: {e}")))?;

    Ok(dtos
        .into_iter()
        .map(|d| Game {
            app_id: d.app_id,
            name: d.name,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_steam_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/steam")
    }

    #[test]
    fn loads_two_games_from_fixture() {
        let games = load_owned_games(&fixture_steam_root(), "123456").unwrap();
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].app_id, 10);
        assert_eq!(games[0].name, "Counter-Strike");
        assert_eq!(games[1].app_id, 220);
    }

    #[test]
    fn missing_fixture_is_user_action() {
        let err = load_owned_games(&fixture_steam_root(), "999999").unwrap_err();
        assert!(matches!(err, Error::UserAction(_)));
    }
}
