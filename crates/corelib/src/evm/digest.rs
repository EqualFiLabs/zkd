use alloy_sol_types::{sol, SolValue};

use crate::crypto::hash::hash_one_shot;
use crate::crypto::keccak::Keccak256;
use crate::proof::ProofHeader;

sol! {
    struct EvmDigestInput {
        uint64 backendIdHash;
        uint64 profileIdHash;
        uint64 pubioHash;
        uint64 bodyLen;
        bytes body;
    }
}

pub fn keccak256_bytes(data: &[u8]) -> [u8; 32] {
    hash_one_shot::<Keccak256>(data)
}

#[allow(non_snake_case)]
pub fn digest_D(header: &ProofHeader, body: &[u8]) -> [u8; 32] {
    let payload = EvmDigestInput {
        backendIdHash: header.backend_id_hash,
        profileIdHash: header.profile_id_hash,
        pubioHash: header.pubio_hash,
        bodyLen: header.body_len,
        body: body.to_vec().into(),
    };
    let encoded = payload.abi_encode();
    keccak256_bytes(&encoded)
}
