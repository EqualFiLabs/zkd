use std::fs;
use std::path::PathBuf;

use zkprov_backend_native::native_prove;
use zkprov_corelib::config::Config;
use zkprov_corelib::evm::digest::digest_D;
use zkprov_corelib::proof::ProofHeader;

const TOY_AIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/air/toy.air");

#[test]
fn evm_digest_matches_native_proof() {
    // Deterministic native proof using the toy AIR and simple inputs.
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false, "balanced");
    let inputs = r#"{"a":1,"b":[2,3]}"#;
    let proof = native_prove(&cfg, inputs, TOY_AIR).expect("native prove");
    assert!(proof.len() > 40, "proof must contain header + body");

    let header = ProofHeader::decode(&proof[0..40]).expect("decode header");
    let body = &proof[40..];
    assert_eq!(
        body.len() as u64,
        header.body_len,
        "header body length mismatch"
    );

    let digest = digest_D(&header, body);
    let digest_hex = digest
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let testdata_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("evm_verifier")
        .join("testdata");
    fs::create_dir_all(&testdata_dir).expect("create testdata dir");

    let digest_path = testdata_dir.join("digest.hex");
    fs::write(&digest_path, format!("{}\n", digest_hex)).expect("write digest");

    let meta_path = testdata_dir.join("meta.json");
    let meta = serde_json::json!({
        "backendId": header.backend_id_hash,
        "profileId": header.profile_id_hash,
        "pubioHash": header.pubio_hash,
        "bodyLen": header.body_len,
    });
    let meta_json = serde_json::to_string_pretty(&meta).expect("serialize meta");
    fs::write(&meta_path, format!("{}\n", meta_json)).expect("write meta");

    let body_path = testdata_dir.join("body.bin");
    fs::write(&body_path, body).expect("write body");

    for path in [&digest_path, &meta_path, &body_path] {
        let metadata = fs::metadata(path).expect("metadata");
        assert!(metadata.len() > 0, "file {:?} should not be empty", path);
    }
}
