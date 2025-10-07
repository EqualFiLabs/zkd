use zkprov_corelib::config::Config;
use zkprov_corelib::registry::ensure_builtins_registered;
use zkprov_corelib::validate::validate_config;

#[test]
fn validate_ok_native_default() {
    ensure_builtins_registered();
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false, "balanced");
    assert!(validate_config(&cfg).is_ok());
}

#[test]
fn invalid_field() {
    ensure_builtins_registered();
    let cfg = Config::new("native@0.0", "Goldilocks", "blake3", 2, false, "balanced");
    let err = validate_config(&cfg).unwrap_err().to_string();
    assert!(err.contains("field 'Goldilocks'"));
}

#[test]
fn invalid_hash() {
    ensure_builtins_registered();
    let cfg = Config::new("native@0.0", "Prime254", "keccak", 2, false, "balanced");
    let err = validate_config(&cfg).unwrap_err().to_string();
    assert!(err.contains("hash 'keccak'"));
}

#[test]
fn invalid_arity() {
    ensure_builtins_registered();
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 8, false, "balanced");
    let err = validate_config(&cfg).unwrap_err().to_string();
    assert!(err.contains("FRI arity '8'"));
}

#[test]
fn recursion_unavailable() {
    ensure_builtins_registered();
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, true, "balanced");
    let err = validate_config(&cfg).unwrap_err().to_string();
    assert!(err.contains("recursion required"));
}

#[test]
fn profile_missing() {
    ensure_builtins_registered();
    let cfg = Config::new(
        "native@0.0",
        "Prime254",
        "blake3",
        2,
        false,
        "does-not-exist",
    );
    let err = validate_config(&cfg).unwrap_err().to_string();
    assert!(err.contains("profile 'does-not-exist'"));
}
