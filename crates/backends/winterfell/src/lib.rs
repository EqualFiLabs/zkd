//! Winterfell backend adapter (stub).

use anyhow::{ensure, Result};
use thiserror::Error;
use zkprov_corelib::air::types::AirIr;
use zkprov_corelib::air::AirHash;
use zkprov_corelib::backend::{Capabilities, ProverBackend, VerifierBackend};

#[derive(Clone, Debug, serde::Serialize)]
pub struct WinterfellCapabilities {
    pub name: &'static str,
    pub field: &'static str,
    pub hashes: Vec<&'static str>,
    pub commitments: Vec<&'static str>,
    pub recursion: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub blowup: u8,
    pub fri_arity: u8,
    pub queries: u8,
    pub grinding: u8,
}

pub fn capabilities() -> WinterfellCapabilities {
    WinterfellCapabilities {
        name: "winterfell@0.6",
        field: "Prime256",
        hashes: vec!["blake3", "poseidon2", "rescue", "keccak"],
        commitments: vec!["Pedersen(placeholder)", "PoseidonCommit", "KeccakCommit"],
        recursion: false,
    }
}

pub fn profile_map(id: &str) -> Profile {
    match id {
        "fast" | "dev-fast" => Profile {
            blowup: 8,
            fri_arity: 2,
            queries: 24,
            grinding: 16,
        },
        "secure" => Profile {
            blowup: 32,
            fri_arity: 2,
            queries: 50,
            grinding: 20,
        },
        _ => Profile {
            blowup: 16,
            fri_arity: 2,
            queries: 30,
            grinding: 18,
        },
    }
}

#[derive(Debug, Default)]
pub struct WinterfellBackend;

impl ProverBackend for WinterfellBackend {
    fn id(&self) -> &'static str {
        "winterfell@0.6"
    }

    fn capabilities(&self) -> Capabilities {
        let wf_caps = capabilities();
        Capabilities {
            fields: vec![wf_caps.field],
            hashes: wf_caps.hashes.clone(),
            fri_arities: vec![2, 4],
            recursion: if wf_caps.recursion {
                "stark-in-stark"
            } else {
                "none"
            },
            lookups: false,
            curves: vec!["placeholder"],
            pedersen: wf_caps
                .commitments
                .iter()
                .any(|commitment| commitment.starts_with("Pedersen")),
        }
    }
}

impl VerifierBackend for WinterfellBackend {}

