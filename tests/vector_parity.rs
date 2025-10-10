use std::collections::BTreeMap;

use zkprov_corelib::validation::{assert_digest_parity, DeterminismVector, ValidationReport};

#[test]
fn golden_vector_parity_detects_mismatch() {
    let mut digests = BTreeMap::new();
    digests.insert("native@0.0".to_string(), "deadbeef".to_string());
    digests.insert("winterfell@0.6".to_string(), "feedface".to_string());
    let err = assert_digest_parity(&digests).unwrap_err();
    assert!(err.to_string().contains("digest mismatch"));
}

#[test]
fn determinism_vector_manifest_validation() {
    let report = ValidationReport {
        commit_passed: true,
        vector_passed: true,
        determinism: DeterminismVector {
            backend: "native@0.0".to_string(),
            manifest_hash: "cafebabe".to_string(),
            compiler_commit: Some("1234567".to_string()),
            system: Some("linux-x86_64".to_string()),
            seed: Some("01020304".to_string()),
        },
    };
    report
        .verify_manifest_hash("cafebabe")
        .expect("manifest hash should match");
}
