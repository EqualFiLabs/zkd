use alloy_sol_types::{sol, SolType, SolValue};
use anyhow::{anyhow, Result};

use crate::proof::ProofHeader;

sol! {
    /// ABI surface for proof metadata used by the EVM bridge.
    struct EvmProofMeta {
        uint64 backendId;
        uint64 profileId;
        uint64 pubioHash;
        uint64 bodyLen;
    }

    /// ABI container for serialized public IO JSON.
    struct EvmPublicIO {
        bytes data;
    }

    /// ABI container for proof body bytes.
    struct EvmProofBody {
        bytes data;
    }
}

pub fn encode_meta(header: &ProofHeader) -> Vec<u8> {
    let meta = EvmProofMeta {
        backendId: header.backend_id_hash,
        profileId: header.profile_id_hash,
        pubioHash: header.pubio_hash,
        bodyLen: header.body_len,
    };
    meta.abi_encode()
}

pub fn decode_meta(data: &[u8]) -> Result<ProofHeader> {
    let meta = <EvmProofMeta as SolType>::abi_decode(data, true)?;
    Ok(ProofHeader {
        backend_id_hash: meta.backendId,
        profile_id_hash: meta.profileId,
        pubio_hash: meta.pubioHash,
        body_len: meta.bodyLen,
    })
}

pub fn encode_body(body: &[u8]) -> Vec<u8> {
    body.abi_encode()
}

pub fn decode_body(data: &[u8]) -> Result<Vec<u8>> {
    <Vec<u8> as SolValue>::abi_decode(data, true).map_err(|e| anyhow!(e))
}

pub fn encode_public_io(json: &str) -> Vec<u8> {
    let public_io = EvmPublicIO {
        data: json.as_bytes().to_vec().into(),
    };
    public_io.abi_encode()
}

pub fn decode_public_io(data: &[u8]) -> Result<String> {
    let decoded = <EvmPublicIO as SolType>::abi_decode(data, true)?;
    String::from_utf8(decoded.data.to_vec()).map_err(|e| anyhow!(e))
}
