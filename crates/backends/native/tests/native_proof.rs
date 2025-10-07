use zkprov_backend_native::{native_prove, native_verify};
use zkprov_corelib::config::Config;
use zkprov_corelib::proof::ProofHeader;

#[test]
fn stub_prove_verify_roundtrip_with_profile() {
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false, "balanced");
    let inputs = r#"{"a":1,"b":[2,3]}"#;
    let proof = native_prove(&cfg, inputs).expect("prove");
    assert!(native_verify(&cfg, inputs, &proof).expect("verify"));
    // Decode header to ensure profile hash was embedded
    let hdr = ProofHeader::decode(&proof[0..40]).unwrap();
    let expect = zkprov_corelib::proof::hash64("PROFILE", b"balanced");
    assert_eq!(hdr.profile_id_hash, expect);
}

#[test]
fn verify_fails_on_profile_mismatch() {
    let cfg1 = Config::new("native@0.0", "Prime254", "blake3", 2, false, "balanced");
    let cfg2 = Config::new("native@0.0", "Prime254", "blake3", 2, false, "secure");
    let inputs = r#"{"k":"v"}"#;
    let proof = native_prove(&cfg1, inputs).unwrap();
    // Verifying same proof under a different profile must fail
    let ok = native_verify(&cfg2, inputs, &proof);
    assert!(ok.is_err(), "verification should fail on profile mismatch");
}

#[test]
fn verify_fails_on_corruption_still() {
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false, "dev-fast");
    let inputs = r#"{}"#;
    let mut proof = native_prove(&cfg, inputs).unwrap();
    // flip a byte in the body
    let last = proof.len() - 1;
    proof[last] ^= 0xFF;
    let ok = native_verify(&cfg, inputs, &proof);
    assert!(ok.is_err());
}
