use zkprov_backend_native::{native_prove, native_verify};
use zkprov_corelib::config::Config;

const AIR_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../examples/air/toy.air");

#[test]
fn blake3_and_keccak_yield_different_bodies() {
    let cfg_b3 = Config::new("native@0.0", "Prime254", "blake3", 2, false, "balanced");
    let cfg_kc = Config::new("native@0.0", "Prime254", "keccak256", 2, false, "balanced");
    let inputs = r#"{"x":1}"#;

    let proof_b3 = native_prove(&cfg_b3, inputs, AIR_PATH).unwrap();
    let proof_kc = native_prove(&cfg_kc, inputs, AIR_PATH).unwrap();

    assert_ne!(&proof_b3[40..], &proof_kc[40..]);

    assert!(native_verify(&cfg_b3, inputs, AIR_PATH, &proof_b3).unwrap());
    assert!(native_verify(&cfg_kc, inputs, AIR_PATH, &proof_kc).unwrap());

    assert!(native_verify(&cfg_b3, inputs, AIR_PATH, &proof_kc).is_err());
    assert!(native_verify(&cfg_kc, inputs, AIR_PATH, &proof_b3).is_err());
}

#[test]
fn poseidon2_and_rescue_placeholders_work() {
    for hash in ["poseidon2", "rescue"] {
        let cfg = Config::new("native@0.0", "Prime254", hash, 2, false, "balanced");
        let inputs = r#"{"k":"v"}"#;
        let proof = native_prove(&cfg, inputs, AIR_PATH).unwrap();
        assert!(native_verify(&cfg, inputs, AIR_PATH, &proof).unwrap());
    }
}
