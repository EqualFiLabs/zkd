use zkprov_corelib::gadgets::commitment::{
    CommitmentScheme32, PedersenParams, PedersenPlaceholder, Witness,
};

fn c(params: PedersenParams) -> PedersenPlaceholder {
    PedersenPlaceholder::new(params)
}

#[test]
fn commit_and_open_ok() {
    let ped = c(PedersenParams {
        hash_id: "blake3".into(),
    });
    let w = Witness {
        msg: b"hello",
        blind: b"r-123",
    };
    let commitment = ped.commit(&w).unwrap();
    assert!(ped.open(&w, &commitment).unwrap());
}

#[test]
fn binding_property() {
    let ped = c(PedersenParams {
        hash_id: "blake3".into(),
    });
    let w1 = Witness {
        msg: b"hello",
        blind: b"r-123",
    };
    let w2 = Witness {
        msg: b"hello!",
        blind: b"r-123",
    };
    let commitment = ped.commit(&w1).unwrap();
    assert!(ped.open(&w1, &commitment).unwrap());
    assert!(!ped.open(&w2, &commitment).unwrap());
}

#[test]
fn hiding_property_change_blind() {
    let ped = c(PedersenParams {
        hash_id: "blake3".into(),
    });
    let m = b"hello";
    let c1 = ped
        .commit(&Witness {
            msg: m,
            blind: b"r-1",
        })
        .unwrap();
    let c2 = ped
        .commit(&Witness {
            msg: m,
            blind: b"r-2",
        })
        .unwrap();
    assert_ne!(c1.0, c2.0); // blinding affects commitment
}

#[test]
fn different_hash_ids_change_commitment() {
    let p1 = c(PedersenParams {
        hash_id: "blake3".into(),
    });
    let p2 = c(PedersenParams {
        hash_id: "keccak256".into(),
    });
    let w = Witness {
        msg: b"M",
        blind: b"R",
    };
    assert_ne!(p1.commit(&w).unwrap().0, p2.commit(&w).unwrap().0);
}
