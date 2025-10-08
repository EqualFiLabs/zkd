use zkprov_corelib::crypto::blake3::Blake3;
use zkprov_corelib::crypto::field::{h2f_32_be, h2f_64_be, prime254_modulus};
use zkprov_corelib::crypto::hash::{hash_labeled, hash_one_shot};

#[test]
fn h2f_basic_bounds() {
    let p = prime254_modulus();
    let d = hash_one_shot::<Blake3>(b"hello");
    let x = h2f_32_be(d);
    assert!(x < p);

    let a = hash_labeled::<Blake3>("A", b"hello");
    let b = hash_labeled::<Blake3>("B", b"hello");
    let y = h2f_64_be(a, b);
    assert!(y < p);
    assert_ne!(x, y);
}
