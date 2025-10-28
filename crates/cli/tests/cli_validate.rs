use std::fs;
use std::process::Command;

use tempfile::tempdir;
use zkprov_corelib::validation::ValidationReport;

const BIN: &str = env!("CARGO_BIN_EXE_zkd");

fn air_path() -> String {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    base.join("../../examples/air/toy.air")
        .to_str()
        .expect("utf8 path")
        .to_owned()
}

#[test]
fn validate_emits_report_with_commit_status() {
    let dir = tempdir().unwrap();
    let inputs_path = dir.path().join("inputs.json");
    let proof_path = dir.path().join("toy.proof");
    let reports_dir = dir.path().join("reports");

    fs::write(&inputs_path, r#"{"demo":true,"n":5}"#).expect("write inputs");

    let air = air_path();
    let prove_status = Command::new(BIN)
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
    assert!(prove_status.success(), "prove failed");

    let validate_status = Command::new(BIN)
        .args([
            "validate",
            "-p",
            &air,
            "-i",
            inputs_path.to_str().unwrap(),
            "-P",
            proof_path.to_str().unwrap(),
            "-o",
            reports_dir.to_str().unwrap(),
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
        .expect("run validate");
    assert!(validate_status.success(), "validate failed");

    let entries: Vec<_> = fs::read_dir(&reports_dir)
        .expect("list reports")
        .map(|res| res.expect("dir entry").path())
        .collect();
    assert_eq!(entries.len(), 1, "expected single report file");

    let report_contents = fs::read_to_string(&entries[0]).expect("read report");
    let report: ValidationReport = serde_json::from_str(&report_contents).expect("parse report");

    assert!(report.ok, "report must indicate success");
    assert!(report.commit_passed, "commitment checks must pass");
    assert_eq!(report.meta.backend_id, "native@0.0");
    assert_eq!(report.meta.profile_id, "balanced");
    assert_eq!(report.meta.hash_id, "blake3");
    assert_eq!(report.meta.curve.as_deref(), Some("placeholder"));
}
