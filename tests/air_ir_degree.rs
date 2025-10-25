use zkprov_corelib::air::{parse_air_str, AirIr};

fn base_air() -> String {
    r#"[meta]
name = "commitment_degree"
field = "Prime254"
hash = "poseidon2"
degree_hint = 5

[columns]
trace_cols = 8
const_cols = 1
periodic_cols = 2

[constraints]
transition_count = 3
boundary_count = 2

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

fn air_with_commitments() -> String {
    let mut src = base_air();
    src.push_str(
        r#"
[commitments]
pedersen = { curve = "placeholder", public = ["x", "y"] }
poseidon_commit = { public = ["acc"] }
keccak_commit = { public = ["digest"] }
"#,
    );
    src
}

#[derive(Debug, PartialEq, Eq)]
struct DegreeMetrics {
    degree_hint: Option<u32>,
    trace_cols: u32,
    const_cols: u32,
    periodic_cols: u32,
    transition_count: u32,
    boundary_count: u32,
}

fn degree_metrics(ir: &AirIr) -> DegreeMetrics {
    DegreeMetrics {
        degree_hint: ir.meta.degree_hint,
        trace_cols: ir.columns.trace_cols,
        const_cols: ir.columns.const_cols,
        periodic_cols: ir.columns.periodic_cols,
        transition_count: ir.constraints.transition_count,
        boundary_count: ir.constraints.boundary_count,
    }
}

#[test]
fn degree_accounting_preserves_metrics_with_bindings() {
    let without_bindings = parse_air_str(&base_air()).expect("parse AIR without bindings");
    let with_bindings = parse_air_str(&air_with_commitments()).expect("parse AIR with bindings");

    let without_metrics = degree_metrics(&without_bindings);
    let with_metrics = degree_metrics(&with_bindings);

    assert_eq!(without_metrics, with_metrics);
}
