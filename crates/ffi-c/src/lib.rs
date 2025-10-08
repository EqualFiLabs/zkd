mod error;
mod ffi_json;

pub use error::{
    ErrorCode, ZKP_ERR_BACKEND, ZKP_ERR_INTERNAL, ZKP_ERR_INVALID_ARG, ZKP_ERR_PROFILE,
    ZKP_ERR_PROOF_CORRUPT, ZKP_ERR_VERIFY_FAIL, ZKP_OK,
};
pub use ffi_json::{err, ok, with_field, Envelope};

#[no_mangle]
pub extern "C" fn zkp_init() {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::ffi::CString;

    fn parse_cstring(cstr: CString) -> Value {
        let json = cstr
            .into_string()
            .expect("ffi_json must emit UTF-8 strings");
        serde_json::from_str(&json).expect("ffi_json must emit valid JSON")
    }

    #[test]
    fn ok_envelope_uses_success_code() {
        let cstr = ok().into_cstring();
        let value = parse_cstring(cstr);
        assert_eq!(value["code"], Value::from(ZKP_OK));
        assert!(value["ok"].as_bool().unwrap());
        assert_eq!(value["msg"], Value::from("OK"));
    }

    #[test]
    fn proof_corrupt_error_has_correct_code() {
        let cstr = err(ErrorCode::ProofCorrupt, "proof bytes truncated").into_cstring();
        let value = parse_cstring(cstr);
        assert_eq!(value["code"], Value::from(ZKP_ERR_PROOF_CORRUPT));
        assert!(!value["ok"].as_bool().unwrap());
    }

    #[test]
    fn envelopes_are_proper_c_strings() {
        let cstr = ok().into_cstring();
        let bytes_with_nul = cstr.as_bytes_with_nul();
        assert_eq!(bytes_with_nul.last().copied(), Some(0));
        let without_nul = &bytes_with_nul[..bytes_with_nul.len() - 1];
        assert!(std::str::from_utf8(without_nul).is_ok());
    }
}
