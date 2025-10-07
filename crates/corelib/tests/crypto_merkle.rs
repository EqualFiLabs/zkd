use zkprov_corelib::crypto::blake3::Blake3;
use zkprov_corelib::crypto::merkle::*;

fn leaves(n: usize) -> Vec<Vec<u8>> {
    (0..n).map(|i| format!("leaf-{i}").into_bytes()).collect()
}

#[test]
fn merkle2_small_roots() {
    let ls = leaves(4);
    let r1 = root_arity2::<Blake3>(&ls);
    let r2 = root_arity2::<Blake3>(&ls); // deterministic
    assert_eq!(r1, r2);
}

#[test]
fn merkle4_small_roots() {
    let ls = leaves(8);
    let r1 = root_arity4::<Blake3>(&ls);
    let r2 = root_arity4::<Blake3>(&ls);
    assert_eq!(r1, r2);
}

#[test]
fn merkle2_vs_merkle4_differ() {
    let ls = leaves(5); // padding behavior differs
    let r2 = root_arity2::<Blake3>(&ls);
    let r4 = root_arity4::<Blake3>(&ls);
    assert_ne!(r2, r4);
}

#[test]
fn inclusion_proof_roundtrip_arity2() {
    let ls = leaves(7);
    let root = root_arity2::<Blake3>(&ls);
    for i in 0..ls.len() {
        let prf = prove_arity2::<Blake3>(&ls, i);
        assert!(verify_arity2::<Blake3>(&ls[i], i, &prf, &root));
    }
}
