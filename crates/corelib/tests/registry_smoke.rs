use zkprov_corelib::registry::{ensure_builtins_registered, list_backend_infos};

#[test]
fn registry_lists_native_backend() {
    ensure_builtins_registered();
    let infos = list_backend_infos();
    assert!(infos.iter().any(|b| b.id.starts_with("native@")));
}
