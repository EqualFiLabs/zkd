use zkprov_corelib::crypto::registry::hash64_by_id;
use zkprov_corelib::proof::{hash64, HEADER_HASH_ID};

#[test]
fn header_hash_matches_registry_policy() {
    // Our policy says header hash is blake3 for now.
    assert_eq!(HEADER_HASH_ID, "blake3");

    let lbl = "TEST";
    let data = b"The cake is a lie.";

    let via_policy = hash64(lbl, data);
    let via_registry = hash64_by_id("blake3", lbl, data).unwrap();
    assert_eq!(via_policy, via_registry);

    // Sanity: a different id would generally change it.
    let via_keccak = hash64_by_id("keccak256", lbl, data).unwrap();
    assert_ne!(via_policy, via_keccak);
}
