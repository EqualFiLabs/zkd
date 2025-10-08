use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::ptr;
use std::slice;
use std::sync::{Mutex, OnceLock};

use anyhow::Error as AnyhowError;
use serde::Serialize;
use zkprov_backend_native::{native_prove, native_verify};
use zkprov_corelib::backend::BackendInfo;
use zkprov_corelib::config::Config;
use zkprov_corelib::errors::{CapabilityError, RegistryError};
use zkprov_corelib::evm::digest::digest_D;
use zkprov_corelib::profile::load_all_profiles;
use zkprov_corelib::proof::ProofHeader;
use zkprov_corelib::{registry, validate::validate_config};

mod error;
mod ffi_json;

pub use error::{
    ErrorCode, ZKP_ERR_BACKEND, ZKP_ERR_INTERNAL, ZKP_ERR_INVALID_ARG, ZKP_ERR_PROFILE,
    ZKP_ERR_PROOF_CORRUPT, ZKP_ERR_VERIFY_FAIL, ZKP_OK,
};
pub use ffi_json::{err, ok, with_field, Envelope};

#[derive(Debug, Clone, Copy)]
struct Allocation {
    len: usize,
    cap: usize,
}

type FfiResult<T> = Result<T, ErrorCode>;

static ALLOCATIONS: OnceLock<Mutex<HashMap<usize, Allocation>>> = OnceLock::new();
static INIT_RESULT: OnceLock<Result<(), ErrorCode>> = OnceLock::new();

fn allocations() -> &'static Mutex<HashMap<usize, Allocation>> {
    ALLOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn store_allocation(ptr: *mut u8, len: usize, cap: usize) -> FfiResult<()> {
    let mut guard = allocations().lock().map_err(|_| ErrorCode::Internal)?;
    guard.insert(ptr as usize, Allocation { len, cap });
    Ok(())
}

fn take_allocation(ptr: *mut u8) -> Option<Allocation> {
    allocations()
        .lock()
        .ok()
        .and_then(|mut guard| guard.remove(&(ptr as usize)))
}

fn release_allocation(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    if let Some(alloc) = take_allocation(ptr) {
        unsafe {
            Vec::from_raw_parts(ptr, alloc.len, alloc.cap);
        }
    }
}

fn alloc_bytes(len: usize) -> FfiResult<*mut u8> {
    if len == 0 {
        return Ok(ptr::null_mut());
    }
    let mut buf = vec![0u8; len];
    let ptr = buf.as_mut_ptr();
    let cap = buf.capacity();
    store_allocation(ptr, len, cap)?;
    std::mem::forget(buf);
    Ok(ptr)
}

fn leak_vec(mut vec: Vec<u8>) -> FfiResult<*mut u8> {
    if vec.is_empty() {
        return Ok(ptr::null_mut());
    }
    let len = vec.len();
    let cap = vec.capacity();
    let ptr = vec.as_mut_ptr();
    store_allocation(ptr, len, cap)?;
    std::mem::forget(vec);
    Ok(ptr)
}

fn alloc_cstring(s: &str) -> FfiResult<*mut c_char> {
    let bytes = s.as_bytes();
    let len = bytes.len().checked_add(1).ok_or(ErrorCode::Internal)?;
    let ptr = alloc_bytes(len)?;
    if ptr.is_null() {
        return Ok(ptr::null_mut());
    }
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        *ptr.add(bytes.len()) = 0;
    }
    Ok(ptr as *mut c_char)
}

fn init_runtime() -> FfiResult<()> {
    let result = INIT_RESULT.get_or_init(|| {
        if let Err(err) = zkprov_backend_native::register_native_backend() {
            if !matches!(err, RegistryError::DuplicateBackend(_)) {
                return Err(map_registry_error(&err));
            }
        }
        registry::ensure_builtins_registered();
        Ok(())
    });
    *result
}

fn read_cstring(ptr: *const c_char) -> FfiResult<String> {
    if ptr.is_null() {
        return Err(ErrorCode::InvalidArg);
    }
    unsafe {
        let cstr = CStr::from_ptr(ptr);
        if cstr.to_bytes().is_empty() {
            return Err(ErrorCode::InvalidArg);
        }
        cstr.to_str()
            .map(|s| s.to_owned())
            .map_err(|_| ErrorCode::InvalidArg)
    }
}

fn ensure_output_ptr<T>(out: *mut *mut T) -> FfiResult<()> {
    if out.is_null() {
        return Err(ErrorCode::InvalidArg);
    }
    unsafe {
        *out = ptr::null_mut();
    }
    Ok(())
}

fn ensure_output_scalar<T: Default>(out: *mut T) -> FfiResult<()> {
    if out.is_null() {
        return Err(ErrorCode::InvalidArg);
    }
    unsafe {
        *out = T::default();
    }
    Ok(())
}

