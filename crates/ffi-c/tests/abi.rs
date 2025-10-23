use libloading::Library;
use serde_json::Value;
use std::env;
use std::ffi::{c_char, c_void, CStr};
use std::path::{Path, PathBuf};

type InitFn = unsafe extern "C" fn() -> i32;
type ListFn = unsafe extern "C" fn(*mut *mut c_char) -> i32;
type ProveFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *const c_char,
    u32,
    *const c_char,
    *const c_char,
    *const c_char,
    *mut *mut u8,
    *mut u64,
    *mut *mut c_char,
) -> i32;
type VerifyFn = unsafe extern "C" fn(
    *const c_char,
    *const c_char,
    *const c_char,
    u32,
    *const c_char,
    *const c_char,
    *const c_char,
    *const u8,
    u64,
    *mut *mut c_char,
) -> i32;
type AllocFn = unsafe extern "C" fn(u64) -> *mut c_void;
type FreeFn = unsafe extern "C" fn(*mut c_void);

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("workspace root must resolve")
}

fn library_names() -> &'static [&'static str] {
    if cfg!(target_os = "windows") {
        &["zkprov.dll"]
    } else if cfg!(target_os = "macos") {
        &["libzkprov.dylib"]
    } else {
        &["libzkprov.so"]
    }
}

fn find_library() -> PathBuf {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".into());
    let root = workspace_root();
    let candidates = [
        root.join("target").join(&profile),
        root.join("target").join(&profile).join("deps"),
    ];

    for dir in &candidates {
        for name in library_names() {
            let candidate = dir.join(name);
            if candidate.exists() {
                return candidate;
            }
        }
    }

    panic!(
        "unable to locate libzkprov.{:?} under {:?}",
        library_names(),
        candidates
    );
}

fn load_library() -> Library {
    let path = find_library();
    unsafe { Library::new(path) }.expect("failed to load libzkprov")
}

#[test]
fn exports_expected_symbols() {
    let lib = load_library();
    unsafe {
        lib.get::<InitFn>(b"zkp_init\0").expect("zkp_init missing");
        lib.get::<ListFn>(b"zkp_list_backends\0")
            .expect("zkp_list_backends missing");
        lib.get::<ListFn>(b"zkp_list_profiles\0")
            .expect("zkp_list_profiles missing");
        lib.get::<ListFn>(b"zkp_version\0")
            .expect("zkp_version missing");
        lib.get::<ProveFn>(b"zkp_prove\0")
            .expect("zkp_prove missing");
        lib.get::<VerifyFn>(b"zkp_verify\0")
            .expect("zkp_verify missing");
        lib.get::<AllocFn>(b"zkp_alloc\0")
            .expect("zkp_alloc missing");
        lib.get::<FreeFn>(b"zkp_free\0").expect("zkp_free missing");
    }
}

#[test]
fn version_symbol_reports_semver() {
    let lib = load_library();
    unsafe {
        let version_fn: libloading::Symbol<unsafe extern "C" fn(*mut *mut c_char) -> i32> =
            lib.get(b"zkp_version\0").expect("zkp_version missing");
        let free_fn: libloading::Symbol<FreeFn> = lib.get(b"zkp_free\0").expect("zkp_free missing");

        let mut out_ptr: *mut c_char = std::ptr::null_mut();
        let status = version_fn(&mut out_ptr);
        assert_eq!(status, 0, "zkp_version must return ZKP_OK");
        assert!(
            !out_ptr.is_null(),
            "zkp_version must allocate a JSON string"
        );
        let json = CStr::from_ptr(out_ptr)
            .to_str()
            .expect("UTF-8 version JSON");
        let value: Value = serde_json::from_str(json).expect("valid JSON");
        assert_eq!(
            value["version"],
            Value::from(env!("CARGO_PKG_VERSION")),
            "version field must match crate semver"
        );
        free_fn(out_ptr.cast());
    }
}
