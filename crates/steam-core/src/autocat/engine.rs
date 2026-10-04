use std::collections::{HashMap, HashSet};

use crate::autocat::meta::MetadataIndex;
use crate::autocat::rules::{AutoCatPreset, AutoCatRule, YearMode};
use crate::types::{AppId, Collection, CollectionChangeset, Game};
use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoCatResult {
    pub changeset: CollectionChangeset,
    pub matched_games: u32,
    pub skipped_missing_meta: u32,
}

/// Dry-run AutoCat: produce a changeset without writing Steam files.
pub fn run_preset(
    games: &[Game],
    meta: &MetadataIndex,
    existing: &[Collection],
    preset: &AutoCatPreset,
) -> Result<AutoCatResult> {
    let mut matched_games = 0u32;
    let mut skipped_missing_meta = 0u32;

    // Working membership: id -> (name, members)
    let mut by_id: HashMap<String, (String, HashSet<AppId>)> = HashMap::new();
    let mut name_to_id: HashMap<String, String> = HashMap::new();

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
                        let col_name = format!("{prefix}{genre}");
                        let id = ensure_collection(
                            &col_name,
                            &mut by_id,
                            &mut name_to_id,
                            &mut created_ids,
                        );
                        by_id.get_mut(&id).unwrap().1.insert(game.app_id);
                        any = true;
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
                        by_id.get_mut(&id).unwrap().1.insert(game.app_id);
                        any = true;
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
                    by_id.get_mut(&id).unwrap().1.insert(game.app_id);
                    matched_games += 1;
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
                        by_id.get_mut(&id).unwrap().1.insert(game.app_id);
                        any = true;
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
                        by_id.get_mut(&id).unwrap().1.insert(game.app_id);
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
                    by_id.get_mut(&id).unwrap().1.insert(game.app_id);
                    matched_games += 1;
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
                    let set = &mut by_id.get_mut(&id).unwrap().1;
                    for app in &targets {
                        set.insert(*app);
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

    for id in &created_ids {
        if let Some((name, members)) = by_id.get(id) {
            changeset.create.push(Collection {
                id: id.clone(),
                name: name.clone(),
                app_ids: {
                    let mut v: Vec<AppId> = members.iter().copied().collect();
                    v.sort_unstable();
                    v
                },
            });
        }
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

    // For newly created collections, members are already in create[]; also emit add_members for clarity? Plan said create + add. Keeping members inside create is enough; engine also adds via create.app_ids.

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
}
