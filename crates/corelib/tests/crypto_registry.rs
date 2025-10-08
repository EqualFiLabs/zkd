use zkprov_corelib::crypto::registry::{hash32_by_id, hash64_by_id};

#[test]
fn registry_known_ids() {
    for id in ["blake3", "keccak256", "poseidon2", "rescue"] {
        let digest = hash32_by_id(id, "LBL", b"data").expect("supported id");
        assert_eq!(digest.len(), 32);
        let _ = hash64_by_id(id, "LBL", b"data").expect("u64");
    }
}

#[test]
fn registry_unknown_id_none() {
    assert!(hash32_by_id("unknown", "LBL", b"data").is_none());
    assert!(hash64_by_id("unknown", "LBL", b"data").is_none());
}
