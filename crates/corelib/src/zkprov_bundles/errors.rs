use serde::{Deserialize, Serialize};

/// DoD-specified error taxonomy for privacy gadget bundles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyError {
    InvalidCurvePoint,
    BlindingReuse,
    RangeCheckOverflow,
    UnsupportedCurve, // helpful internal; not required by DoD but used in messages
    Internal(String),
}

impl std::fmt::Display for PrivacyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PrivacyError::*;
        match self {
            InvalidCurvePoint => write!(f, "InvalidCurvePoint"),
            BlindingReuse => write!(f, "BlindingReuse"),
            RangeCheckOverflow => write!(f, "RangeCheckOverflow"),
            UnsupportedCurve => write!(f, "UnsupportedCurve"),
            Internal(s) => write!(f, "Internal({})", s),
        }
    }
}

impl std::error::Error for PrivacyError {}