fn map_capability_error(err: &CapabilityError) -> ErrorCode {
    match err {
        CapabilityError::ProfileNotFound(_) => ErrorCode::Profile,
        CapabilityError::Mismatch(_) => ErrorCode::Backend,
        CapabilityError::FieldUnsupported { .. }
        | CapabilityError::HashUnsupported { .. }
        | CapabilityError::FriArityUnsupported { .. }
        | CapabilityError::RecursionUnavailable { .. } => ErrorCode::Backend,
    }
}

fn map_registry_error(err: &RegistryError) -> ErrorCode {
    match err {
        RegistryError::DuplicateBackend(_) => ErrorCode::Internal,
        RegistryError::BackendNotFound(_) => ErrorCode::Backend,
    }
}

fn map_prove_error(err: &AnyhowError) -> ErrorCode {
    if let Some(cap) = err.downcast_ref::<CapabilityError>() {
        return map_capability_error(cap);
    }
    if let Some(reg) = err.downcast_ref::<RegistryError>() {
        return map_registry_error(reg);
    }
    ErrorCode::Internal
}

fn is_proof_corrupt_message(msg: &str) -> bool {
    const CORRUPT_MARKERS: &[&str] = &[
        "too short",
        "bad magic",
        "unsupported proof version",
        "body length mismatch",
    ];
    CORRUPT_MARKERS.iter().any(|needle| msg.contains(needle))
}

fn map_verify_error(err: &AnyhowError) -> ErrorCode {
    if let Some(cap) = err.downcast_ref::<CapabilityError>() {
        return map_capability_error(cap);
    }
    if let Some(reg) = err.downcast_ref::<RegistryError>() {
        return map_registry_error(reg);
    }
    let msg = err.to_string();
    if is_proof_corrupt_message(&msg) {
        ErrorCode::ProofCorrupt
    } else {
        ErrorCode::VerifyFail
    }
}

fn to_i32(result: FfiResult<()>) -> i32 {
    match result {
        Ok(()) => ZKP_OK,
        Err(code) => code.into(),
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2 + 2);
    out.push_str("0x");
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

fn serialize_json<T: Serialize>(value: &T) -> FfiResult<String> {
    serde_json::to_string(value).map_err(|_| ErrorCode::Internal)
}

#[no_mangle]
pub extern "C" fn zkp_init() -> i32 {
    to_i32(init_runtime())
}

/// # Safety
///
/// - `out_json` must point to valid, writable memory where a pointer to a newly
///   allocated, null-terminated string can be stored.
/// - The caller is responsible for freeing the returned string with
///   [`zkp_free_string`](crate::zkp_free_string).
#[no_mangle]
pub unsafe extern "C" fn zkp_list_backends(out_json: *mut *mut c_char) -> i32 {
    to_i32((|| {
        ensure_output_ptr(out_json)?;
        init_runtime()?;
        let infos: Vec<BackendInfo> = registry::list_backend_infos();
        let json = serialize_json(&infos)?;
        let ptr = alloc_cstring(&json)?;
        unsafe {
            *out_json = ptr;
        }
        Ok(())
    })())
}

/// # Safety
///
/// - `out_json` must point to valid, writable memory where a pointer to a newly
///   allocated, null-terminated string can be stored.
/// - The caller is responsible for freeing the returned string with
///   [`zkp_free_string`](crate::zkp_free_string).
#[no_mangle]
pub unsafe extern "C" fn zkp_list_profiles(out_json: *mut *mut c_char) -> i32 {
    to_i32((|| {
        ensure_output_ptr(out_json)?;
        init_runtime()?;
        let profiles = load_all_profiles().map_err(|_| ErrorCode::Internal)?;
        let json = serialize_json(&profiles)?;
        let ptr = alloc_cstring(&json)?;
        unsafe {
            *out_json = ptr;
        }
        Ok(())
    })())
}

/// # Safety
///
/// - All pointer arguments must be valid for reads of a null-terminated string
///   (for `*_id`, `air_path`, and `public_inputs_json`).
/// - `out_proof`, `out_proof_len`, and `out_json_meta` must be valid, writable
///   pointers where this function can store ownership of newly allocated
///   buffers.
/// - The caller is responsible for eventually releasing any allocations via the
///   corresponding `zkp_free_*` helpers.
#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub unsafe extern "C" fn zkp_prove(
    backend_id: *const c_char,
    field: *const c_char,
    hash_id: *const c_char,
    fri_arity: u32,
    profile_id: *const c_char,
    air_path: *const c_char,
    public_inputs_json: *const c_char,
    out_proof: *mut *mut u8,
    out_proof_len: *mut u64,
    out_json_meta: *mut *mut c_char,
) -> i32 {
    to_i32((|| {
        ensure_output_ptr(out_proof)?;
        ensure_output_scalar(out_proof_len)?;
        ensure_output_ptr(out_json_meta)?;
        init_runtime()?;

        let backend = read_cstring(backend_id)?;
        let field = read_cstring(field)?;
        let hash = read_cstring(hash_id)?;
        let profile = read_cstring(profile_id)?;
        let air = read_cstring(air_path)?;
        let pub_inputs = read_cstring(public_inputs_json)?;

        let config = Config::new(backend, field, hash, fri_arity, false, profile);
        validate_config(&config).map_err(|e| map_capability_error(&e))?;

        let proof = native_prove(&config, &pub_inputs, &air).map_err(|e| map_prove_error(&e))?;
        let proof_len = proof.len();
        let proof_len_u64 = u64::try_from(proof_len).map_err(|_| ErrorCode::Internal)?;
        if proof_len < 40 {
            return Err(ErrorCode::Internal);
        }
        let header = ProofHeader::decode(&proof[0..40]).map_err(|_| ErrorCode::Internal)?;
        let body = &proof[40..];
        let digest = digest_D(&header, body);
        let digest_hex = hex_encode(&digest);

        let meta_envelope = with_field(
            with_field(ok(), "digest", digest_hex),
            "proof_len",
            proof_len_u64,
        );
        let meta_json = meta_envelope.into_string();
        let meta_ptr = alloc_cstring(&meta_json)?;

        let proof_ptr = leak_vec(proof).inspect_err(|_| {
            release_allocation(meta_ptr as *mut u8);
        })?;

        unsafe {
            *out_proof = proof_ptr;
            *out_proof_len = proof_len_u64;
            *out_json_meta = meta_ptr;
        }
        Ok(())
    })())
}

