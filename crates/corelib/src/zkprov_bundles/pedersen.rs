//! PedersenCommit(Cx, Cy) with curve/no-reuse policy and DoD errors.
//! Backed by corelib's PedersenPlaceholder. For placeholder, we synthesize (Cx,Cy)
//! as two domain-separated 32-byte digests, then expose them as a pair.

use super::errors::PrivacyError;
use crate::air::bindings::Bindings;
use crate::crypto::registry::hash32_by_id;
use crate::gadgets::commitment::{
    Comm32, CommitmentScheme32, PedersenParams, PedersenPlaceholder, Witness,
};
use std::collections::HashSet;

/// Tracks used blindings in a session to enforce no-reuse when policy says so.
#[derive(Debug, Default)]
pub struct BlindingTracker {
    used: HashSet<Vec<u8>>,
}
impl BlindingTracker {
    pub fn new() -> Self {
        Self {
            used: HashSet::new(),
        }
    }
    pub fn note_and_check(&mut self, r: &[u8], no_reuse: bool) -> Result<(), PrivacyError> {
        if !no_reuse {
            return Ok(());
        }
        let key = r.to_vec();
        if self.used.contains(&key) {
            return Err(PrivacyError::BlindingReuse);
        }
        self.used.insert(key);
        Ok(())
    }
}

/// Context: curve + hash selection resolved from AIR bindings.
pub struct PedersenCtx {
    ped: PedersenPlaceholder,
    curve: String,
    no_r_reuse: bool,
}

impl PedersenCtx {
    pub fn from_bindings(b: &Bindings) -> Result<Self, PrivacyError> {
        // Validate curve compatibility (placeholder supports only "placeholder")
        let curve = b
            .commitments
            .curve
            .clone()
            .unwrap_or_else(|| "placeholder".to_string());
        if curve != "placeholder" {
            // Backend would have rejected earlier; we mirror DoD error taxonomy here:
            return Err(PrivacyError::UnsupportedCurve);
        }
        let hash_id = b
            .hash_id_for_commitments
            .clone()
            .unwrap_or_else(|| "blake3".to_string());
        Ok(Self {
            ped: PedersenPlaceholder::new(PedersenParams { hash_id }),
            curve,
            no_r_reuse: b.commitments.no_r_reuse.unwrap_or(false),
        })
    }
}

/// Return type: PedersenCommit(Cx,Cy).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PedersenCommit {
    pub cx: [u8; 32],
    pub cy: [u8; 32],
}

impl PedersenCommit {
    pub fn as_tuple(&self) -> (&[u8; 32], &[u8; 32]) {
        (&self.cx, &self.cy)
    }
}

/// Compute placeholder "affine" (Cx,Cy) from a 32-byte commitment by hashing
/// with two different labels. This stands in for real EC map-to-point.
fn expand_to_point(hash_id: &str, base: &Comm32) -> Result<([u8; 32], [u8; 32]), PrivacyError> {
    let cx = hash32_by_id(hash_id, "PEDERSEN.CX", base.as_bytes())
        .ok_or_else(|| PrivacyError::Internal("hash id not supported".into()))?;
    let cy = hash32_by_id(hash_id, "PEDERSEN.CY", base.as_bytes())
        .ok_or_else(|| PrivacyError::Internal("hash id not supported".into()))?;
    Ok((cx, cy))
}

/// Validate "curve point". Placeholder always accepts, but if Bindings specifies
/// a non-placeholder curve the ctx creation already rejected; reaching here means OK.
fn validate_point_ok(_curve: &str, _cx: &[u8; 32], _cy: &[u8; 32]) -> Result<(), PrivacyError> {
    Ok(())
}

impl PedersenCtx {
    pub fn commit(
        &self,
        tracker: &mut BlindingTracker,
        msg: &[u8],
        blind: &[u8],
    ) -> Result<PedersenCommit, PrivacyError> {
        tracker.note_and_check(blind, self.no_r_reuse)?;
        let commitment = self
            .ped
            .commit(&Witness { msg, blind })
            .map_err(|e| PrivacyError::Internal(e.to_string()))?;
        let (cx, cy) = expand_to_point(self.ped.hash_id(), &commitment)?;
        validate_point_ok(&self.curve, &cx, &cy)?;
        Ok(PedersenCommit { cx, cy })
    }

    pub fn open(
        &self,
        msg: &[u8],
        blind: &[u8],
        cx: &[u8; 32],
        cy: &[u8; 32],
    ) -> Result<bool, PrivacyError> {
        let commitment = self
            .ped
            .commit(&Witness { msg, blind })
            .map_err(|e| PrivacyError::Internal(e.to_string()))?;
        let (exp_cx, exp_cy) = expand_to_point(self.ped.hash_id(), &commitment)?;
        // If a real curve, this would also check on-curve. Map failure to InvalidCurvePoint.
        if cx != &exp_cx || cy != &exp_cy {
            return Err(PrivacyError::InvalidCurvePoint);
        }
        Ok(true)
    }

    pub fn hash_id(&self) -> &str {
        self.ped.hash_id()
    }
    pub fn no_reuse(&self) -> bool {
        self.no_r_reuse
    }
}
