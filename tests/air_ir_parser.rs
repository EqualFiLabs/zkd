use std::path::Path;
use std::path::PathBuf;

use zkprov_corelib::air::types::{CommitmentBinding, CommitmentKind};
use zkprov_corelib::air::{parse_air_file, parse_air_str};

fn base_air() -> String {
    r#"
[meta]
name = "demo"
field = "Prime254"
hash = "poseidon2"

[columns]
trace_cols = 4
const_cols = 0
periodic_cols = 0

[constraints]
transition_count = 1
boundary_count = 1

[[public_inputs]]
name = "x"

[[public_inputs]]
name = "y"

[[public_inputs]]
name = "acc"

[[public_inputs]]
name = "digest"
"#
    .to_string()
}

fn air_with_commitments(section: &str) -> String {
    format!("{}\n{}\n", base_air(), section)
}

fn expected_bindings() -> Vec<CommitmentBinding> {
    vec![
        CommitmentBinding {
            kind: CommitmentKind::Pedersen {
                curve: "placeholder".to_string(),
            },
            public_inputs: vec!["x".to_string(), "y".to_string()],
        },
        CommitmentBinding {
            kind: CommitmentKind::PoseidonCommit,
            public_inputs: vec!["acc".to_string()],
        },
        CommitmentBinding {
            kind: CommitmentKind::KeccakCommit,
            public_inputs: vec!["digest".to_string()],
        },
    ]
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct BindingKey(String, String, Vec<String>);

fn sort_bindings(bindings: &mut Vec<CommitmentBinding>) {
    bindings.sort_by(|a, b| binding_key(a).cmp(&binding_key(b)));
}

fn binding_key(binding: &CommitmentBinding) -> BindingKey {
    let kind_label = match &binding.kind {
        CommitmentKind::Pedersen { .. } => "pedersen".to_string(),
        CommitmentKind::PoseidonCommit => "poseidon_commit".to_string(),
        CommitmentKind::KeccakCommit => "keccak_commit".to_string(),
    };
    let curve_label = match &binding.kind {
        CommitmentKind::Pedersen { curve } => curve.clone(),
        _ => String::new(),
    };
    let mut publics = binding.public_inputs.clone();
    publics.sort();
    BindingKey(kind_label, curve_label, publics)
}

#[test]
fn parse_commitments_table_section() {
    let src = air_with_commitments(
        r#"[commitments]
    pedersen = { curve = "placeholder", public = ["x", "y"] }
    poseidon_commit = { public = ["acc"] }
    keccak_commit = { public = ["digest"] }
    "#,
    );

    let ir = parse_air_str(&src).expect("parse commitments table");
    let mut actual = ir.commitments;
    let mut expected = expected_bindings();
    sort_bindings(&mut actual);
    sort_bindings(&mut expected);
    assert_eq!(actual, expected);
    assert_eq!(ir.public_inputs.len(), 4);
}

#[test]
fn parse_commitments_from_file() {
    let path: PathBuf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/air/commit_demo.air");
    let ir = parse_air_file(&path).expect("parse sample file");
    let mut actual = ir.commitments;
    let mut expected = expected_bindings();
    sort_bindings(&mut actual);
    sort_bindings(&mut expected);
    assert_eq!(actual, expected);
    assert_eq!(ir.public_inputs.len(), 4);
}

#[test]
fn pedersen_missing_curve_errors() {
    let src = air_with_commitments(
        r#"[commitments]
    pedersen = { public = ["x"] }
    "#,
    );
    let err = parse_air_str(&src).expect_err("missing curve error");
    assert_eq!(err.to_string(), "CommitmentBindingMissingCurve");
}

#[test]
fn unexpected_curve_for_poseidon_errors() {
    let src = air_with_commitments(
        r#"[commitments]
    poseidon_commit = { curve = "foo", public = ["acc"] }
    "#,
    );
    let err = parse_air_str(&src).expect_err("unexpected curve error");
    assert!(
        err.chain().any(|cause| cause
            .to_string()
            .contains("CommitmentBindingUnexpectedCurve")),
        "unexpected error: {}",
        err
    );
}

#[test]
fn unknown_public_input_errors() {
    let src = air_with_commitments(
        r#"[commitments]
    pedersen = { curve = "placeholder", public = ["unknown"] }
    "#,
    );
    let err = parse_air_str(&src).expect_err("unknown public input error");
    assert_eq!(
        err.to_string(),
        "CommitmentBindingUnknownPublicInput(\"unknown\")"
    );
}

#[test]
fn duplicate_binding_errors() {
    let src = format!(
        "{base}\n[[commitments]]\nkind = \"PoseidonCommit\"\npublic = [\"acc\"]\n\n[[commitments]]\nkind = \"PoseidonCommit\"\npublic = [\"acc\"]\n",
        base = base_air()
    );
    let err = parse_air_str(&src).expect_err("duplicate binding error");
    assert_eq!(
        err.to_string(),
        "CommitmentBindingDuplicate(\"poseidon_commit\",\"acc\")"
    );
}