/// # Safety
///
/// - All pointer arguments must be valid for reads of a null-terminated string
///   (for `*_id`, `air_path`, and `public_inputs_json`).
/// - When `proof_len` is non-zero, `proof_ptr` must reference a buffer of at
///   least `proof_len` bytes.
/// - `out_json_meta` must be a valid, writable pointer where this function can
///   store ownership of a newly allocated string. The caller is responsible for
///   freeing it with [`zkp_free_string`](crate::zkp_free_string).
#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub unsafe extern "C" fn zkp_verify(
    backend_id: *const c_char,
    field: *const c_char,
    hash_id: *const c_char,
    fri_arity: u32,
    profile_id: *const c_char,
    air_path: *const c_char,
    public_inputs_json: *const c_char,
    proof_ptr: *const u8,
    proof_len: u64,
    out_json_meta: *mut *mut c_char,
) -> i32 {
    to_i32((|| {
        ensure_output_ptr(out_json_meta)?;
        init_runtime()?;

        let backend = read_cstring(backend_id)?;
        let field = read_cstring(field)?;
        let hash = read_cstring(hash_id)?;
        let profile = read_cstring(profile_id)?;
        let air = read_cstring(air_path)?;
        let pub_inputs = read_cstring(public_inputs_json)?;

        let proof_len_usize = usize::try_from(proof_len).map_err(|_| ErrorCode::InvalidArg)?;
        if proof_len_usize == 0 {
            return Err(ErrorCode::ProofCorrupt);
        }
        if proof_ptr.is_null() {
            return Err(ErrorCode::InvalidArg);
        }
        let proof = unsafe { slice::from_raw_parts(proof_ptr, proof_len_usize) };

        if proof.len() < 40 {
            return Err(ErrorCode::ProofCorrupt);
        }
        let header = ProofHeader::decode(&proof[0..40]).map_err(|_| ErrorCode::ProofCorrupt)?;
        let body = &proof[40..];
        if u64::try_from(body.len()).map_err(|_| ErrorCode::Internal)? != header.body_len {
            return Err(ErrorCode::ProofCorrupt);
        }
        let digest = digest_D(&header, body);
        let digest_hex = hex_encode(&digest);

        let config = Config::new(backend, field, hash, fri_arity, false, profile);
        validate_config(&config).map_err(|e| map_capability_error(&e))?;

        match native_verify(&config, &pub_inputs, &air, proof) {
            Ok(true) => {}
            Ok(false) => return Err(ErrorCode::VerifyFail),
            Err(err) => return Err(map_verify_error(&err)),
        }

        let meta_envelope = with_field(with_field(ok(), "verified", true), "digest", digest_hex);
        let meta_json = meta_envelope.into_string();
        let meta_ptr = alloc_cstring(&meta_json)?;
        unsafe {
            *out_json_meta = meta_ptr;
        }
        Ok(())
    })())
}

