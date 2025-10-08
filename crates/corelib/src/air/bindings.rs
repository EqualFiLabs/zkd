//! AIR → runtime gadget bindings/policy.
//! Consumes AirProgram (already validated) and exposes a policy struct.

use crate::air::AirProgram;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentsPolicy {
    pub pedersen: bool,
    pub curve: Option<String>,
    pub no_r_reuse: Option<bool>,
}

/// Bindings: selected hashes/curves and policy flags made explicit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bindings {
    pub commitments: CommitmentsPolicy,
    /// Optionally allow specifying hash for commitments distinct from transcript hash.
    pub hash_id_for_commitments: Option<String>,
}

impl Bindings {
    pub fn from_air(air: &AirProgram) -> Self {
        let ped = air
            .commitments
            .as_ref()
            .map(|c| c.pedersen)
            .unwrap_or(false);
        let curve = air.commitments.as_ref().and_then(|c| c.curve.clone());
        // Default: allow reuse unless program says otherwise (Phase-0)
        let no_r_reuse = Some(false);
        // Hash for commitments: use program hash name if poseidon/rescue/blake3,
        // else fall back to "blake3".
        let hash_id_for_commitments = Some(
            match format!("{:?}", air.meta.hash).to_lowercase().as_str() {
                "blake3" => "blake3".to_string(),
                "poseidon2" => "poseidon2".to_string(),
                "rescue" => "rescue".to_string(),
                other => {
                    let _ = other;
                    "blake3".to_string()
                }
            },
        );

        Self {
            commitments: CommitmentsPolicy {
                pedersen: ped,
                curve,
                no_r_reuse,
            },
            hash_id_for_commitments,
        }
    }
}
