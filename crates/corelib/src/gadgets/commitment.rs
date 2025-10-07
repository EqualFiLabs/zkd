//! Commitment interfaces and a Pedersen-like placeholder.
//!
//! Placeholder construction (to be swapped for real EC):
//!   C = H_id("PEDERSEN" || m || r)
//! where H_id is resolved from crypto::registry by its string id.
//!
//! API is stable so we can replace internals later with real curve math.

use crate::crypto::registry::hash32_by_id;
use anyhow::{anyhow, Result};

/// 32-byte commitment type
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Comm32(pub [u8; 32]);

impl Comm32 {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Witness for basic commitments: message & blinding (both as bytes)
#[derive(Clone, Debug)]
pub struct Witness<'a> {
    pub msg: &'a [u8],
    pub blind: &'a [u8],
}

/// Simple commitment scheme over 32-byte digests
pub trait CommitmentScheme32 {
    /// Produce a 32-byte commitment
    fn commit(&self, w: &Witness<'_>) -> Result<Comm32>;
    /// Verify opening of commitment
    fn open(&self, w: &Witness<'_>, commitment: &Comm32) -> Result<bool>;
    /// Identifier for the scheme (e.g., "pedersen")
    fn id(&self) -> &'static str;
}

/// Parameters for placeholder Pedersen
#[derive(Clone, Debug)]
pub struct PedersenParams {
    /// Hash function id (e.g., "blake3", "keccak256", "poseidon2", "rescue")
    pub hash_id: String,
}

impl Default for PedersenParams {
    fn default() -> Self {
        Self {
            hash_id: "blake3".to_string(),
        }
    }
}

/// Pedersen-like commitment using domain-separated hash (placeholder).
pub struct PedersenPlaceholder {
    params: PedersenParams,
}

impl PedersenPlaceholder {
    pub fn new(params: PedersenParams) -> Self {
        Self { params }
    }

    fn commit_raw(&self, msg: &[u8], blind: &[u8]) -> Result<[u8; 32]> {
        // H("PEDERSEN" || len(m) || m || len(r) || r)
        // Include lengths to avoid ambiguity, then domain-separated label.
        let mut buf = Vec::with_capacity(16 + msg.len() + blind.len());
        buf.extend_from_slice(&(msg.len() as u64).to_le_bytes());
        buf.extend_from_slice(msg);
        buf.extend_from_slice(&(blind.len() as u64).to_le_bytes());
        buf.extend_from_slice(blind);

        hash32_by_id(&self.params.hash_id, "PEDERSEN", &buf)
            .ok_or_else(|| anyhow!("unsupported hash id '{}'", self.params.hash_id))
    }
}

impl CommitmentScheme32 for PedersenPlaceholder {
    fn commit(&self, w: &Witness<'_>) -> Result<Comm32> {
        Ok(Comm32(self.commit_raw(w.msg, w.blind)?))
    }

    fn open(&self, w: &Witness<'_>, commitment: &Comm32) -> Result<bool> {
        Ok(self.commit_raw(w.msg, w.blind)? == commitment.0)
    }

    fn id(&self) -> &'static str {
        "pedersen"
    }
}
