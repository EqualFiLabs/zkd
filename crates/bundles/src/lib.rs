//! Privacy Gadget Bundles (v1): Pedersen, Range, Arithmetic-under-Commitments.
//! Thin layer that enforces policy & DoD-specific errors on top of corelib gadgets/crypto.

pub mod arith;
pub mod errors;
pub mod pedersen;
pub mod range;

pub use arith::AddUnderCommit;
pub use errors::PrivacyError;
pub use pedersen::{BlindingTracker, PedersenCommit, PedersenCtx};
pub use range::RangeCheck;
