pub mod engine;
pub mod meta;
pub mod rules;

pub use engine::{run_preset, AutoCatResult};
pub use meta::{GameMetadata, MetadataIndex};
pub use rules::{AutoCatPreset, AutoCatRule, UserScoreBand, YearMode};
