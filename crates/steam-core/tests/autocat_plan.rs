//! Plan-level locks for the AutoCat pipeline: rule pack -> match -> plan ->
//! preview -> apply-to-queue. The preview and the export queue are both derived
//! from this same changeset, so their counts must never diverge.

use std::collections::{HashMap, HashSet};

use steam_core::autocat::{
    run_preset_with_options, AchievementIndex, AutoCatPreset, AutoCatResult, AutoCatRule,
};
use steam_core::session::Session;
use steam_core::types::{AppId, Collection, Game};
use steam_core::{ConflictPolicy, GameMetadata};

fn game(id: AppId, name: &str) -> Game {
    Game {
        app_id: id,
        name: name.into(),
    }
}

fn genre_preset() -> AutoCatPreset {
    AutoCatPreset {
        name: "genre".into(),
        rules: vec![AutoCatRule::Genre {
            prefix: "g_".into(),
            max_categories: 0,
            remove_existing: false,
        }],
    }
}

fn matches(games: &[Game], meta: &HashMap<AppId, GameMetadata>, existing: &[Collection]) -> AutoCatResult {
    run_preset_with_options(
        games,
        meta,
        &AchievementIndex::new(),
        existing,
        &genre_preset(),
        ConflictPolicy::Merge,
    )
    .unwrap()
}

fn two_genre_games() -> (Vec<Game>, HashMap<AppId, GameMetadata>) {
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
    (games, meta)
}

#[test]
fn matched_games_produce_add_member_ops_for_created_collections() {
    let (games, meta) = two_genre_games();
    let result = matches(&games, &meta, &[]);

    assert_eq!(result.matched_games, 2);
    assert_eq!(result.changeset.create.len(), 2);
    assert!(
        !result.changeset.add_members.is_empty(),
        "matched games must produce add_members operations, not only create[].app_ids"
    );
    let added: HashSet<AppId> = result
        .changeset
        .add_members
        .iter()
        .flat_map(|(_, ids)| ids.iter().copied())
        .collect();
    assert!(added.contains(&10));
    assert!(added.contains(&220));
}

#[test]
fn matched_games_append_to_existing_collections() {
    let (games, meta) = two_genre_games();
    let existing = vec![Collection {
        id: "c1".into(),
        name: "g_Action".into(),
        app_ids: vec![10],
    }];
    let result = matches(&games, &meta, &existing);

    let adds = result
        .changeset
        .add_members
        .iter()
        .find(|(id, _)| id == "c1")
        .map(|(_, ids)| ids.clone())
        .unwrap_or_default();
    assert_eq!(adds, vec![220]);
    assert!(result.changeset.create.iter().any(|c| c.name == "g_Indie"));
}

#[test]
fn zero_member_collections_are_dropped() {
    // A Manual rule with no games still names a collection. It must not
    // produce an empty create (and therefore no phantom "new" collection).
    let preset = AutoCatPreset {
        name: "manual".into(),
        rules: vec![AutoCatRule::Manual {
            add: vec!["Empty".into()],
            remove: vec![],
            app_ids: vec![],
        }],
    };
    let result = run_preset_with_options(
        &[],
        &HashMap::new(),
        &AchievementIndex::new(),
        &[],
        &preset,
        ConflictPolicy::Merge,
    )
    .unwrap();

    assert!(result.changeset.create.is_empty());
    assert!(result.changeset.add_members.is_empty());
}

#[test]
fn preview_totals_equal_queue_totals() {
    use steam_core::types::ChangesetCounts;

    let (games, meta) = two_genre_games();
    let result = matches(&games, &meta, &[]);
    let mut session = Session::new(games, vec![], vec![]);

    let preview: ChangesetCounts = session.preview_autocat(&result);
    let queued: ChangesetCounts = session.queue_autocat(result);

    assert_eq!(preview, queued);
    assert_eq!(queued, session.pending.counts());
    assert_eq!(preview.create, 2);
    assert_eq!(preview.add_members, 2);
    assert_eq!(preview.rename, 0);
    assert_eq!(preview.delete, 0);
    assert_eq!(preview.remove_members, 0);
}

#[test]
fn preview_totals_equal_queue_totals_with_existing_pending() {
    let (games, meta) = two_genre_games();
    let result = matches(&games, &meta, &[]);
    let replay = result.clone();
    let mut session = Session::new(games, vec![], vec![]);
    let manual_id = session.create_collection("Manual").unwrap();
    session.add_games_to_collection(&manual_id, &[10]).unwrap();

    let preview = session.preview_autocat(&result);
    let queued = session.queue_autocat(result);
    assert_eq!(preview, queued);
    assert_eq!(queued, session.pending.counts());

    // "加入待导出" twice must be idempotent: no duplicated create/add ops.
    let replayed = session.queue_autocat(replay);
    assert_eq!(replayed, queued);
    assert_eq!(session.pending.counts(), queued);
}

#[test]
fn queued_changeset_reproduces_every_matched_member() {
    let (games, meta) = two_genre_games();
    let result = matches(&games, &meta, &[]);
    let mut session = Session::new(games, vec![], vec![]);
    session.queue_autocat(result);
    let cols = session.effective_collections().unwrap();

    let action = cols.iter().find(|c| c.name == "g_Action").unwrap();
    assert_eq!(action.app_ids, vec![10, 220]);
    let indie = cols.iter().find(|c| c.name == "g_Indie").unwrap();
    assert_eq!(indie.app_ids, vec![220]);
}
