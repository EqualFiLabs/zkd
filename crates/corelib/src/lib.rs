//! Core library: registry, profiles, and top-level APIs used by CLI/FFI.

pub mod air;
pub mod air_bindings {
    pub use crate::air::bindings::*;
}
pub mod backend;
pub mod config;
pub mod crypto;
pub mod errors;
pub mod gadgets;
pub mod profile;
pub mod proof;
pub mod registry;
pub mod trace;
pub mod validate;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use profile::{load_all_profiles_or_default, Profile};

static PROFILES: Lazy<Vec<Profile>> = Lazy::new(load_all_profiles_or_default);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub id: String,
    pub lambda_bits: u32,
}

pub fn list_profiles() -> Vec<ProfileInfo> {
    PROFILES
        .iter()
        .map(|p| ProfileInfo {
            id: p.id.clone(),
            lambda_bits: p.lambda_bits,
        })
        .collect()
}

/// Public API (registry-backed)
pub fn list_backends() -> Vec<backend::BackendInfo> {
    registry::ensure_builtins_registered();
    registry::list_backend_infos()
}

pub use validate::validate_config;

/// Version helper for CLI
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profiles_exist() {
        assert!(list_profiles().len() >= 3);
    }
}
