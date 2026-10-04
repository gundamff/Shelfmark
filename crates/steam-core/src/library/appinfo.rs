use std::path::Path;

use crate::autocat::GameMetadata;
use crate::types::AppId;

/// Best-effort local Steam metadata lookup.
///
/// Full `appinfo.vdf` parsing is version-sensitive; this MVP returns `None`
/// so the Store provider remains the fill path. Hook real parsing here later.
pub fn metadata_from_local_steam(
    _steam_root: &Path,
    _app_id: AppId,
) -> Option<GameMetadata> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn stub_returns_none() {
        assert!(metadata_from_local_steam(Path::new("."), 0).is_none());
    }
}
