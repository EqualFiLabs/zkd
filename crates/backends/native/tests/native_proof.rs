use zkprov_backend_native::{native_prove, native_verify};
use zkprov_corelib::config::Config;

#[test]
fn stub_prove_verify_roundtrip() {
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false);
    let inputs = r#"{"a":1,"b":[2,3]}"#;
    let proof = native_prove(&cfg, inputs).expect("prove");
    assert!(native_verify(&cfg, inputs, &proof).expect("verify"));
}

#[test]
fn verify_fails_on_pubio_change() {
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false);
    let inputs = r#"{"k":"v"}"#;
    let proof = native_prove(&cfg, inputs).unwrap();
    let tampered_inputs = r#"{"k":"w"}"#;
    let ok = native_verify(&cfg, tampered_inputs, &proof);
    assert!(ok.is_err());
}

#[test]
fn verify_fails_on_corruption() {
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false);
    let inputs = r#"{}"#;
    let mut proof = native_prove(&cfg, inputs).unwrap();
    // flip a byte in the body
    let last = proof.len() - 1;
    proof[last] ^= 0xFF;
    let ok = native_verify(&cfg, inputs, &proof);
    assert!(ok.is_err());
}
