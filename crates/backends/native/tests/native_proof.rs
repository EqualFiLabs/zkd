use zkprov_backend_native::{native_prove, native_verify};
use zkprov_corelib::config::Config;
use zkprov_corelib::proof::{hash64, ProofHeader};

const AIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../examples/air/toy.air");

#[test]
fn prove_verify_with_air() {
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false, "balanced");
    let inputs = r#"{"a":1,"b":[2,3]}"#;
    let proof = native_prove(&cfg, inputs, AIR).expect("prove");
    assert!(native_verify(&cfg, inputs, AIR, &proof).expect("verify"));

    // Header sanity
    let hdr = ProofHeader::decode(&proof[0..40]).unwrap();
    assert_eq!(hdr.backend_id_hash, hash64("BACKEND", b"native@0.0"));
    assert_eq!(hdr.profile_id_hash, hash64("PROFILE", b"balanced"));
}

#[test]
fn verify_fails_on_different_air() {
    // Create a small temp AIR by tweaking rows_hint → different fake root
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        tmp.path(),
        r#"rows_hint = 32768

[meta]
name = "toy_merkle"
field = "Prime254"
hash = "blake3"

[columns]
trace_cols = 4
const_cols = 1
periodic_cols = 1

[constraints]
transition_count = 3
boundary_count = 2
"#,
    )
    .unwrap();

    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false, "balanced");
    let inputs = r#"{"a":1}"#;
    let proof = native_prove(&cfg, inputs, AIR).unwrap();
    let ok = native_verify(&cfg, inputs, tmp.path().to_str().unwrap(), &proof);
    assert!(ok.is_err(), "verify must fail when AIR changes");
}
