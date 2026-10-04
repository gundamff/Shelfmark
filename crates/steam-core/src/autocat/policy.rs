use serde::{Deserialize, Serialize};

/// How AutoCat treats collections that already exist by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    /// Leave existing same-named collections untouched.
    Skip,
    /// Append new members (default).
    #[default]
    Merge,
    /// Clear members of touched existing collections, then apply rules.
    Replace,
}
