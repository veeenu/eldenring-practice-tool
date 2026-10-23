use std::sync::OnceLock;

pub use crate::prelude::base_addresses::Version;

static VERSION: OnceLock<Version> = OnceLock::new();

/// Ensures that the VERSION static gets filled, or returns an error.
/// The caller MUST exit cleanly in case of an error.
pub fn check_version() -> Result<Version, (u32, u32, u32)> {
    practice_tool_core_windows::version::check_version(&VERSION, "Elden Ring Practice Tool")
}

pub fn get_version() -> Version {
    VERSION.get().copied().expect("Game version not found")
}
