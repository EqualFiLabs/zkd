use crate::air::AirProgram;
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
    if !caps.fields.contains(&cfg.field.as_str()) {
        return Err(CapabilityError::FieldUnsupported {
            backend_id: cfg.backend_id.clone(),
            field: cfg.field.clone(),
        });
    }

    // Hash
    if !caps.hashes.contains(&cfg.hash.as_str()) {
        return Err(CapabilityError::HashUnsupported {
            backend_id: cfg.backend_id.clone(),
            hash: cfg.hash.clone(),
        });
    }

    // FRI arity
    if !caps.fri_arities.contains(&cfg.fri_arity) {
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

/// Validate program (AIR) commitments against backend capabilities.
/// - If AIR requires pedersen, backend must advertise pedersen=true.
/// - If AIR provides a curve hint, backend.curves must contain it.
pub fn validate_air_against_backend(
    air: &AirProgram,
    backend_id: &str,
) -> Result<(), CapabilityError> {
    let caps = get_caps(backend_id)
        .map_err(|_| CapabilityError::Mismatch(format!("unknown backend '{}'", backend_id)))?;

    if let Some(req) = &air.commitments {
        if req.pedersen && !caps.pedersen {
            return Err(CapabilityError::Mismatch(format!(
                "program requires pedersen commitments but backend '{}' does not support them",
                backend_id
            )));
        }
        if let Some(curve) = &req.curve {
            if !caps.curves.iter().any(|c| *c == curve.as_str()) {
                return Err(CapabilityError::Mismatch(format!(
                    "program requests curve '{}' but backend '{}' supports {:?}",
                    curve, backend_id, caps.curves
                )));
            }
        }
    }
    Ok(())
}
