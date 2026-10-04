/// Convert SteamID3 (account id) to SteamID64.
pub fn steam_id3_to_id64(id3: u64) -> u64 {
    id3.saturating_add(76561197960265728)
}

pub fn parse_user_id3(s: &str) -> Option<u64> {
    s.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id3_to_id64_formula() {
        assert_eq!(steam_id3_to_id64(136330193), 76561197960265728 + 136330193);
    }
}
