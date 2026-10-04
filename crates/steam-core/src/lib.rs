pub mod error;

pub use error::{Error, Result};

pub fn crate_name() -> &'static str {
    "steam-core"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_is_steam_core() {
        assert_eq!(crate_name(), "steam-core");
    }
}
