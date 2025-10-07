use zkprov_corelib::gadgets::arithmetic::{
    add_under_commit_u64, commit_u64, scalar_mul_under_commit_u64,
};
use zkprov_corelib::gadgets::commitment::{
    CommitmentScheme32, PedersenParams, PedersenPlaceholder, Witness,
};
use zkprov_corelib::gadgets::range::range_check_u64;

fn ped() -> PedersenPlaceholder {
    PedersenPlaceholder::new(PedersenParams {
        hash_id: "blake3".into(),
    })
}

#[test]
fn add_under_commit_roundtrip() {
    let p = ped();
    let m1 = 7u64;
    let m2 = 9u64;
    let r1 = b"r1";
    let r2 = b"r2";

    let c1 = commit_u64(&p, m1, r1).unwrap();
    let c2 = commit_u64(&p, m2, r2).unwrap();

    let (c_sum, r12) = add_under_commit_u64(&p, m1, r1, m2, r2).unwrap();

    let sum = m1.wrapping_add(m2);
    assert!(p
        .open(
            &Witness {
                msg: &sum.to_le_bytes(),
                blind: &r12
            },
            &c_sum
        )
        .unwrap());

    let bad_sum = sum + 1;
    assert!(!p
        .open(
            &Witness {
                msg: &bad_sum.to_le_bytes(),
                blind: &r12
            },
            &c_sum
        )
        .unwrap());
    assert!(!p
        .open(
            &Witness {
                msg: &sum.to_le_bytes(),
                blind: b"wrong"
            },
            &c_sum
        )
        .unwrap());

    assert_ne!(c_sum.0, c1.0);
    assert_ne!(c_sum.0, c2.0);
}

#[test]
fn scalar_mul_under_commit_roundtrip() {
    let p = ped();
    let m = 13u64;
    let r = b"r";
    let k = 5u64;

    let c = commit_u64(&p, m, r).unwrap();
    let (c_prime, r_prime) = scalar_mul_under_commit_u64(&p, m, r, k).unwrap();
    let prod = m.wrapping_mul(k);
    assert!(p
        .open(
            &Witness {
                msg: &prod.to_le_bytes(),
                blind: &r_prime
            },
            &c_prime
        )
        .unwrap());

    assert!(!p
        .open(
            &Witness {
                msg: &m.to_le_bytes(),
                blind: &r_prime
            },
            &c_prime
        )
        .unwrap());
    assert!(!p
        .open(
            &Witness {
                msg: &prod.to_le_bytes(),
                blind: b"x"
            },
            &c_prime
        )
        .unwrap());

    assert_ne!(c.0, c_prime.0);
}

#[test]
fn range_check_before_commit() {
    range_check_u64(255, 8).unwrap();
    assert!(range_check_u64(256, 8).is_err());
}
