#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    Ok = 0,
    InvalidArg = 1,
    Backend = 2,
    Profile = 3,
    ProofCorrupt = 4,
    VerifyFail = 5,
    Internal = 6,
}

impl ErrorCode {
    #[inline]
    pub const fn code(self) -> i32 {
        self as i32
    }
}

impl From<ErrorCode> for i32 {
    fn from(code: ErrorCode) -> Self {
        code.code()
    }
}

pub const ZKP_OK: i32 = ErrorCode::Ok.code();
pub const ZKP_ERR_INVALID_ARG: i32 = ErrorCode::InvalidArg.code();
pub const ZKP_ERR_BACKEND: i32 = ErrorCode::Backend.code();
pub const ZKP_ERR_PROFILE: i32 = ErrorCode::Profile.code();
pub const ZKP_ERR_PROOF_CORRUPT: i32 = ErrorCode::ProofCorrupt.code();
pub const ZKP_ERR_VERIFY_FAIL: i32 = ErrorCode::VerifyFail.code();
pub const ZKP_ERR_INTERNAL: i32 = ErrorCode::Internal.code();
