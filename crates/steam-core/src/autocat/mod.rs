pub mod engine;
pub mod meta;
pub mod pack;
pub mod policy;
pub mod rules;

pub use engine::{run_preset, run_preset_with_options, AchievementIndex, AutoCatResult};
pub use meta::{GameMetadata, MetadataIndex};
pub use pack::{
    default_user_rule_packs_dir, load_rule_packs, parse_rule_pack_json, RulePack,
};
pub use policy::ConflictPolicy;
pub use rules::{
    AchievementMode, AutoCatPreset, AutoCatRule, UserScoreBand, YearMode,
};
