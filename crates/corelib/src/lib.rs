//! Core library: registry, profiles, and top-level APIs used by CLI/FFI.

pub mod backend;
pub mod errors;
pub mod registry;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// --- Profiles (keep minimal for scaffold; full profiles land later) ---
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub id: &'static str,
    pub lambda_bits: u32,
}

static DEFAULT_PROFILES: Lazy<Vec<ProfileInfo>> = Lazy::new(|| {
    vec![
        ProfileInfo {
            id: "dev-fast",
            lambda_bits: 80,
        },
        ProfileInfo {
            id: "balanced",
            lambda_bits: 100,
        },
        ProfileInfo {
            id: "secure",
            lambda_bits: 120,
        },
    ]
});

/// Public API
pub fn list_profiles() -> &'static [ProfileInfo] {
    DEFAULT_PROFILES.as_slice()
}

/// Public API (registry-backed)
pub fn list_backends() -> Vec<backend::BackendInfo> {
    registry::ensure_builtins_registered();
    registry::list_backend_infos()
}

/// Version helper for CLI
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profiles_exist() {
        assert_eq!(list_profiles().len(), 3);
    }
}
