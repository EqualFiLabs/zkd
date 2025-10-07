//! C ABI scaffold. Safe to load via Dart FFI; does nothing substantial yet.

#[no_mangle]
pub extern "C" fn zkp_init() -> i32 {
    // Future: load backend registry, profiles, etc.
    0 // ZKP_OK
}

#[no_mangle]
pub extern "C" fn zkp_free(ptr: *mut core::ffi::c_void) {
    if !ptr.is_null() {
        unsafe { libc::free(ptr) };
    }
}
