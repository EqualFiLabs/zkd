use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde_json::json;
use zkprov_backend_native::native_prove;
use zkprov_corelib::config::Config;
use zkprov_corelib::evm::{
    abi::{encode_body, encode_meta},
    digest::digest_D,
};
use zkprov_corelib::proof::ProofHeader;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn toy_air_path() -> Result<String> {
    let path = workspace_root()
        .join("examples")
        .join("air")
        .join("toy.air");
    path.to_str()
        .map(|s| s.to_owned())
        .context("toy.air path must be valid UTF-8")
}

fn testdata_dir() -> PathBuf {
    workspace_root()
        .join("examples")
        .join("evm_verifier")
        .join("testdata")
}

fn proof_header_and_body() -> Result<(ProofHeader, Vec<u8>)> {
    const INPUTS: &str = r#"{"a":1,"b":[2,3]}"#;
    let cfg = Config::new("native@0.0", "Prime254", "blake3", 2, false, "balanced");
    let proof = native_prove(&cfg, INPUTS, &toy_air_path()?)?;
    anyhow::ensure!(proof.len() >= 40, "proof must contain header and body");

    let header = ProofHeader::decode(&proof[0..40]).context("decode header")?;
    let body = proof[40..].to_vec();
    anyhow::ensure!(body.len() as u64 == header.body_len, "body length mismatch");
    Ok((header, body))
}

fn write_hex(path: &Path, data: &[u8]) -> Result<()> {
    let hex = data
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    fs::write(path, format!("{}\n", hex)).context("write hex")
}

#[test]
fn evm_end_to_end_parity() -> Result<()> {
    let (header, body) = proof_header_and_body()?;
    let digest = digest_D(&header, &body);

    let dir = testdata_dir();
    fs::create_dir_all(&dir).context("create testdata dir")?;

    let meta_json = json!({
        "backendId": header.backend_id_hash,
        "profileId": header.profile_id_hash,
        "pubioHash": header.pubio_hash,
        "bodyLen": header.body_len,
    });
    fs::write(
        dir.join("meta.json"),
        format!("{}\n", serde_json::to_string_pretty(&meta_json)?),
    )
    .context("write meta.json")?;
    fs::write(dir.join("body.bin"), &body).context("write body.bin")?;
    write_hex(&dir.join("digest.hex"), &digest)?;

    let meta_abi = encode_meta(&header);
    let body_abi = encode_body(&body);
    fs::write(dir.join("meta.abi"), &meta_abi).context("write meta.abi")?;
    fs::write(dir.join("body.abi"), &body_abi).context("write body.abi")?;

    for name in [
        "meta.json",
        "body.bin",
        "digest.hex",
        "meta.abi",
        "body.abi",
    ] {
        let path = dir.join(name);
        let metadata = fs::metadata(&path).with_context(|| format!("metadata for {:?}", path))?;
        anyhow::ensure!(metadata.len() > 0, "fixture {:?} should not be empty", path);
    }

    let status = Command::new("forge")
        .arg("test")
        .arg("-vv")
        .current_dir(workspace_root().join("examples").join("evm_verifier"))
        .status()
        .context("run forge test")?;
    anyhow::ensure!(status.success(), "forge test failed: {status}");

    Ok(())
}