#[no_mangle]
pub extern "C" fn zkp_alloc(nbytes: u64) -> *mut c_void {
    match usize::try_from(nbytes) {
        Ok(len) => match alloc_bytes(len) {
            Ok(ptr) => ptr.cast(),
            Err(_) => ptr::null_mut(),
        },
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn zkp_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    release_allocation(ptr as *mut u8);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::ffi::{CStr, CString};
    use std::path::PathBuf;
    use std::ptr;

    fn parse_cstring(cstr: CString) -> Value {
        let json = cstr
            .into_string()
            .expect("ffi_json must emit UTF-8 strings");
        serde_json::from_str(&json).expect("ffi_json must emit valid JSON")
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
    }

    fn toy_air_path() -> CString {
        let path = workspace_root()
            .join("examples")
            .join("air")
            .join("toy.air");
        CString::new(path.to_str().expect("toy.air path must be UTF-8")).unwrap()
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

    #[test]
    fn prove_and_verify_roundtrip_via_ffi() {
        assert_eq!(zkp_init(), ZKP_OK);

        let mut backends_ptr: *mut c_char = ptr::null_mut();
        assert_eq!(unsafe { zkp_list_backends(&mut backends_ptr) }, ZKP_OK);
        assert!(!backends_ptr.is_null());
        let backends_json = unsafe { CStr::from_ptr(backends_ptr) }
            .to_str()
            .expect("backends JSON must be UTF-8");
        let backends: Value = serde_json::from_str(backends_json).unwrap();
        assert!(backends
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b["id"] == "native@0.0"));
        zkp_free(backends_ptr.cast());

        let mut profiles_ptr: *mut c_char = ptr::null_mut();
        assert_eq!(unsafe { zkp_list_profiles(&mut profiles_ptr) }, ZKP_OK);
        assert!(!profiles_ptr.is_null());
        let profiles_json = unsafe { CStr::from_ptr(profiles_ptr) }
            .to_str()
            .expect("profiles JSON must be UTF-8");
        let profiles: Value = serde_json::from_str(profiles_json).unwrap();
        assert!(profiles
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == "balanced"));
        zkp_free(profiles_ptr.cast());

        let backend = CString::new("native@0.0").unwrap();
        let field = CString::new("Prime254").unwrap();
        let hash = CString::new("blake3").unwrap();
        let profile = CString::new("balanced").unwrap();
        let air = toy_air_path();
        let inputs = CString::new("{\"a\":1,\"b\":[2,3]}").unwrap();

        let mut proof_ptr: *mut u8 = ptr::null_mut();
        let mut proof_len: u64 = 0;
        let mut prove_meta_ptr: *mut c_char = ptr::null_mut();
        let status = unsafe {
            zkp_prove(
                backend.as_ptr(),
                field.as_ptr(),
                hash.as_ptr(),
                2,
                profile.as_ptr(),
                air.as_ptr(),
                inputs.as_ptr(),
                &mut proof_ptr,
                &mut proof_len,
                &mut prove_meta_ptr,
            )
        };
        assert_eq!(status, ZKP_OK);
        assert!(proof_len >= 40);
        assert!(!proof_ptr.is_null());
        assert!(!prove_meta_ptr.is_null());

        let prove_meta = unsafe { CStr::from_ptr(prove_meta_ptr) }
            .to_str()
            .expect("meta must be UTF-8");
        let prove_meta_json: Value = serde_json::from_str(prove_meta).unwrap();
        assert!(prove_meta_json["ok"].as_bool().unwrap());
        assert!(prove_meta_json.get("digest").is_some());
        assert_eq!(prove_meta_json["proof_len"], Value::from(proof_len));

        let mut verify_meta_ptr: *mut c_char = ptr::null_mut();
        let status = unsafe {
            zkp_verify(
                backend.as_ptr(),
                field.as_ptr(),
                hash.as_ptr(),
                2,
                profile.as_ptr(),
                air.as_ptr(),
                inputs.as_ptr(),
                proof_ptr as *const u8,
                proof_len,
                &mut verify_meta_ptr,
            )
        };
        assert_eq!(status, ZKP_OK);
        assert!(!verify_meta_ptr.is_null());
        let verify_meta = unsafe { CStr::from_ptr(verify_meta_ptr) }
            .to_str()
            .expect("verify meta must be UTF-8");
        let verify_meta_json: Value = serde_json::from_str(verify_meta).unwrap();
        assert!(verify_meta_json["ok"].as_bool().unwrap());
        assert!(verify_meta_json["verified"].as_bool().unwrap());
        assert_eq!(verify_meta_json["digest"], prove_meta_json["digest"]);

        zkp_free(prove_meta_ptr.cast());
        zkp_free(verify_meta_ptr.cast());
        zkp_free(proof_ptr.cast());
    }
}
