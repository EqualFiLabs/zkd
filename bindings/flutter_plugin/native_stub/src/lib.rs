#![no_std]
#![allow(improper_ctypes_definitions)]

use core::ffi::{c_char, c_int, c_uchar, c_uint, c_ulonglong, c_void};

const ZKP_OK: c_int = 0;
const ZKP_ERR_INVALID_ARG: c_int = 1;
const ZKP_ERR_VERIFY_FAIL: c_int = 5;

const BACKENDS_JSON: &[u8] =
    b"{\"items\":[{\"id\":\"native@0.0\",\"name\":\"Native Toy Backend\",\"protocol\":\"toy\"}]}\0";
const PROFILES_JSON: &[u8] =
    b"{\"items\":[{\"id\":\"balanced\",\"name\":\"Balanced Toy Profile\"}]}\0";
const PROVE_META_JSON: &[u8] = b"{\"digest\":\"DDEMO0001\",\"proof_len\":16}\0";
const VERIFY_OK_META_JSON: &[u8] = b"{\"digest\":\"DDEMO0001\",\"verified\":true}\0";
const VERIFY_FAIL_META_JSON: &[u8] = b"{\"digest\":\"DDEMO0001\",\"verified\":false}\0";
const PROOF_BYTES: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

#[no_mangle]
pub extern "C" fn zkp_init() -> c_int {
    ZKP_OK
}

#[no_mangle]
pub extern "C" fn zkp_list_backends(out_json: *mut *mut c_char) -> c_int {
    if out_json.is_null() {
        return ZKP_ERR_INVALID_ARG;
    }
    unsafe {
        *out_json = BACKENDS_JSON.as_ptr() as *mut c_char;
    }
    ZKP_OK
}

#[no_mangle]
pub extern "C" fn zkp_list_profiles(out_json: *mut *mut c_char) -> c_int {
    if out_json.is_null() {
        return ZKP_ERR_INVALID_ARG;
    }
    unsafe {
        *out_json = PROFILES_JSON.as_ptr() as *mut c_char;
    }
    ZKP_OK
}

#[no_mangle]
pub extern "C" fn zkp_prove(
    _backend_id: *const c_char,
    _field: *const c_char,
    _hash_id: *const c_char,
    _fri_arity: c_uint,
    _profile_id: *const c_char,
    _air_path: *const c_char,
    _public_inputs_json: *const c_char,
    out_proof: *mut *mut c_uchar,
    out_proof_len: *mut c_ulonglong,
    out_json_meta: *mut *mut c_char,
) -> c_int {
    unsafe {
        if !out_proof.is_null() {
            *out_proof = PROOF_BYTES.as_ptr() as *mut c_uchar;
        }
        if !out_proof_len.is_null() {
            *out_proof_len = PROOF_BYTES.len() as c_ulonglong;
        }
        if !out_json_meta.is_null() {
            *out_json_meta = PROVE_META_JSON.as_ptr() as *mut c_char;
        }
    }
    ZKP_OK
}

fn proof_matches(ptr: *const c_uchar, len: c_ulonglong) -> bool {
    if ptr.is_null() {
        return false;
    }
    if len as usize != PROOF_BYTES.len() {
        return false;
    }
    let slice = unsafe { core::slice::from_raw_parts(ptr, len as usize) };
    slice == PROOF_BYTES
}

#[no_mangle]
pub extern "C" fn zkp_verify(
    _backend_id: *const c_char,
    _field: *const c_char,
    _hash_id: *const c_char,
    _fri_arity: c_uint,
    _profile_id: *const c_char,
    _air_path: *const c_char,
    _public_inputs_json: *const c_char,
    proof_ptr: *const c_uchar,
    proof_len: c_ulonglong,
    out_json_meta: *mut *mut c_char,
) -> c_int {
    unsafe {
        if !out_json_meta.is_null() {
            *out_json_meta = VERIFY_OK_META_JSON.as_ptr() as *mut c_char;
        }
    }
    if proof_matches(proof_ptr, proof_len) {
        ZKP_OK
    } else {
        unsafe {
            if !out_json_meta.is_null() {
                *out_json_meta = VERIFY_FAIL_META_JSON.as_ptr() as *mut c_char;
            }
        }
        ZKP_ERR_VERIFY_FAIL
    }
}

#[no_mangle]
pub extern "C" fn zkp_alloc(_nbytes: c_ulonglong) -> *mut c_void {
    core::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn zkp_free(_ptr: *mut c_void) {}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
