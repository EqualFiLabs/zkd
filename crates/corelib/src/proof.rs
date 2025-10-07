//! Deterministic proof header + body format and helpers.

use anyhow::{bail, Result};
use blake3::Hasher;
use serde::{Deserialize, Serialize};

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
        if &bytes[0..4] != MAGIC {
            bail!("bad magic");
        }

        let mut ver_bytes = [0u8; 4];
        ver_bytes.copy_from_slice(&bytes[4..8]);
        let ver = u32::from_le_bytes(ver_bytes);
        if ver != VERSION {
            bail!("unsupported proof version {ver}");
        }

        let mut backend_bytes = [0u8; 8];
        backend_bytes.copy_from_slice(&bytes[8..16]);
        let backend_id_hash = u64::from_le_bytes(backend_bytes);

        let mut profile_bytes = [0u8; 8];
        profile_bytes.copy_from_slice(&bytes[16..24]);
        let profile_id_hash = u64::from_le_bytes(profile_bytes);

        let mut pubio_bytes = [0u8; 8];
        pubio_bytes.copy_from_slice(&bytes[24..32]);
        let pubio_hash = u64::from_le_bytes(pubio_bytes);

        let mut body_len_bytes = [0u8; 8];
        body_len_bytes.copy_from_slice(&bytes[32..40]);
        let body_len = u64::from_le_bytes(body_len_bytes);

        Ok(ProofHeader {
            backend_id_hash,
            profile_id_hash,
            pubio_hash,
            body_len,
        })
    }
}

/// Deterministic 64-bit hash helpers (BLAKE3 truncated).
pub fn hash64(label: &str, data: &[u8]) -> u64 {
    let mut h = Hasher::new();
    h.update(label.as_bytes());
    h.update(data);
    let mut out8 = [0u8; 8];
    out8.copy_from_slice(&h.finalize().as_bytes()[0..8]);
    u64::from_le_bytes(out8)
}

/// Encode full proof: header(40) + body
pub fn assemble_proof(header: &ProofHeader, body: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(40 + body.len());
    v.extend_from_slice(&header.encode());
    v.extend_from_slice(body);
    v
}
