use serde::{Deserialize, Serialize};

use crate::types::AppId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YearMode {
    Individual,
    Decade,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserScoreBand {
    pub name: String,
    pub min_inclusive: u8,
    pub max_inclusive: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AutoCatRule {
    Genre {
        prefix: String,
        /// 0 = unlimited
        max_categories: u32,
        remove_existing: bool,
    },
    Tags {
        prefix: String,
        /// 0 = unlimited
        max_tags_per_game: u32,
        /// If Some, only these tags; if None, all tags (up to max).
        included: Option<Vec<String>>,
    },
    Year {
        prefix: String,
        mode: YearMode,
    },
    Flags {
        prefix: String,
        included: Vec<String>,
    },
    DevPub {
        prefix: String,
        use_developers: bool,
        use_publishers: bool,
    },
    UserScore {
        bands: Vec<UserScoreBand>,
    },
    Manual {
        add: Vec<String>,
        remove: Vec<String>,
        /// Empty = all games passed to the engine.
        app_ids: Vec<AppId>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoCatPreset {
    pub name: String,
    pub rules: Vec<AutoCatRule>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_holds_rules() {
        let p = AutoCatPreset {
            name: "default".into(),
            rules: vec![AutoCatRule::Genre {
                prefix: "g_".into(),
                max_categories: 0,
                remove_existing: false,
            }],
        };
        assert_eq!(p.rules.len(), 1);
    }
}
