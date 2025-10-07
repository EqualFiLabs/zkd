//! Range-check utilities (Phase-0).
//! - k-bit checks for u64 values
//! - batch helpers

use anyhow::{anyhow, Result};

/// Ensure `x` fits within `k` bits (1..=64). Returns Ok(()) or error with message.
pub fn range_check_u64(x: u64, k: u32) -> Result<()> {
    if !(1..=64).contains(&k) {
        return Err(anyhow!("range_check: k={} out of bounds [1..=64]", k));
    }
    let mask_ok = if k == 64 { u64::MAX } else { (1u64 << k) - 1 };
    if x & !mask_ok != 0 {
        return Err(anyhow!(
            "range_check: value {} does not fit in {} bits",
            x,
            k
        ));
    }
    Ok(())
}

/// Batch variant: every element must satisfy the same bound.
pub fn range_check_slice_u64(xs: &[u64], k: u32) -> Result<()> {
    for &x in xs {
        range_check_u64(x, k)?;
    }
    Ok(())
}
