//! Backend adapter traits and capability model.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capabilities {
    pub fields: Vec<&'static str>, // e.g., ["Goldilocks","BabyBear"]
    pub hashes: Vec<&'static str>, // e.g., ["poseidon2","rescue","blake3"]
    pub fri_arities: Vec<u32>,     // e.g., [2,4,8]
    pub recursion: &'static str,   // "none" | "stark-in-stark" | "snark-wrapper"
    pub lookups: bool,
    /// Named curves supported for Pedersen-style commitments (placeholder for now)
    pub curves: Vec<&'static str>, // e.g., ["placeholder"]
    /// Whether Pedersen-style commitments (and related gadgets) are supported
    pub pedersen: bool,
}

pub trait ProverBackend: Send + Sync {
    fn id(&self) -> &'static str; // "native@0.0" etc.
    fn capabilities(&self) -> Capabilities;
    fn prove_stub(&self) -> Vec<u8> {
        // Placeholder to ensure end-to-end linkage for now; real prove() lands later.
        // Returns a deterministic short "proof" for smoke tests.
        b"PROOF\0".to_vec()
    }
}

pub trait VerifierBackend: Send + Sync {
    fn verify_stub(&self, proof: &[u8]) -> bool {
        proof == b"PROOF\0"
    }
}

/// Public info returned by listing APIs (subset of Capabilities)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    pub id: &'static str,
    pub recursion: bool,
}
