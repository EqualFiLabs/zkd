use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_zkd");

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN).args(args).output().expect("run");
    let code = out.status.code().unwrap_or(-1);
    (
        code,
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn commit_and_open_roundtrip() {
    let (code, c_hex, _err) = run(&[
        "commit",
        "--hash",
        "blake3",
        "--msg-hex",
        "010203",
        "--blind-hex",
        "aa55",
    ]);
    assert_eq!(code, 0, "commit exit code");
    let c_hex = c_hex.trim();
    assert_eq!(c_hex.len(), 64, "commit hex length");

    let (code, out, _err) = run(&[
        "open-commit",
        "--hash",
        "blake3",
        "--msg-hex",
        "010203",
        "--blind-hex",
        "aa55",
        "--commit-hex",
        c_hex,
    ]);
    assert_eq!(code, 0, "open exit code");
    assert!(out.contains("✅"), "open output");
}

#[test]
fn open_fails_with_wrong_blind() {
    let (_code, c_hex, _err) = run(&[
        "commit",
        "--hash",
        "blake3",
        "--msg-hex",
        "00",
        "--blind-hex",
        "00",
    ]);
    let c_hex = c_hex.trim();

    let (code, out, _err) = run(&[
        "open-commit",
        "--hash",
        "blake3",
        "--msg-hex",
        "00",
        "--blind-hex",
        "01",
        "--commit-hex",
        c_hex,
    ]);
    assert_ne!(code, 0, "open exit code should be non-zero");
    assert!(out.contains("❌"), "open output should show failure");
}
