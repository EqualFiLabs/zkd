use zkprov_corelib::registry::{ensure_builtins_registered, list_backend_infos};

#[test]
fn native_recursion_flag_false() {
    ensure_builtins_registered();
    let infos = list_backend_infos();
    let native = infos.iter().find(|b| b.id == "native@0.0").unwrap();
    assert_eq!(native.recursion, false);
}