const DEFAULT_TRACE_ROWS: usize = 1 << 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToyDescriptor {
    pub transition_count: usize,
    pub boundary_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleDescriptor {
    pub hash: AirHash,
    pub arity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WfAirKind {
    Toy(ToyDescriptor),
    Merkle(MerkleDescriptor),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WfProgram {
    pub trace_cols: usize,
    pub trace_rows: usize,
    pub const_cols: usize,
    pub periodic_cols: usize,
    pub public_inputs: Vec<u64>,
    pub air: WfAirKind,
}

#[derive(Debug, Error)]
#[error("Unsupported({0})")]
struct UnsupportedError(String);

fn unsupported(msg: impl Into<String>) -> anyhow::Error {
    UnsupportedError(msg.into()).into()
}

fn convert_toy(ir: &AirIr) -> Result<WfProgram> {
    ensure!(
        ir.columns.trace_cols == 4,
        unsupported("toy AIR expects exactly 4 trace columns")
    );
    ensure!(
        ir.columns.const_cols == 1,
        unsupported("toy AIR expects exactly 1 constant column")
    );
    ensure!(
        ir.columns.periodic_cols == 1,
        unsupported("toy AIR expects exactly 1 periodic column")
    );
    ensure!(
        ir.constraints.transition_count == 3,
        unsupported("toy AIR expects 3 transition constraints")
    );
    ensure!(
        ir.constraints.boundary_count == 2,
        unsupported("toy AIR expects 2 boundary constraints")
    );

    let public_inputs = vec![0; ir.public_inputs.len()];
    Ok(WfProgram {
        trace_cols: ir.columns.trace_cols as usize,
        trace_rows: DEFAULT_TRACE_ROWS,
        const_cols: ir.columns.const_cols as usize,
        periodic_cols: ir.columns.periodic_cols as usize,
        public_inputs,
        air: WfAirKind::Toy(ToyDescriptor {
            transition_count: ir.constraints.transition_count as usize,
            boundary_count: ir.constraints.boundary_count as usize,
        }),
    })
}

fn convert_merkle(ir: &AirIr) -> Result<WfProgram> {
    ensure!(
        ir.columns.const_cols == 0,
        unsupported("merkle AIR must not declare constant columns")
    );
    ensure!(
        ir.columns.periodic_cols == 0,
        unsupported("merkle AIR must not declare periodic columns")
    );
    ensure!(
        ir.columns.trace_cols >= 16,
        unsupported("merkle AIR expects at least 16 trace columns to absorb root")
    );
    ensure!(
        ir.constraints.transition_count >= 1,
        unsupported("merkle AIR requires at least one transition constraint")
    );
    ensure!(
        ir.constraints.boundary_count >= 1,
        unsupported("merkle AIR requires at least one boundary constraint")
    );

    let public_inputs = vec![0; ir.public_inputs.len()];
    Ok(WfProgram {
        trace_cols: ir.columns.trace_cols as usize,
        trace_rows: DEFAULT_TRACE_ROWS,
        const_cols: 0,
        periodic_cols: 0,
        public_inputs,
        air: WfAirKind::Merkle(MerkleDescriptor {
            hash: ir.meta.hash.clone(),
            arity: ir.columns.trace_cols as usize,
        }),
    })
}

pub fn to_wf(ir: &AirIr) -> Result<WfProgram> {
    match ir.meta.name.as_str() {
        name if name.starts_with("toy") => convert_toy(ir),
        name if name.contains("merkle") => convert_merkle(ir),
        other => Err(unsupported(format!(
            "program '{other}' not supported by Winterfell backend"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zkprov_corelib::air::parser::parse_air_str;

    #[test]
    fn exposes_capabilities() {
        let caps = capabilities();
        assert_eq!(caps.name, "winterfell@0.6");
        assert_eq!(caps.field, "Prime256");
        assert_eq!(caps.hashes, vec!["blake3", "poseidon2", "rescue", "keccak"]);
        assert!(!caps.recursion);
    }

    #[test]
    fn profiles_have_reasonable_defaults() {
        let fast = profile_map("fast");
        let balanced = profile_map("balanced");
        let secure = profile_map("secure");

        assert!(fast.blowup < balanced.blowup);
        assert!(secure.blowup > balanced.blowup);
        assert_eq!(balanced.fri_arity, 2);
        assert_eq!(profile_map("unknown").blowup, balanced.blowup);
    }

    #[test]
    fn converts_toy_air_to_winterfell_program() {
        let src = include_str!("../../../../examples/air/toy.air");
        let ir = parse_air_str(src).expect("parse toy AIR");

        let wf = to_wf(&ir).expect("convert toy AIR");
        assert_eq!(wf.trace_cols, 4);
        assert_eq!(wf.const_cols, 1);
        assert_eq!(wf.periodic_cols, 1);
        assert_eq!(wf.trace_rows, DEFAULT_TRACE_ROWS);
        assert_eq!(wf.public_inputs.len(), ir.public_inputs.len());

        match wf.air {
            WfAirKind::Toy(ref toy) => {
                assert_eq!(toy.transition_count, 3);
                assert_eq!(toy.boundary_count, 2);
            }
            _ => panic!("expected toy descriptor"),
        }
    }

    #[test]
    fn rejects_unknown_program() {
        let src = r#"
            [meta]
            name = "unknown"
            field = "Prime254"
            hash = "blake3"

            [columns]
            trace_cols = 4

            [constraints]
            transition_count = 1
            boundary_count = 1
        "#;
        let mut ir = parse_air_str(src).expect("parse minimal AIR");
        ir.meta.name = "unknown".to_string();

        let err = to_wf(&ir).expect_err("should reject unsupported program");
        let msg = format!("{err}");
        assert!(msg.contains("Unsupported"));
    }
}
