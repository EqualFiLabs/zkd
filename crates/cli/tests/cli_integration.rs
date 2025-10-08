use std::fs;
use std::process::Command;
use tempfile::tempdir;

use zkprov_corelib::evm::digest::digest_D;
use zkprov_corelib::proof::ProofHeader;

const BIN: &str = env!("CARGO_BIN_EXE_zkd");
fn air_path() -> String {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    base.join("../../examples/air/toy.air")
        .to_str()
        .expect("utf8 path")
        .to_owned()
}

fn write(path: &std::path::Path, s: &str) {
    fs::write(path, s).expect("write");
}

fn testdata_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/evm_verifier/testdata")
}

#[test]
fn corrupted_proof_yields_exit_4() {
    let dir = tempdir().unwrap();
    let inputs = dir.path().join("inputs.json");
    let proof = dir.path().join("ok.proof");
    write(&inputs, r#"{"demo":true,"n":7}"#);

    // Prove
    let air = air_path();
    let status = Command::new(BIN)
        .args([
            "prove",
            "-p",
            &air,
            "-i",
            inputs.to_str().unwrap(),
            "-o",
            proof.to_str().unwrap(),
            "--backend",
            "native@0.0",
            "--field",
            "Prime254",
            "--hash",
            "blake3",
            "--fri-arity",
            "2",
            "--profile",
            "balanced",
        ])
        .status()
        .expect("run prove");
    assert!(status.success());

    // Tamper last byte
    let mut buf = fs::read(&proof).unwrap();
    let last = buf.len() - 1;
    buf[last] ^= 0xFF;
    let bad = dir.path().join("bad.proof");
    fs::write(&bad, buf).unwrap();

    // Verify should exit with 4
    let status = Command::new(BIN)
        .args([
            "verify",
            "-p",
            &air,
            "-i",
            inputs.to_str().unwrap(),
            "-P",
            bad.to_str().unwrap(),
            "--backend",
            "native@0.0",
            "--field",
            "Prime254",
            "--hash",
            "blake3",
            "--fri-arity",
            "2",
            "--profile",
            "balanced",
        ])
        .status()
        .expect("run verify");
    assert_eq!(status.code(), Some(4));
}

#[test]
fn io_schema_outputs_json() {
    let air = air_path();
    let out = Command::new(BIN)
        .args(["io-schema", "-p", &air])
        .output()
        .expect("run io-schema");
    assert!(out.status.success());
    let s = String::from_utf8(out.stdout).unwrap();
    // Must be valid JSON and include program field name
    let v: serde_json::Value = serde_json::from_str(&s).expect("json");
    assert_eq!(v["program"], serde_json::json!("toy_merkle"));
}

#[test]
fn evm_digest_matches_testdata_fixture() {
    let tmp = tempdir().expect("tempdir");
    let inputs_path = tmp.path().join("inputs.json");
    let proof_path = tmp.path().join("toy.proof");
    write(&inputs_path, r#"{"a":1,"b":[2,3]}"#);

    let air = air_path();
    let status = Command::new(BIN)
        .args([
            "prove",
            "-p",
            &air,
            "-i",
            inputs_path.to_str().unwrap(),
            "-o",
            proof_path.to_str().unwrap(),
            "--backend",
            "native@0.0",
            "--field",
            "Prime254",
            "--hash",
            "blake3",
            "--fri-arity",
            "2",
            "--profile",
            "balanced",
        ])
        .status()
        .expect("run prove");
    assert!(status.success());

    let proof = fs::read(&proof_path).expect("read proof");
    assert!(proof.len() > 40, "proof missing body");
    let header = ProofHeader::decode(&proof[0..40]).expect("decode header");
    let body = &proof[40..];
    assert_eq!(body.len() as u64, header.body_len);

    let digest = digest_D(&header, body);
    let expected_hex: String = digest.iter().map(|b| format!("{:02x}", b)).collect();
    let expected = format!("0x{}", expected_hex);

    let output = Command::new(BIN)
        .args(["evm-digest", "-P", proof_path.to_str().unwrap()])
        .output()
        .expect("run evm-digest");

    let stdout = String::from_utf8(output.stdout).expect("stdout utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr utf8");
    assert!(output.status.success(), "stderr: {}", stderr);
    assert_eq!(stdout.trim(), expected);

    let digest_path = testdata_dir().join("digest.hex");
    let fixture_hex = fs::read_to_string(&digest_path)
        .expect("read digest")
        .trim()
        .to_owned();
    assert_eq!(expected_hex, fixture_hex);
}
