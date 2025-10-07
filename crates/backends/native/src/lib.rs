//! Native backend adapter with AIR-aware stub proving.

use zkprov_corelib::air::AirProgram;
use zkprov_corelib::backend::{Capabilities, ProverBackend, VerifierBackend};
use zkprov_corelib::errors::RegistryError;
use zkprov_corelib::registry::register_backend;
use zkprov_corelib::trace::TraceShape;
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
    fn prove_stub(&self) -> Vec<u8> {
        b"PROOF\0".to_vec()
    }
}
impl VerifierBackend for NativeBackend {
    fn verify_stub(&self, proof: &[u8]) -> bool {
        proof == b"PROOF\0"
    }
}

pub fn register_native_backend() -> Result<(), RegistryError> {
    register_backend(Box::new(NativeBackend), Box::new(NativeBackend))
}

/// Deterministic root over AIR+Trace+Inputs (64-bit)
fn fake_trace_root_u64(air: &AirProgram, inputs_json: &str) -> u64 {
    // Mix in salient fields; order matters (stable).
    let mut accum = 0u64;
    let mix = |acc: &mut u64, label: &str, bytes: &[u8]| {
        let h = proof::hash64(label, bytes);
        *acc ^= h.rotate_left(13) ^ h.wrapping_mul(0x9e3779b97f4a7c15);
    };
    let shape = TraceShape::from_air(air);

    mix(&mut accum, "AIR.NAME", air.meta.name.as_bytes());
    mix(&mut accum, "AIR.FIELD", air.meta.field.as_bytes());
    mix(
        &mut accum,
        "AIR.HASH",
        format!("{:?}", air.meta.hash).as_bytes(),
    );
    mix(&mut accum, "TRACE.ROWS", &shape.rows.to_le_bytes());
    mix(&mut accum, "TRACE.COLS", &shape.cols.to_le_bytes());
    mix(&mut accum, "IO.JSON", inputs_json.as_bytes());

    accum
}

/// Prove: AIR-aware deterministic proof.
pub fn native_prove(
    config: &Config,
    public_inputs_json: &str,
    air_path: &str,
) -> anyhow::Result<Vec<u8>> {
    zkprov_corelib::registry::ensure_builtins_registered();
    validate_config(config)?;

    // Load and validate AIR
    let air = AirProgram::load_from_file(air_path)?;

    // Header identifiers
    let backend_id_hash = proof::hash64("BACKEND", config.backend_id.as_bytes());
    let profile_id_hash = proof::hash64("PROFILE", config.profile_id.as_bytes());
    let pubio_hash = proof::hash64("PUBIO", public_inputs_json.as_bytes());

    // Body = fake trace root as 8 bytes
    let root = fake_trace_root_u64(&air, public_inputs_json);
    let body = root.to_le_bytes();

    let header = proof::ProofHeader {
        backend_id_hash,
        profile_id_hash,
        pubio_hash,
        body_len: body.len() as u64,
    };
    Ok(proof::assemble_proof(&header, &body))
}

/// Verify: recompute fake root and compare bytes.
pub fn native_verify(
    config: &Config,
    public_inputs_json: &str,
    air_path: &str,
    proof_bytes: &[u8],
) -> anyhow::Result<bool> {
    zkprov_corelib::registry::ensure_builtins_registered();
    validate_config(config)?;

    let air = AirProgram::load_from_file(air_path)?;

    if proof_bytes.len() < 40 {
        anyhow::bail!("proof too short");
    }
    let header = proof::ProofHeader::decode(&proof_bytes[0..40])?;
    let body = &proof_bytes[40..];

    if body.len() as u64 != header.body_len {
        anyhow::bail!("body length mismatch");
    }

    // Check header bindings
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

    // Check fake root
    let expect_root = fake_trace_root_u64(&air, public_inputs_json).to_le_bytes();
    if body != expect_root {
        anyhow::bail!("fake trace root mismatch");
    }
    Ok(true)
}
