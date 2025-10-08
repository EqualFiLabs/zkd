//! Arithmetic under commitments (placeholder semantics).
//!
//! These helpers operate on our PedersenPlaceholder commitment scheme from 0.7.A.
//! Messages are interpreted as unsigned integers (u64) and encoded canonically
//! as 8-byte little-endian for the purpose of committing.
//!
//! Combining blinds: we derive a new blind deterministically from existing blinds
//! (domain-separated hashing) so recomputed commitments are deterministic.
//!
//! SECURITY NOTE: This is a placeholder over a hash-based commitment; it does NOT
//! preserve homomorphic properties like real Pedersen on elliptic curves would.
//! It is deterministic glue so callers can write flows and tests now, and we'll
//! swap the internals with real curve math later.

use anyhow::{anyhow, Result};

use crate::crypto::registry::hash32_by_id;
use crate::gadgets::commitment::{Comm32, CommitmentScheme32, PedersenPlaceholder, Witness};

/// Canonical encoding of u64 message as 8-byte little endian.
fn enc_u64_le(x: u64) -> [u8; 8] {
    x.to_le_bytes()
}

/// Derive a deterministic blind from two blinds using the scheme's hash id.
fn combine_blinds(hash_id: &str, label: &str, b1: &[u8], b2: &[u8]) -> Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(16 + b1.len() + b2.len());
    buf.extend_from_slice(&(b1.len() as u64).to_le_bytes());
    buf.extend_from_slice(b1);
    buf.extend_from_slice(&(b2.len() as u64).to_le_bytes());
    buf.extend_from_slice(b2);
    let d = hash32_by_id(hash_id, label, &buf)
        .ok_or_else(|| anyhow!("unsupported hash id '{}'", hash_id))?;
    Ok(d.to_vec())
}

/// Re-commit a u64 with given blinding using PedersenPlaceholder.
pub fn commit_u64(ped: &PedersenPlaceholder, x: u64, blind: &[u8]) -> Result<Comm32> {
    ped.commit(&Witness {
        msg: &enc_u64_le(x),
        blind,
    })
}

/// Given C1 = commit(m1, r1), C2 = commit(m2, r2),
/// compute Csum = commit(m1+m2, r12), where r12 = H("PEDERSEN.ADD", r1||r2).
/// Returns (Csum, r12).
pub fn add_under_commit_u64(
    ped: &PedersenPlaceholder,
    m1: u64,
    r1: &[u8],
    m2: u64,
    r2: &[u8],
) -> Result<(Comm32, Vec<u8>)> {
    let sum = m1.wrapping_add(m2);
    let r12 = combine_blinds(ped.hash_id(), "PEDERSEN.ADD", r1, r2)?;
    let c_sum = commit_u64(ped, sum, &r12)?;
    Ok((c_sum, r12))
}

/// Given C = commit(m, r), compute C' = commit(k*m, r'),
/// where r' = H("PEDERSEN.SCALAR", r || k_le).
/// Returns (C', r').
pub fn scalar_mul_under_commit_u64(
    ped: &PedersenPlaceholder,
    m: u64,
    r: &[u8],
    k: u64,
) -> Result<(Comm32, Vec<u8>)> {
    let prod = m.wrapping_mul(k);
    let mut buf = Vec::with_capacity(r.len() + 8);
    buf.extend_from_slice(r);
    buf.extend_from_slice(&enc_u64_le(k));
    let d = hash32_by_id(ped.hash_id(), "PEDERSEN.SCALAR", &buf)
        .ok_or_else(|| anyhow!("unsupported hash id"))?;
    let c_prime = commit_u64(ped, prod, &d)?;
    Ok((c_prime, d.to_vec()))
}
