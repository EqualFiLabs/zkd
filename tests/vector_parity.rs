use std::collections::BTreeMap;

use zkprov_corelib::validation::{assert_digest_parity, ReportMeta, ValidationReport};

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
    let meta = ReportMeta {
        backend_id: "native@0.0".to_string(),
        profile_id: "profile-a".to_string(),
        hash_id: "cafebabe".to_string(),
        curve: Some("bls12-377".to_string()),
        time_ms: 10,
    };
    let report = ValidationReport::new_ok(meta);
    report
        .verify_manifest_hash("cafebabe")
        .expect("manifest hash should match");
}
