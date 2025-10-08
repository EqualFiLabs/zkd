//! AddUnderCommit over placeholder Pedersen, with r-reuse policy enforcement.

use crate::errors::PrivacyError;
use crate::pedersen::{BlindingTracker, PedersenCommit, PedersenCtx};
use zkprov_corelib::crypto::registry::hash32_by_id;

/// combine blinds deterministically: r12 = H(hash_id, "PEDERSEN.ADD", r1||r2)
fn combine_blinds(hash_id: &str, r1: &[u8], r2: &[u8]) -> Result<Vec<u8>, PrivacyError> {
    let mut buf = Vec::with_capacity(r1.len() + r2.len());
    buf.extend_from_slice(r1);
    buf.extend_from_slice(r2);
    let d = hash32_by_id(hash_id, "PEDERSEN.ADD", &buf)
        .ok_or_else(|| PrivacyError::Internal("hash id not supported".into()))?;
    Ok(d.to_vec())
}

pub struct AddUnderCommit;

impl AddUnderCommit {
    /// Compute Csum for m1+m2 with derived r12. Enforces no_r_reuse using tracker:
    /// - If policy disallows reuse, passing r1 == r2 will still derive a new r12,
    ///   but the tracker will now contain both r1 and r2; if the same r is attempted
    ///   again, it triggers BlindingReuse.
    pub fn run(
        ctx: &PedersenCtx,
        tracker: &mut BlindingTracker,
        m1: &[u8],
        r1: &[u8],
        m2: &[u8],
        r2: &[u8],
    ) -> Result<(PedersenCommit, Vec<u8>), PrivacyError> {
        // Enforce reuse policy on inputs (both must be "fresh" if policy forbids reuse)
        tracker.note_and_check(r1, ctx.no_reuse())?;
        tracker.note_and_check(r2, ctx.no_reuse())?;

        let r12 = combine_blinds(ctx.hash_id(), r1, r2)?;
        // For "open" semantics, compute msg = m1||"+"||m2 as placeholder (caller may choose canonical u64)
        let mut msg_sum = Vec::with_capacity(m1.len() + 1 + m2.len());
        msg_sum.extend_from_slice(m1);
        msg_sum.push(b'+');
        msg_sum.extend_from_slice(m2);

        let csum = ctx.commit(tracker, &msg_sum, &r12)?;
        Ok((csum, r12))
    }
}
