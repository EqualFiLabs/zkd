use std::fs;
use std::process::Command;
use tempfile::tempdir;

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
