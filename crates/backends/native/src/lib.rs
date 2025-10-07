//! Native backend adapter (scaffold). Implements traits and registers itself.
//! This step uses the real profile_id when building/verifying the proof header.

use zkprov_corelib::backend::{Capabilities, ProverBackend, VerifierBackend};
use zkprov_corelib::errors::RegistryError;
use zkprov_corelib::registry::register_backend;
use zkprov_corelib::{config::Config, proof, validate::validate_config};

#[derive(Debug, Default)]
pub struct NativeBackend;

impl ProverBackend for NativeBackend {
    fn id(&self) -> &'static str {
        "native@0.0"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            fields: vec!["Prime254"],
            hashes: vec!["blake3"],
            fri_arities: vec![2, 4],
            recursion: "none",
            lookups: false,
        }
    }

    /// Produce a deterministic "stub proof" with header+body.
    fn prove_stub(&self) -> Vec<u8> {
        // retained for trait default compatibility; not used below
        b"PROOF\0".to_vec()
    }
}

impl VerifierBackend for NativeBackend {
    fn verify_stub(&self, proof: &[u8]) -> bool {
        // retained for trait default compatibility; not used below
        proof == b"PROOF\0"
    }
}

/// Public API: register this backend.
pub fn register_native_backend() -> Result<(), RegistryError> {
    register_backend(Box::new(NativeBackend), Box::new(NativeBackend))
}

/// Backend-specific helpers used by CLI/SDK (later) and tests (now).

/// Deterministic "prover" stub that uses Config + public_inputs to emit header+body.
pub fn native_prove(config: &Config, public_inputs_json: &str) -> anyhow::Result<Vec<u8>> {
    // 1) validate config against capabilities
    zkprov_corelib::registry::ensure_builtins_registered(); // idempotent
    validate_config(config)?;

    // 2) compute identifiers (hashes for header fields)
    let backend_id_hash = proof::hash64("BACKEND", config.backend_id.as_bytes());
    let profile_id_hash = proof::hash64("PROFILE", config.profile_id.as_bytes());
    let pubio_hash = proof::hash64("PUBIO", public_inputs_json.as_bytes());

    // 3) construct a tiny deterministic body (digest of inputs)
    let body_digest = proof::hash64("BODY", public_inputs_json.as_bytes());
    let body = body_digest.to_le_bytes();

    // 4) build header
    let header = proof::ProofHeader {
        backend_id_hash,
        profile_id_hash,
        pubio_hash,
        body_len: body.len() as u64,
    };

    // 5) assemble
    Ok(proof::assemble_proof(&header, &body))
}

pub fn native_verify(
    config: &Config,
    public_inputs_json: &str,
    proof_bytes: &[u8],
) -> anyhow::Result<bool> {
    zkprov_corelib::registry::ensure_builtins_registered();
    validate_config(config)?;

    if proof_bytes.len() < 40 {
        anyhow::bail!("proof too short");
    }
    let header = proof::ProofHeader::decode(&proof_bytes[0..40])?;
    let body = &proof_bytes[40..];

    // header/body sanity
    if body.len() as u64 != header.body_len {
        anyhow::bail!("body length mismatch");
    }

    // recompute expected fields
    let expect_backend = proof::hash64("BACKEND", config.backend_id.as_bytes());
    if expect_backend != header.backend_id_hash {
        anyhow::bail!("backend id hash mismatch");
    }
    let expect_profile = proof::hash64("PROFILE", config.profile_id.as_bytes());
    if expect_profile != header.profile_id_hash {
        anyhow::bail!("profile id hash mismatch");
    }
    let expect_pubio = proof::hash64("PUBIO", public_inputs_json.as_bytes());
    if expect_pubio != header.pubio_hash {
        anyhow::bail!("public io hash mismatch");
    }
    let expect_body = proof::hash64("BODY", public_inputs_json.as_bytes()).to_le_bytes();
    if body != expect_body {
        anyhow::bail!("body digest mismatch");
    }
    Ok(true)
}
