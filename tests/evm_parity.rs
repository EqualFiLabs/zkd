use tiny_keccak::{Hasher, Keccak};

use alloy_sol_types::{sol, SolValue};
use zkprov_corelib::evm::digest::digest_D;
use zkprov_corelib::proof::ProofHeader;

fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let mut keccak = Keccak::v256();
    keccak.update(data);
    keccak.finalize(&mut out);
    out
}

fn encode_uint64(value: u64) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[24..32].copy_from_slice(&value.to_be_bytes());
    buf
}

fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&encode_uint64(bytes.len() as u64));
    let mut chunk = vec![0u8; bytes.len().div_ceil(32) * 32];
    chunk[..bytes.len()].copy_from_slice(bytes);
    out.extend_from_slice(&chunk);
    out
}

fn manual_encoding(header: &ProofHeader, body: &[u8]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&encode_uint64(32));
    encoded.extend_from_slice(&encode_uint64(header.backend_id_hash));
    encoded.extend_from_slice(&encode_uint64(header.profile_id_hash));
    encoded.extend_from_slice(&encode_uint64(header.pubio_hash));
    encoded.extend_from_slice(&encode_uint64(header.body_len));
    encoded.extend_from_slice(&encode_uint64(32 * 5));
    encoded.extend_from_slice(&encode_bytes(body));
    encoded
}

#[test]
fn digest_matches_manual_encoding() {
    let header = ProofHeader {
        backend_id_hash: 0x1111,
        profile_id_hash: 0x2222,
        pubio_hash: 0x3333,
        body_len: 3,
    };
    let body = vec![0xde, 0xad, 0xbe];
    let digest = digest_D(&header, &body);
    let manual_encoded = manual_encoding(&header, &body);
    let manual = keccak256(&manual_encoded);
    sol! {
        struct Input {
            uint64 backendIdHash;
            uint64 profileIdHash;
            uint64 pubioHash;
            uint64 bodyLen;
            bytes body;
        }
    }
    let encoded = Input {
        backendIdHash: header.backend_id_hash,
        profileIdHash: header.profile_id_hash,
        pubioHash: header.pubio_hash,
        bodyLen: header.body_len,
        body: body.clone().into(),
    }
    .abi_encode();
    assert_eq!(digest, manual);
    assert_eq!(manual_encoded, encoded);
}
