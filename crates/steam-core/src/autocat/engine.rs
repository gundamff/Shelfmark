use std::collections::{HashMap, HashSet};

use crate::autocat::meta::MetadataIndex;
use crate::autocat::policy::ConflictPolicy;
use crate::autocat::rules::{AchievementMode, AutoCatPreset, AutoCatRule, YearMode};
use crate::types::{AppId, Collection, CollectionChangeset, Game};
use crate::Result;

/// app_id -> unlock ratio in \[0.0, 1.0\].
pub type AchievementIndex = HashMap<AppId, f32>;

/// Map a raw Steam genre string to one canonical spelling.
///
/// Metadata for the same title arrives either from the store API (English, e.g. "Action")
/// or from older/localised caches (Chinese, e.g. "动作"), so without normalisation AutoCat
/// creates two near-duplicate collections such as `g_Action` and `g_动作` — one of which
/// ends up with a single member. Normalise to the English store spelling and fall back to
/// the trimmed original for anything unknown, so no genre is silently dropped.
pub fn canonical_genre(raw: &str) -> String {
    let key = raw.trim().to_lowercase();
    let canonical = match key.as_str() {
        "action" | "动作" | "动作类" => "Action",
        "adventure" | "冒险" => "Adventure",
        "casual" | "休闲" => "Casual",
        "indie" | "独立" | "独立游戏" => "Indie",
        "massively multiplayer" | "mmo" | "大型多人在线" | "多人在线" => "Massively Multiplayer",
        "racing" | "竞速" | "赛车" => "Racing",
        "rpg" | "role-playing" | "role playing" | "角色扮演" => "RPG",
        "simulation" | "模拟" => "Simulation",
        "sports" | "体育" => "Sports",
        "strategy" | "策略" => "Strategy",
        "early access" | "抢先体验" => "Early Access",
        "free to play" | "f2p" | "免费开玩" | "免费游玩" => "Free To Play",
        "animation & modeling" | "animation and modeling" | "动画与建模" | "动画和建模" => {
            "Animation & Modeling"
        }
        "audio production" | "音频制作" => "Audio Production",
        "design & illustration" | "design and illustration" | "设计与插画" => "Design & Illustration",
        "education" | "教育" => "Education",
        "game development" | "游戏开发" => "Game Development",
        "photo editing" | "照片编辑" => "Photo Editing",
        "software training" | "软件培训" => "Software Training",
        "utilities" | "工具" | "实用工具" => "Utilities",
        "video production" | "视频制作" => "Video Production",
        "web publishing" | "网络出版" => "Web Publishing",
        "sexual content" | "性内容" => "Sexual Content",
        "gore" | "violent" | "血腥" | "暴力" => "Gore",
        _ => return raw.trim().to_string(),
    };
    canonical.to_string()
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoCatResult {
    pub changeset: CollectionChangeset,
    pub matched_games: u32,
    pub skipped_missing_meta: u32,
}

/// Dry-run AutoCat with merge conflict policy and empty achievements.
pub fn run_preset(
    games: &[Game],
    meta: &MetadataIndex,
    existing: &[Collection],
    preset: &AutoCatPreset,
) -> Result<AutoCatResult> {
    run_preset_with_options(
        games,
        meta,
        &AchievementIndex::new(),
        existing,
        preset,
        ConflictPolicy::Merge,
    )
}

/// Dry-run AutoCat with conflict policy and achievement ratios.
pub fn run_preset_with_options(
    games: &[Game],
    meta: &MetadataIndex,
    achievements: &AchievementIndex,
    existing: &[Collection],
    preset: &AutoCatPreset,
    conflict: ConflictPolicy,
) -> Result<AutoCatResult> {
    let mut matched_games = 0u32;
    let mut skipped_missing_meta = 0u32;

    // Working membership: id -> (name, members)
    let mut by_id: HashMap<String, (String, HashSet<AppId>)> = HashMap::new();
    let mut name_to_id: HashMap<String, String> = HashMap::new();
    let preexisting_ids: HashSet<String> = existing.iter().map(|c| c.id.clone()).collect();
    let mut cleared: HashSet<String> = HashSet::new();

    for c in existing {
        let members: HashSet<AppId> = c.app_ids.iter().copied().collect();
        name_to_id.insert(c.name.clone(), c.id.clone());
        by_id.insert(c.id.clone(), (c.name.clone(), members));
    }

    let mut created_ids: Vec<String> = Vec::new();
    let mut deleted_ids: Vec<String> = Vec::new();
    let ensure_collection = |name: &str, by_id: &mut HashMap<String, (String, HashSet<AppId>)>, name_to_id: &mut HashMap<String, String>, created_ids: &mut Vec<String>| -> String {
        if let Some(id) = name_to_id.get(name) {
            return id.clone();
        }
        let id = format!("autocat-{}", slug(name));
        // collision-safe
        let mut final_id = id.clone();
        let mut n = 1u32;
        while by_id.contains_key(&final_id) {
            final_id = format!("{id}-{n}");
            n += 1;
        }
        by_id.insert(final_id.clone(), (name.to_string(), HashSet::new()));
        name_to_id.insert(name.to_string(), final_id.clone());
        created_ids.push(final_id.clone());
        final_id
    };

    for rule in &preset.rules {
        match rule {
            AutoCatRule::Genre {
                prefix,
                max_categories,
                remove_existing,
            } => {
                if *remove_existing {
                    let prefix = prefix.clone();
                    let doomed: Vec<String> = by_id
                        .iter()
                        .filter(|(_, (name, _))| name.starts_with(&prefix))
                        .map(|(id, _)| id.clone())
                        .collect();
                    for id in doomed {
                        if let Some((name, _)) = by_id.remove(&id) {
                            name_to_id.remove(&name);
                            if existing.iter().any(|c| c.id == id) {
                                deleted_ids.push(id);
                            }
                        }
                    }
                }
                for game in games {
                    let Some(m) = meta.get(&game.app_id) else {
                        skipped_missing_meta += 1;
                        continue;
                    };
                    if m.genres.is_empty() {
                        skipped_missing_meta += 1;
                        continue;
                    }
                    let mut any = false;
                    let limit = if *max_categories == 0 {
                        m.genres.len()
                    } else {
                        (*max_categories as usize).min(m.genres.len())
                    };
                    for genre in m.genres.iter().take(limit) {
                        let col_name = format!("{prefix}{}", canonical_genre(genre));
                        let id = ensure_collection(
                            &col_name,
                            &mut by_id,
                            &mut name_to_id,
                            &mut created_ids,
                        );
                        if place_member(
                            &id,
                            game.app_id,
                            &mut by_id,
                            &preexisting_ids,
                            &mut cleared,
                            conflict,
                        ) {
                            any = true;
                        }
                    }
                    if any {
                        matched_games += 1;
                    }
                }
            }
            AutoCatRule::Tags {
                prefix,
                max_tags_per_game,
                included,
            } => {
                for game in games {
                    let Some(m) = meta.get(&game.app_id) else {
                        skipped_missing_meta += 1;
                        continue;
                    };
                    let tags: Vec<&String> = match included {
                        Some(allow) => m
                            .tags
                            .iter()
                            .filter(|t| allow.iter().any(|a| a == *t))
                            .collect(),
                        None => m.tags.iter().collect(),
                    };
                    if tags.is_empty() {
                        skipped_missing_meta += 1;
                        continue;
                    }
                    let limit = if *max_tags_per_game == 0 {
                        tags.len()
                    } else {
                        (*max_tags_per_game as usize).min(tags.len())
                    };
                    let mut any = false;
                    for tag in tags.into_iter().take(limit) {
                        let col_name = format!("{prefix}{tag}");
                        let id = ensure_collection(
                            &col_name,
                            &mut by_id,
                            &mut name_to_id,
                            &mut created_ids,
                        );
                        if place_member(
                            &id,
                            game.app_id,
                            &mut by_id,
                            &preexisting_ids,
                            &mut cleared,
                            conflict,
                        ) {
                            any = true;
                        }
                    }
                    if any {
                        matched_games += 1;
                    }
                }
            }
            AutoCatRule::Year { prefix, mode } => {
                for game in games {
                    let Some(m) = meta.get(&game.app_id) else {
                        skipped_missing_meta += 1;
                        continue;
                    };
                    let Some(year) = m.release_year else {
                        skipped_missing_meta += 1;
                        continue;
                    };
                    let label = match mode {
                        YearMode::Individual => year.to_string(),
                        YearMode::Decade => format!("{}s", year / 10 * 10),
                    };
                    let col_name = format!("{prefix}{label}");
                    let id =
                        ensure_collection(&col_name, &mut by_id, &mut name_to_id, &mut created_ids);
                    if place_member(
                        &id,
                        game.app_id,
                        &mut by_id,
                        &preexisting_ids,
                        &mut cleared,
                        conflict,
                    ) {
                        matched_games += 1;
                    }
                }
            }
            AutoCatRule::Flags { prefix, included } => {
                for game in games {
                    let Some(m) = meta.get(&game.app_id) else {
                        skipped_missing_meta += 1;
                        continue;
                    };
                    let flags: Vec<&String> = m
                        .flags
                        .iter()
                        .filter(|f| included.iter().any(|a| a == *f))
                        .collect();
                    if flags.is_empty() {
                        skipped_missing_meta += 1;
                        continue;
                    }
                    let mut any = false;
                    for flag in flags {
                        let col_name = format!("{prefix}{flag}");
                        let id = ensure_collection(
                            &col_name,
                            &mut by_id,
                            &mut name_to_id,
                            &mut created_ids,
                        );
                        if place_member(
                            &id,
                            game.app_id,
                            &mut by_id,
                            &preexisting_ids,
                            &mut cleared,
                            conflict,
                        ) {
                            any = true;
                        }
                    }
                    if any {
                        matched_games += 1;
                    }
                }
            }
            AutoCatRule::DevPub {
                prefix,
                use_developers,
                use_publishers,
            } => {
                for game in games {
                    let Some(m) = meta.get(&game.app_id) else {
                        skipped_missing_meta += 1;
                        continue;
                    };
                    let mut names: Vec<String> = Vec::new();
                    if *use_developers {
                        names.extend(m.developers.iter().cloned());
                    }
                    if *use_publishers {
                        names.extend(m.publishers.iter().cloned());
                    }
                    names.sort();
                    names.dedup();
                    if names.is_empty() {
                        skipped_missing_meta += 1;
                        continue;
                    }
                    for name in names {
                        let col_name = format!("{prefix}{name}");
                        let id = ensure_collection(
                            &col_name,
                            &mut by_id,
                            &mut name_to_id,
                            &mut created_ids,
                        );
                        let _ = place_member(
                            &id,
                            game.app_id,
                            &mut by_id,
                            &preexisting_ids,
                            &mut cleared,
                            conflict,
                        );
                    }
                    matched_games += 1;
                }
            }
            AutoCatRule::UserScore { bands } => {
                for game in games {
                    let Some(m) = meta.get(&game.app_id) else {
                        skipped_missing_meta += 1;
                        continue;
                    };
                    let Some(score) = m.user_score_percent else {
                        skipped_missing_meta += 1;
                        continue;
                    };
                    let Some(band) = bands
                        .iter()
                        .find(|b| score >= b.min_inclusive && score <= b.max_inclusive)
                    else {
                        continue;
                    };
                    let id = ensure_collection(
                        &band.name,
                        &mut by_id,
                        &mut name_to_id,
                        &mut created_ids,
                    );
                    if place_member(
                        &id,
                        game.app_id,
                        &mut by_id,
                        &preexisting_ids,
                        &mut cleared,
                        conflict,
                    ) {
                        matched_games += 1;
                    }
                }
            }
            AutoCatRule::Achievements { prefix, mode } => {
                let label = match mode {
                    AchievementMode::AllUnlocked => "100%",
                    AchievementMode::NoneUnlocked => "0%",
                    AchievementMode::Partial => "partial",
                };
                let col_name = format!("{prefix}{label}");
                for game in games {
                    let Some(&ratio) = achievements.get(&game.app_id) else {
                        skipped_missing_meta += 1;
                        continue;
                    };
                    let matches = match mode {
                        AchievementMode::AllUnlocked => ratio >= 1.0,
                        AchievementMode::NoneUnlocked => ratio <= 0.0,
                        AchievementMode::Partial => ratio > 0.0 && ratio < 1.0,
                    };
                    if !matches {
                        continue;
                    }
                    let id = ensure_collection(
                        &col_name,
                        &mut by_id,
                        &mut name_to_id,
                        &mut created_ids,
                    );
                    if place_member(
                        &id,
                        game.app_id,
                        &mut by_id,
                        &preexisting_ids,
                        &mut cleared,
                        conflict,
                    ) {
                        matched_games += 1;
                    }
                }
            }
            AutoCatRule::Manual {
                add,
                remove,
                app_ids,
            } => {
                let targets: Vec<AppId> = if app_ids.is_empty() {
                    games.iter().map(|g| g.app_id).collect()
                } else {
                    app_ids.clone()
                };
                for name in add {
                    let id =
                        ensure_collection(name, &mut by_id, &mut name_to_id, &mut created_ids);
                    for app in &targets {
                        let _ = place_member(
                            &id,
                            *app,
                            &mut by_id,
                            &preexisting_ids,
                            &mut cleared,
                            conflict,
                        );
                    }
                }
                for name in remove {
                    if let Some(id) = name_to_id.get(name).cloned() {
                        if let Some((_, set)) = by_id.get_mut(&id) {
                            for app in &targets {
                                set.remove(app);
                            }
                        }
                    }
                }
                if !targets.is_empty() {
                    matched_games += targets.len() as u32;
                }
            }
        }
    }

    // Diff against existing → changeset
    let existing_map: HashMap<String, &Collection> =
        existing.iter().map(|c| (c.id.clone(), c)).collect();

    let mut changeset = CollectionChangeset::default();
    changeset.delete.extend(deleted_ids);

    // The changeset is the single source of truth for both the preview and the
    // export queue. A created collection carries its members *and* emits the
    // matching add_members op, so matched games always show up as add ops.
    // Collections that ended up with no members are dropped instead of being
    // created empty (e.g. a Manual rule naming a collection with no games).
    for id in &created_ids {
        let Some((name, members)) = by_id.get(id) else {
            continue;
        };
        if members.is_empty() {
            continue;
        }
        let mut app_ids: Vec<AppId> = members.iter().copied().collect();
        app_ids.sort_unstable();
        changeset.add_members.push((id.clone(), app_ids.clone()));
        changeset.create.push(Collection {
            id: id.clone(),
            name: name.clone(),
            app_ids,
        });
    }

    for (id, (name, members)) in &by_id {
        if created_ids.contains(id) {
            continue;
        }
        let Some(old) = existing_map.get(id) else {
            continue;
        };
        if &old.name != name {
            changeset.rename.push((id.clone(), name.clone()));
        }
        let old_set: HashSet<AppId> = old.app_ids.iter().copied().collect();
        let add: Vec<AppId> = members.difference(&old_set).copied().collect();
        let remove: Vec<AppId> = old_set.difference(members).copied().collect();
        if !add.is_empty() {
            let mut add = add;
            add.sort_unstable();
            changeset.add_members.push((id.clone(), add));
        }
        if !remove.is_empty() {
            let mut remove = remove;
            remove.sort_unstable();
            changeset.remove_members.push((id.clone(), remove));
        }
    }

    Ok(AutoCatResult {
        changeset,
        matched_games,
        skipped_missing_meta,
    })
}

fn slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

fn place_member(
    id: &str,
    app_id: AppId,
    by_id: &mut HashMap<String, (String, HashSet<AppId>)>,
    preexisting_ids: &HashSet<String>,
    cleared: &mut HashSet<String>,
    conflict: ConflictPolicy,
) -> bool {
    match conflict {
        ConflictPolicy::Skip if preexisting_ids.contains(id) => false,
        ConflictPolicy::Replace if preexisting_ids.contains(id) => {
            if cleared.insert(id.to_string()) {
                if let Some((_, set)) = by_id.get_mut(id) {
                    set.clear();
                }
            }
            if let Some((_, set)) = by_id.get_mut(id) {
                set.insert(app_id);
                true
            } else {
                false
            }
        }
        _ => {
            if let Some((_, set)) = by_id.get_mut(id) {
                set.insert(app_id);
                true
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autocat::meta::GameMetadata;
    use crate::autocat::rules::{AutoCatRule, UserScoreBand, YearMode};
    use crate::types::Game;
    use std::collections::HashMap;

    fn game(id: u32, name: &str) -> Game {
        Game {
            app_id: id,
            name: name.into(),
        }
    }

    #[test]
    fn genre_creates_collections_and_adds_members() {
        let games = vec![game(10, "CS"), game(220, "HL2")];
        let mut meta = HashMap::new();
        meta.insert(
            10,
            GameMetadata {
                genres: vec!["Action".into()],
                ..Default::default()
            },
        );
        meta.insert(
            220,
            GameMetadata {
                genres: vec!["Action".into(), "Indie".into()],
                ..Default::default()
            },
        );
        let preset = AutoCatPreset {
            name: "g".into(),
            rules: vec![AutoCatRule::Genre {
                prefix: "g_".into(),
                max_categories: 0,
                remove_existing: false,
            }],
        };
        let result = run_preset(&games, &meta, &[], &preset).unwrap();
        assert!(result.matched_games >= 2);
        assert_eq!(result.skipped_missing_meta, 0);
        assert!(result.changeset.create.iter().any(|c| c.name == "g_Action"));
        assert!(result.changeset.create.iter().any(|c| c.name == "g_Indie"));
        let action = result
            .changeset
            .create
            .iter()
            .find(|c| c.name == "g_Action")
            .unwrap();
        assert!(action.app_ids.contains(&10));
        assert!(action.app_ids.contains(&220));
    }

    #[test]
    fn conflict_skip_leaves_existing_collection() {
        use crate::autocat::policy::ConflictPolicy;
        use crate::types::Collection;

        let games = vec![game(10, "CS"), game(220, "HL2")];
        let mut meta = HashMap::new();
        meta.insert(
            10,
            GameMetadata {
                genres: vec!["Action".into()],
                ..Default::default()
            },
        );
        meta.insert(
            220,
            GameMetadata {
                genres: vec!["Action".into()],
                ..Default::default()
            },
        );
        let existing = vec![Collection {
            id: "c1".into(),
            name: "g_Action".into(),
            app_ids: vec![10],
        }];
        let preset = AutoCatPreset {
            name: "g".into(),
            rules: vec![AutoCatRule::Genre {
                prefix: "g_".into(),
                max_categories: 0,
                remove_existing: false,
            }],
        };
        let result = run_preset_with_options(
            &games,
            &meta,
            &AchievementIndex::new(),
            &existing,
            &preset,
            ConflictPolicy::Skip,
        )
        .unwrap();
        assert!(result.changeset.add_members.is_empty());
        assert!(result.changeset.remove_members.is_empty());
    }

    #[test]
    fn conflict_replace_clears_then_fills() {
        use crate::autocat::policy::ConflictPolicy;
        use crate::types::Collection;

        let games = vec![game(220, "HL2")];
        let mut meta = HashMap::new();
        meta.insert(
            220,
            GameMetadata {
                genres: vec!["Action".into()],
                ..Default::default()
            },
        );
        let existing = vec![Collection {
            id: "c1".into(),
            name: "g_Action".into(),
            app_ids: vec![10, 99],
        }];
        let preset = AutoCatPreset {
            name: "g".into(),
            rules: vec![AutoCatRule::Genre {
                prefix: "g_".into(),
                max_categories: 0,
                remove_existing: false,
            }],
        };
        let result = run_preset_with_options(
            &games,
            &meta,
            &AchievementIndex::new(),
            &existing,
            &preset,
            ConflictPolicy::Replace,
        )
        .unwrap();
        let removes = result
            .changeset
            .remove_members
            .iter()
            .find(|(id, _)| id == "c1")
            .map(|(_, apps)| apps.clone())
            .unwrap_or_default();
        assert!(removes.contains(&10) && removes.contains(&99));
        let adds = result
            .changeset
            .add_members
            .iter()
            .find(|(id, _)| id == "c1")
            .map(|(_, apps)| apps.clone())
            .unwrap_or_default();
        assert_eq!(adds, vec![220]);
    }

    #[test]
    fn achievements_all_unlocked() {
        use crate::autocat::policy::ConflictPolicy;
        use crate::autocat::rules::AchievementMode;

        let games = vec![game(1, "A"), game(2, "B")];
        let meta = HashMap::new();
        let mut ach = AchievementIndex::new();
        ach.insert(1, 1.0);
        ach.insert(2, 0.5);
        let preset = AutoCatPreset {
            name: "a".into(),
            rules: vec![AutoCatRule::Achievements {
                prefix: "ach_".into(),
                mode: AchievementMode::AllUnlocked,
            }],
        };
        let result = run_preset_with_options(
            &games,
            &meta,
            &ach,
            &[],
            &preset,
            ConflictPolicy::Merge,
        )
        .unwrap();
        assert_eq!(result.matched_games, 1);
        assert!(result
            .changeset
            .create
            .iter()
            .any(|c| c.name == "ach_100%" && c.app_ids == vec![1]));
    }

    #[test]
    fn achievements_none_and_partial_modes() {
        use crate::autocat::policy::ConflictPolicy;
        use crate::autocat::rules::AchievementMode;

        let games = vec![game(1, "None"), game(2, "Partial"), game(3, "All")];
        let meta = HashMap::new();
        let mut ach = AchievementIndex::new();
        ach.insert(1, 0.0);
        ach.insert(2, 0.5);
        ach.insert(3, 1.0);
        let preset = AutoCatPreset {
            name: "a".into(),
            rules: vec![
                AutoCatRule::Achievements {
                    prefix: "none_".into(),
                    mode: AchievementMode::NoneUnlocked,
                },
                AutoCatRule::Achievements {
                    prefix: "part_".into(),
                    mode: AchievementMode::Partial,
                },
            ],
        };
        let result = run_preset_with_options(
            &games,
            &meta,
            &ach,
            &[],
            &preset,
            ConflictPolicy::Merge,
        )
        .unwrap();
        assert!(result
            .changeset
            .create
            .iter()
            .any(|c| c.name == "none_0%" && c.app_ids == vec![1]));
        assert!(result
            .changeset
            .create
            .iter()
            .any(|c| c.name == "part_partial" && c.app_ids == vec![2]));
        assert_eq!(result.matched_games, 2);
    }

    #[test]
    fn genre_skips_missing_meta() {
        let games = vec![game(1, "A"), game(2, "B")];
        let mut meta = HashMap::new();
        meta.insert(1, GameMetadata::default());
        let preset = AutoCatPreset {
            name: "g".into(),
            rules: vec![AutoCatRule::Genre {
                prefix: String::new(),
                max_categories: 0,
                remove_existing: false,
            }],
        };
        let result = run_preset(&games, &meta, &[], &preset).unwrap();
        assert!(result.skipped_missing_meta >= 2);
    }

    #[test]
    fn tags_year_flags_rules() {
        let games = vec![game(5, "X")];
        let mut meta = HashMap::new();
        meta.insert(
            5,
            GameMetadata {
                tags: vec!["Roguelike".into(), "Pixel".into()],
                flags: vec!["Single-player".into(), "Steam Cloud".into()],
                release_year: Some(2015),
                ..Default::default()
            },
        );
        let preset = AutoCatPreset {
            name: "mix".into(),
            rules: vec![
                AutoCatRule::Tags {
                    prefix: "t_".into(),
                    max_tags_per_game: 1,
                    included: None,
                },
                AutoCatRule::Year {
                    prefix: "y_".into(),
                    mode: YearMode::Decade,
                },
                AutoCatRule::Flags {
                    prefix: "f_".into(),
                    included: vec!["Single-player".into()],
                },
            ],
        };
        let result = run_preset(&games, &meta, &[], &preset).unwrap();
        let names: Vec<_> = result.changeset.create.iter().map(|c| c.name.as_str()).collect();
        assert!(names.contains(&"t_Roguelike"));
        assert!(!names.contains(&"t_Pixel")); // max 1
        assert!(names.contains(&"y_2010s"));
        assert!(names.contains(&"f_Single-player"));
        assert!(!names.contains(&"f_Steam Cloud"));
    }

    #[test]
    fn devpub_userscore_manual() {
        let games = vec![game(7, "G"), game(8, "H")];
        let mut meta = HashMap::new();
        meta.insert(
            7,
            GameMetadata {
                developers: vec!["Valve".into()],
                publishers: vec!["Valve".into()],
                user_score_percent: Some(95),
                ..Default::default()
            },
        );
        meta.insert(
            8,
            GameMetadata {
                user_score_percent: Some(40),
                ..Default::default()
            },
        );
        let preset = AutoCatPreset {
            name: "mix2".into(),
            rules: vec![
                AutoCatRule::DevPub {
                    prefix: "d_".into(),
                    use_developers: true,
                    use_publishers: false,
                },
                AutoCatRule::UserScore {
                    bands: vec![
                        UserScoreBand {
                            name: "Great".into(),
                            min_inclusive: 80,
                            max_inclusive: 100,
                        },
                        UserScoreBand {
                            name: "Meh".into(),
                            min_inclusive: 0,
                            max_inclusive: 59,
                        },
                    ],
                },
                AutoCatRule::Manual {
                    add: vec!["Keepers".into()],
                    remove: vec![],
                    app_ids: vec![7],
                },
            ],
        };
        let result = run_preset(&games, &meta, &[], &preset).unwrap();
        let names: Vec<_> = result.changeset.create.iter().map(|c| c.name.clone()).collect();
        assert!(names.iter().any(|n| n == "d_Valve"));
        assert!(names.iter().any(|n| n == "Great"));
        assert!(names.iter().any(|n| n == "Meh"));
        assert!(names.iter().any(|n| n == "Keepers"));
        let keepers = result
            .changeset
            .create
            .iter()
            .find(|c| c.name == "Keepers")
            .unwrap();
        assert_eq!(keepers.app_ids, vec![7]);
    }

    #[test]
    fn canonical_genre_folds_localised_spellings() {
        assert_eq!(canonical_genre("Action"), "Action");
        assert_eq!(canonical_genre("动作"), "Action");
        assert_eq!(canonical_genre("  动作 "), "Action");
        assert_eq!(canonical_genre("RPG"), "RPG");
        assert_eq!(canonical_genre("角色扮演"), "RPG");
        assert_eq!(canonical_genre("Simulation"), "Simulation");
        assert_eq!(canonical_genre("模拟"), "Simulation");
        assert_eq!(canonical_genre("免费开玩"), "Free To Play");
        assert_eq!(canonical_genre("大型多人在线"), "Massively Multiplayer");
        // 未收录的取值原样保留，不能被丢掉
        assert_eq!(canonical_genre("Something New"), "Something New");
        assert_eq!(canonical_genre("  "), "");
    }

    #[test]
    fn genre_rule_merges_duplicate_localisations() {
        let games = vec![game(1, "Localised"), game(2, "English")];
        let mut meta = HashMap::new();
        meta.insert(
            1,
            GameMetadata {
                genres: vec!["动作".into(), "模拟".into(), "免费开玩".into()],
                ..Default::default()
            },
        );
        meta.insert(
            2,
            GameMetadata {
                genres: vec!["Action".into(), "Simulation".into(), "Free To Play".into()],
                ..Default::default()
            },
        );
        let preset = AutoCatPreset {
            name: "g".into(),
            rules: vec![AutoCatRule::Genre {
                prefix: "g_".into(),
                max_categories: 0,
                remove_existing: false,
            }],
        };
        let result = run_preset(&games, &meta, &[], &preset).unwrap();
        let names: Vec<_> = result.changeset.create.iter().map(|c| c.name.clone()).collect();
        assert_eq!(names.len(), 3, "localised and english genres must merge: {names:?}");
        for expected in ["g_Action", "g_Simulation", "g_Free To Play"] {
            let col = result
                .changeset
                .create
                .iter()
                .find(|c| c.name == expected)
                .unwrap_or_else(|| panic!("missing {expected}: {names:?}"));
            assert_eq!(col.app_ids.len(), 2, "{expected} should hold both games");
        }
    }

    #[test]
    fn genre_remove_existing_deletes_duplicate_localisations_and_rebuilds_canonical() {
        use crate::types::Collection;

        let games = vec![game(1, "G1"), game(2, "G2")];
        let mut meta = HashMap::new();
        meta.insert(
            1,
            GameMetadata {
                genres: vec!["动作".into()],
                ..Default::default()
            },
        );
        meta.insert(
            2,
            GameMetadata {
                genres: vec!["Action".into()],
                ..Default::default()
            },
        );
        let existing = vec![
            Collection {
                id: "old-zh".into(),
                name: "g_动作".into(),
                app_ids: vec![1],
            },
            Collection {
                id: "old-en".into(),
                name: "g_Action".into(),
                app_ids: vec![99],
            },
        ];
        let preset = AutoCatPreset {
            name: "g".into(),
            rules: vec![AutoCatRule::Genre {
                prefix: "g_".into(),
                max_categories: 0,
                remove_existing: true,
            }],
        };
        let result = run_preset(&games, &meta, &existing, &preset).unwrap();
        assert!(
            result.changeset.delete.iter().any(|id| id == "old-zh"),
            "expected g_动作 collection to be deleted: {:?}",
            result.changeset.delete
        );
        assert!(
            result.changeset.delete.iter().any(|id| id == "old-en"),
            "expected g_Action collection to be deleted: {:?}",
            result.changeset.delete
        );
        let action_creates: Vec<_> = result
            .changeset
            .create
            .iter()
            .filter(|c| c.name == "g_Action")
            .collect();
        assert_eq!(action_creates.len(), 1, "only one canonical g_Action should be rebuilt: {:?}", result.changeset.create);
        assert!(!result.changeset.create.iter().any(|c| c.name == "g_动作"));
        assert!(action_creates[0].app_ids.contains(&1));
        assert!(action_creates[0].app_ids.contains(&2));
    }
}
