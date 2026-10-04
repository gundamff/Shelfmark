use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::autocat::rules::{AutoCatPreset, AutoCatRule, UserScoreBand, YearMode};
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    Skip,
    Merge,
    Replace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct I18nStr {
    pub en: String,
    pub zh: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PackRule {
    Genre {
        #[serde(default = "default_prefix_g")]
        prefix: String,
        #[serde(default)]
        max_categories: u32,
        #[serde(default)]
        remove_existing: bool,
    },
    Tags {
        #[serde(default = "default_prefix_t")]
        prefix: String,
        #[serde(default)]
        max_tags_per_game: u32,
        #[serde(default)]
        included: Option<Vec<String>>,
    },
    Year {
        #[serde(default = "default_prefix_y")]
        prefix: String,
        #[serde(default = "default_year_mode")]
        mode: YearMode,
    },
    Flags {
        #[serde(default = "default_prefix_f")]
        prefix: String,
        included: Vec<String>,
    },
    DevPub {
        #[serde(default = "default_prefix_d")]
        prefix: String,
        #[serde(default = "default_true")]
        use_developers: bool,
        #[serde(default = "default_true")]
        use_publishers: bool,
    },
    UserScore {
        bands: Vec<UserScoreBand>,
    },
}

fn default_prefix_g() -> String {
    "g_".into()
}
fn default_prefix_t() -> String {
    "tag_".into()
}
fn default_prefix_y() -> String {
    "y_".into()
}
fn default_prefix_f() -> String {
    "f_".into()
}
fn default_prefix_d() -> String {
    "d_".into()
}
fn default_year_mode() -> YearMode {
    YearMode::Individual
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RulePack {
    pub id: String,
    pub name: I18nStr,
    #[serde(default = "empty_i18n")]
    pub description: I18nStr,
    #[serde(default)]
    pub recommended: bool,
    #[serde(default)]
    pub needs_metadata: bool,
    #[serde(default)]
    pub needs_achievements: bool,
    #[serde(default = "default_conflict")]
    pub default_conflict: ConflictPolicy,
    pub rules: Vec<PackRule>,
}

fn empty_i18n() -> I18nStr {
    I18nStr {
        en: String::new(),
        zh: String::new(),
    }
}

fn default_conflict() -> ConflictPolicy {
    ConflictPolicy::Merge
}

impl RulePack {
    pub fn to_preset(&self) -> Result<AutoCatPreset> {
        let mut rules = Vec::with_capacity(self.rules.len());
        for r in &self.rules {
            rules.push(pack_rule_to_autocat(r)?);
        }
        Ok(AutoCatPreset {
            name: self.id.clone(),
            rules,
        })
    }
}

fn pack_rule_to_autocat(r: &PackRule) -> Result<AutoCatRule> {
    Ok(match r {
        PackRule::Genre {
            prefix,
            max_categories,
            remove_existing,
        } => AutoCatRule::Genre {
            prefix: prefix.clone(),
            max_categories: *max_categories,
            remove_existing: *remove_existing,
        },
        PackRule::Tags {
            prefix,
            max_tags_per_game,
            included,
        } => AutoCatRule::Tags {
            prefix: prefix.clone(),
            max_tags_per_game: *max_tags_per_game,
            included: included.clone(),
        },
        PackRule::Year { prefix, mode } => AutoCatRule::Year {
            prefix: prefix.clone(),
            mode: *mode,
        },
        PackRule::Flags { prefix, included } => AutoCatRule::Flags {
            prefix: prefix.clone(),
            included: included.clone(),
        },
        PackRule::DevPub {
            prefix,
            use_developers,
            use_publishers,
        } => AutoCatRule::DevPub {
            prefix: prefix.clone(),
            use_developers: *use_developers,
            use_publishers: *use_publishers,
        },
        PackRule::UserScore { bands } => AutoCatRule::UserScore {
            bands: bands.clone(),
        },
    })
}

pub fn parse_rule_pack_json(raw: &str) -> Result<RulePack> {
    let pack: RulePack = serde_json::from_str(raw)
        .map_err(|e| Error::Parse(format!("rule pack: {e}")))?;
    if pack.id.trim().is_empty() {
        return Err(Error::Parse("rule pack id empty".into()));
    }
    if pack.rules.is_empty() {
        return Err(Error::Parse("rule pack has no rules".into()));
    }
    // Ensure kinds convert.
    let _ = pack.to_preset()?;
    Ok(pack)
}

pub fn load_rule_packs_from_dir(dir: &Path) -> Result<Vec<RulePack>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut packs = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let raw = fs::read_to_string(&path)?;
        match parse_rule_pack_json(&raw) {
            Ok(p) => packs.push(p),
            Err(e) => {
                // Skip bad user packs but surface via stderr-style log later; for now error soft.
                let _ = e;
            }
        }
    }
    packs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(packs)
}

/// Built-in packs from repo `rule-packs/`, then user dir (same id replaces).
pub fn load_rule_packs(builtin_dir: &Path, user_dir: &Path) -> Result<Vec<RulePack>> {
    let mut map = std::collections::BTreeMap::new();
    for p in load_rule_packs_from_dir(builtin_dir)? {
        map.insert(p.id.clone(), p);
    }
    for p in load_rule_packs_from_dir(user_dir)? {
        map.insert(p.id.clone(), p);
    }
    Ok(map.into_values().collect())
}

pub fn default_user_rule_packs_dir() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        let appdata = std::env::var_os("APPDATA")
            .ok_or_else(|| Error::UserAction("APPDATA not set".into()))?;
        return Ok(PathBuf::from(appdata).join("Shelfmark").join("rule-packs"));
    }
    #[cfg(not(windows))]
    {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| Error::UserAction("HOME not set".into()))?;
        Ok(PathBuf::from(home)
            .join(".local/share/Shelfmark/rule-packs"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_genre_pack() {
        let raw = r#"{
          "id": "genre",
          "name": {"en": "By genre", "zh": "按类型"},
          "needs_metadata": true,
          "rules": [{"kind": "genre", "prefix": "g_"}]
        }"#;
        let p = parse_rule_pack_json(raw).unwrap();
        assert_eq!(p.id, "genre");
        let preset = p.to_preset().unwrap();
        assert_eq!(preset.rules.len(), 1);
    }

    #[test]
    fn rejects_empty_rules() {
        let raw = r#"{"id":"x","name":{"en":"x","zh":"x"},"rules":[]}"#;
        assert!(parse_rule_pack_json(raw).is_err());
    }
}
