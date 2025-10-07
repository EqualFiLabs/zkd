//! C ABI scaffold. Safe to load via Dart FFI; does nothing substantial yet.

#[no_mangle]
pub extern "C" fn zkp_init() -> i32 {
    // Future: load backend registry, profiles, etc.
    0 // ZKP_OK
}

/// # Safety
///
/// Caller must ensure `ptr` was allocated by a compatible allocator and has not
/// been freed already. Passing an invalid pointer results in undefined behavior.
#[no_mangle]
pub unsafe extern "C" fn zkp_free(ptr: *mut core::ffi::c_void) {
    if !ptr.is_null() {
        unsafe { libc::free(ptr) };
    }
}
