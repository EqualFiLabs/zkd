//! RangeCheck(v,k) emitting RangeCheckOverflow on violation.

use crate::errors::PrivacyError;

pub struct RangeCheck;

impl RangeCheck {
    pub fn check_u64(v: u64, k: u32) -> Result<(), PrivacyError> {
        if !(1..=64).contains(&k) {
            return Err(PrivacyError::RangeCheckOverflow);
        }
        let mask_ok = if k == 64 { u64::MAX } else { (1u64 << k) - 1 };
        if v & !mask_ok != 0 {
            return Err(PrivacyError::RangeCheckOverflow);
        }
        Ok(())
    }
}
