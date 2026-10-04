pub mod engine;
pub mod meta;
pub mod pack;
pub mod rules;

pub use engine::{run_preset, AutoCatResult};
pub use meta::{GameMetadata, MetadataIndex};
pub use pack::{
    default_user_rule_packs_dir, load_rule_packs, parse_rule_pack_json, ConflictPolicy, RulePack,
};
pub use rules::{AutoCatPreset, AutoCatRule, UserScoreBand, YearMode};
