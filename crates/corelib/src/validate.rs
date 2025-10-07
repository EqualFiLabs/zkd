use crate::backend::Capabilities;
use crate::config::Config;
use crate::errors::{CapabilityError, RegistryError};
use crate::profile::load_all_profiles;
use crate::registry;

fn get_caps(backend_id: &str) -> Result<Capabilities, RegistryError> {
    registry::get_backend_capabilities(backend_id)
}

/// Validate a desired Config against a backend's capabilities.
/// Returns Ok(()) if fully compatible; otherwise a precise CapabilityError.
pub fn validate_config(cfg: &Config) -> Result<(), CapabilityError> {
    let caps = get_caps(&cfg.backend_id)
        .map_err(|_| CapabilityError::Mismatch(format!("unknown backend '{}'", cfg.backend_id)))?;

    // Field
    if !caps.fields.iter().any(|f| *f == cfg.field.as_str()) {
        return Err(CapabilityError::FieldUnsupported {
            backend_id: cfg.backend_id.clone(),
            field: cfg.field.clone(),
        });
    }

    // Hash
    if !caps.hashes.iter().any(|h| *h == cfg.hash.as_str()) {
        return Err(CapabilityError::HashUnsupported {
            backend_id: cfg.backend_id.clone(),
            hash: cfg.hash.clone(),
        });
    }

    // FRI arity
    if !caps.fri_arities.iter().any(|a| *a == cfg.fri_arity) {
        return Err(CapabilityError::FriArityUnsupported {
            backend_id: cfg.backend_id.clone(),
            fri_arity: cfg.fri_arity,
        });
    }

    // Recursion if needed
    if cfg.recursion_needed && caps.recursion == "none" {
        return Err(CapabilityError::RecursionUnavailable {
            backend_id: cfg.backend_id.clone(),
        });
    }

    // Profile existence
    let profiles = load_all_profiles().map_err(|e| CapabilityError::Mismatch(e.to_string()))?;
    if !profiles.iter().any(|p| p.id == cfg.profile_id) {
        return Err(CapabilityError::ProfileNotFound(cfg.profile_id.clone()));
    }

    Ok(())
}
