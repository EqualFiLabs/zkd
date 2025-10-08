use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;
use zkprov_corelib::evm::abi::{decode_body, decode_meta, encode_body, encode_meta};
use zkprov_corelib::proof::ProofHeader;

#[derive(Debug, Deserialize)]
struct MetaFixture {
    #[serde(rename = "backendId")]
    backend_id: u64,
    #[serde(rename = "profileId")]
    profile_id: u64,
    #[serde(rename = "pubioHash")]
    pubio_hash: u64,
    #[serde(rename = "bodyLen")]
    body_len: u64,
}

fn testdata_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("examples")
        .join("evm_verifier")
        .join("testdata")
}

#[test]
fn evm_abi_round_trip_matches_fixtures() -> Result<()> {
    let dir = testdata_dir();

    let meta_json = fs::read_to_string(dir.join("meta.json"))?;
    let meta_fixture: MetaFixture = serde_json::from_str(&meta_json)?;

    let header = ProofHeader {
        backend_id_hash: meta_fixture.backend_id,
        profile_id_hash: meta_fixture.profile_id,
        pubio_hash: meta_fixture.pubio_hash,
        body_len: meta_fixture.body_len,
    };

    let body = fs::read(dir.join("body.bin"))?;

    let meta_bytes = encode_meta(&header);
    let body_bytes = encode_body(&body);

    fs::write(dir.join("meta.abi"), &meta_bytes)?;
    fs::write(dir.join("body.abi"), &body_bytes)?;

    let decoded_header = decode_meta(&meta_bytes)?;
    assert_eq!(
        decoded_header, header,
        "decoded meta must match source header"
    );

    let decoded_body = decode_body(&body_bytes)?;
    assert_eq!(decoded_body, body, "decoded body must match fixture body");

    Ok(())
}
