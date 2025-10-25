use std::path::Path;
use std::path::PathBuf;

use zkprov_corelib::air::types::{CommitmentBinding, CommitmentKind, PublicInput, PublicTy};
use zkprov_corelib::air::{parse_air_file, parse_air_str};

fn expect_air_error(src: &str, expected: &str) {
    let err = parse_air_str(src).expect_err("expected AIR parse failure");
    let actual = err
        .chain()
        .last()
        .map(|cause| cause.to_string())
        .unwrap_or_else(|| err.to_string());
    let last_line = actual
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim())
        .unwrap_or_else(|| actual.as_str());
    assert_eq!(last_line, expected, "unexpected error chain: {err:#}");
}

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
type = "field"

[[public_inputs]]
name = "y"
type = "field"

[[public_inputs]]
name = "acc"
type = "bytes"

[[public_inputs]]
name = "digest"
type = "u64"
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

fn expected_public_inputs() -> Vec<PublicInput> {
    vec![
        PublicInput {
            name: "x".to_string(),
            ty: PublicTy::Field,
        },
        PublicInput {
            name: "y".to_string(),
            ty: PublicTy::Field,
        },
        PublicInput {
            name: "acc".to_string(),
            ty: PublicTy::Bytes,
        },
        PublicInput {
            name: "digest".to_string(),
            ty: PublicTy::U64,
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
    assert_eq!(ir.public_inputs, expected_public_inputs());
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
    assert_eq!(ir.public_inputs, expected_public_inputs());
}

#[test]
fn pedersen_missing_curve_errors() {
    let src = air_with_commitments(
        r#"[commitments]
    pedersen = { public = ["x"] }
    "#,
    );
    expect_air_error(&src, "pedersen commitment requires a curve name");
}

#[test]
fn pedersen_whitespace_curve_errors() {
    let src = air_with_commitments(
        r#"[commitments]
    pedersen = { curve = "   \t", public = ["x"] }
    "#,
    );
    expect_air_error(&src, "pedersen commitment requires a curve name");
}

#[test]
fn unexpected_curve_for_poseidon_errors() {
    let src = air_with_commitments(
        r#"[commitments]
    poseidon_commit = { curve = "foo", public = ["acc"] }
    "#,
    );
    expect_air_error(&src, "poseidon_commit commitment must not set a curve");
}

#[test]
fn unexpected_curve_for_keccak_errors() {
    let src = air_with_commitments(
        r#"[commitments]
    keccak_commit = { curve = "foo", public = ["digest"] }
    "#,
    );
    expect_air_error(&src, "keccak_commit commitment must not set a curve");
}

#[test]
fn unknown_public_input_errors() {
    let src = air_with_commitments(
        r#"[commitments]
    pedersen = { curve = "placeholder", public = ["unknown"] }
    "#,
    );
    expect_air_error(
        &src,
        "unknown public input 'unknown' referenced by pedersen",
    );
}

#[test]
fn duplicate_binding_errors() {
    let src = format!(
        "{base}\n[[commitments]]\nkind = \"PoseidonCommit\"\npublic = [\"acc\"]\n\n[[commitments]]\nkind = \"PoseidonCommit\"\npublic = [\"acc\"]\n",
        base = base_air()
    );
    expect_air_error(&src, "public input 'acc' already bound to poseidon_commit");
}

#[test]
fn duplicate_binding_within_entry_errors() {
    let src = air_with_commitments(
        r#"[commitments]
    pedersen = { curve = "placeholder", public = ["x", "x"] }
    "#,
    );
    expect_air_error(&src, "public input 'x' already bound to pedersen");
}

#[test]
fn invalid_public_input_type_errors() {
    let mut src = base_air();
    src = src.replacen("type = \"field\"", "type = \"unknown\"", 1);
    expect_air_error(&src, "unknown public input type 'unknown'");
}

#[test]
fn invalid_meta_name_errors() {
    let mut src = base_air();
    src = src.replacen("name = \"demo\"", "name = \"bad name\"", 1);
    expect_air_error(&src, "invalid meta.name 'bad name'");
}

#[test]
fn empty_meta_field_errors() {
    let mut src = base_air();
    src = src.replacen("field = \"Prime254\"", "field = \"   \"", 1);
    expect_air_error(&src, "meta.field cannot be empty");
}

#[test]
fn zero_trace_columns_errors() {
    let mut src = base_air();
    src = src.replacen("trace_cols = 4", "trace_cols = 0", 1);
    expect_air_error(&src, "columns.trace_cols must be > 0");
}

#[test]
fn too_many_trace_columns_errors() {
    let mut src = base_air();
    src = src.replacen("trace_cols = 4", "trace_cols = 2049", 1);
    expect_air_error(
        &src,
        "columns.trace_cols too large (>2048) for default limits",
    );
}

#[test]
fn zero_transition_count_errors() {
    let mut src = base_air();
    src = src.replacen("transition_count = 1", "transition_count = 0", 1);
    expect_air_error(&src, "constraints.transition_count must be > 0");
}

#[test]
fn degree_hint_out_of_range_errors() {
    let mut src = base_air();
    src = src.replacen(
        "field = \"Prime254\"",
        "field = \"Prime254\"\ndegree_hint = 0",
        1,
    );
    expect_air_error(&src, "degree_hint out of range (1..=64)");
}

#[test]
fn rows_hint_out_of_range_errors() {
    let src = format!("rows_hint = 4\n{}", base_air());
    expect_air_error(&src, "rows_hint out of range [2^3 .. 2^22]");
}

#[test]
fn rows_hint_not_power_of_two_errors() {
    let src = format!("rows_hint = 24\n{}", base_air());
    expect_air_error(&src, "rows_hint must be a power of two");
}
