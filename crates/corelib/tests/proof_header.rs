use zkprov_corelib::proof::{assemble_proof, hash64, ProofHeader};

#[test]
fn header_roundtrip() {
    let hdr = ProofHeader {
        backend_id_hash: hash64("BACKEND", b"native@0.0"),
        profile_id_hash: hash64("PROFILE", b"default"),
        pubio_hash: hash64("PUBIO", br#"{"x":1}"#),
        body_len: 8,
    };
    let enc = hdr.encode();
    let dec = ProofHeader::decode(&enc).unwrap();
    assert_eq!(hdr, dec);

    let body = 12345678u64.to_le_bytes();
    let proof = assemble_proof(&hdr, &body);
    assert_eq!(proof.len(), 40 + 8);
}

#[test]
fn header_rejects_bad_magic_or_version() {
    let mut enc = ProofHeader {
        backend_id_hash: 1,
        profile_id_hash: 2,
        pubio_hash: 3,
        body_len: 0,
    }
    .encode();
    // Corrupt magic
    enc[0] = b'X';
    assert!(ProofHeader::decode(&enc).is_err());

    // Fix magic, corrupt version
    enc[0] = b'P';
    enc[4] ^= 0x01;
    assert!(ProofHeader::decode(&enc).is_err());
}
