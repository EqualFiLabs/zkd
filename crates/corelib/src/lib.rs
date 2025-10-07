//! Core library: registry stubs, profiles, and API surface for CLI/FFI.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Public profile info (minimal for scaffold)
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

/// Public backend info (minimal for scaffold)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    pub id: &'static str,
    pub recursion: bool,
}

static DEFAULT_BACKENDS: Lazy<Vec<BackendInfo>> = Lazy::new(|| {
    vec![BackendInfo {
        id: "native@0.0",
        recursion: false,
    }]
});

/// API: list available profiles
pub fn list_profiles() -> &'static [ProfileInfo] {
    DEFAULT_PROFILES.as_slice()
}

/// API: list available backends
pub fn list_backends() -> &'static [BackendInfo] {
    DEFAULT_BACKENDS.as_slice()
}

/// Version helper for CLI
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn has_profiles_and_backends() {
        assert!(!list_profiles().is_empty());
        assert!(!list_backends().is_empty());
    }
}
