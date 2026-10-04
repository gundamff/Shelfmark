use serde::Deserialize;
use steam_core::{parse_user_id3, steam_id3_to_id64};

#[derive(Debug, Deserialize)]
struct PlayerAchievementsResponse {
    playerstats: Option<PlayerStats>,
}

#[derive(Debug, Deserialize)]
struct PlayerStats {
    success: Option<bool>,
    achievements: Option<Vec<AchievementDto>>,
}

#[derive(Debug, Deserialize)]
struct AchievementDto {
    achieved: u8,
}

/// Fetch achievement unlock ratio for one app (0.0–1.0). Returns None if unavailable.
pub fn fetch_achievement_ratio(
    api_key: &str,
    user_id3: &str,
    app_id: u32,
) -> Result<Option<f32>, String> {
    let id3 = parse_user_id3(user_id3).ok_or_else(|| "invalid user id3".to_string())?;
    let id64 = steam_id3_to_id64(id3);
    let url = format!(
        "https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?key={api_key}&steamid={id64}&appid={app_id}"
    );
    let body: PlayerAchievementsResponse = ureq::get(&url)
        .header("User-Agent", "Shelfmark/0.1")
        .call()
        .map_err(|e| format!("achievements fetch {app_id}: {e}"))?
        .body_mut()
        .read_json()
        .map_err(|e| format!("achievements json {app_id}: {e}"))?;

    let stats = match body.playerstats {
        Some(s) if s.success.unwrap_or(false) => s,
        _ => return Ok(None),
    };
    let list = stats.achievements.unwrap_or_default();
    if list.is_empty() {
        return Ok(None);
    }
    let unlocked = list.iter().filter(|a| a.achieved != 0).count() as f32;
    Ok(Some(unlocked / list.len() as f32))
}
