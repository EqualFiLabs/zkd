use zkprov_corelib::air::AirProgram;
use zkprov_corelib::registry::ensure_builtins_registered;
use zkprov_corelib::validate::validate_air_against_backend;

#[test]
fn pedersen_required_passes_on_native() {
    ensure_builtins_registered();
    let toml = r#"
        [meta]
        name = "needs_pedersen"
        field = "Prime254"
        hash = "blake3"
        [columns]
        trace_cols = 2
        [constraints]
        transition_count = 1
        boundary_count = 1
        [commitments]
        pedersen = true
        curve = "placeholder"
    "#;
    let air: AirProgram = toml::from_str(toml).unwrap();
    air.validate().unwrap();
    validate_air_against_backend(&air, "native@0.0").unwrap();
}

#[test]
fn pedersen_required_fails_on_unknown_curve() {
    ensure_builtins_registered();
    let toml = r#"
        [meta]
        name = "bad_curve"
        field = "Prime254"
        hash = "blake3"
        [columns]
        trace_cols = 2
        [constraints]
        transition_count = 1
        boundary_count = 1
        [commitments]
        pedersen = true
        curve = "bn254"
    "#;
    let air: AirProgram = toml::from_str(toml).unwrap();
    air.validate().unwrap();
    assert!(validate_air_against_backend(&air, "native@0.0").is_err());
}
