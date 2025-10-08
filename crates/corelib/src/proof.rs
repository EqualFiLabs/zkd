//! Deterministic proof header + body format and helpers.

use std::convert::TryInto;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

// Centralized header hashing policy.
// For Phase-0, we keep BLAKE3 for stability. If policy changes, switch here.
pub const HEADER_HASH_ID: &str = "blake3";

use crate::crypto::registry;

/// Magic/version
pub const MAGIC: [u8; 4] = *b"PROF";
pub const VERSION: u32 = 1;

/// Fixed-size header (little endian).
/// Layout (bytes):
/// 0..4   MAGIC "PROF"
/// 4..8   VERSION (u32)
/// 8..16  backend_id_hash (u64)
///16..24  profile_id_hash (u64)
///24..32  pubio_hash (u64)     -- hash of canonical public inputs JSON
///32..40  body_len (u64)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofHeader {
    pub backend_id_hash: u64,
    pub profile_id_hash: u64,
    pub pubio_hash: u64,
    pub body_len: u64,
}

impl ProofHeader {
    pub fn encode(&self) -> [u8; 40] {
        let mut out = [0u8; 40];
        out[0..4].copy_from_slice(&MAGIC);
        out[4..8].copy_from_slice(&VERSION.to_le_bytes());
        out[8..16].copy_from_slice(&self.backend_id_hash.to_le_bytes());
        out[16..24].copy_from_slice(&self.profile_id_hash.to_le_bytes());
        out[24..32].copy_from_slice(&self.pubio_hash.to_le_bytes());
        out[32..40].copy_from_slice(&self.body_len.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 40 {
            bail!("proof too short for header");
        }
        if bytes[0..4] != MAGIC {
            bail!("bad magic");
        }
        let ver = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
        if ver != VERSION {
            bail!("unsupported proof version {ver}");
        }
        let backend_id_hash = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let profile_id_hash = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        let pubio_hash = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
        let body_len = u64::from_le_bytes(bytes[32..40].try_into().unwrap());

        Ok(ProofHeader {
            backend_id_hash,
            profile_id_hash,
            pubio_hash,
            body_len,
        })
    }
}

/// Header hashing helper (64-bit), using the centralized policy.
/// Currently equivalent to BLAKE3(label || data), truncated to 64 bits LE.
pub fn hash64(label: &str, data: &[u8]) -> u64 {
    registry::hash64_by_id(HEADER_HASH_ID, label, data).expect("HEADER_HASH_ID must be supported")
}

/// Encode full proof: header(40) + body
pub fn assemble_proof(header: &ProofHeader, body: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(40 + body.len());
    v.extend_from_slice(&header.encode());
    v.extend_from_slice(body);
    v
}
