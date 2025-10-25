use serde::{Deserialize, Serialize};

use super::{AirColumns, AirConstraints, AirMeta, AirProgram};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Public input surface area supported by the AIR DSL.
///
/// Values default to [`PublicTy::Field`] when the `type` key is omitted in the
/// mini-DSL.
pub enum PublicTy {
    Field,
    Bytes,
    U64,
}

impl<'de> serde::Deserialize<'de> for PublicTy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "field" => Ok(Self::Field),
            "bytes" => Ok(Self::Bytes),
            "u64" => Ok(Self::U64),
            other => Err(serde::de::Error::custom(format!(
                "unknown public input type '{other}'"
            ))),
        }
    }
}

impl Default for PublicTy {
    fn default() -> Self {
        Self::Field
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// Backend-neutral AIR intermediate representation produced by the parser.
///
/// The structure mirrors the TOML/YAML schema and normalizes commitment
/// bindings into [`CommitmentBinding`] entries that can be validated in one
/// pass.
///
/// # Examples
///
/// ```
/// use zkprov_corelib::air::parser::parse_air_str;
/// use zkprov_corelib::air::types::{CommitmentKind, PublicTy};
///
/// let ir = parse_air_str(r#"
/// [meta]
/// name = "toy_balance"
/// field = "Prime254"
/// hash = "poseidon2"
/// degree_hint = 8
///
/// [columns]
/// trace_cols = 8
/// const_cols = 2
/// periodic_cols = 1
///
/// [constraints]
/// transition_count = 4
/// boundary_count = 2
///
/// [[public_inputs]]
/// name = "root"
/// type = "bytes"
///
/// commitments = [
///     { kind = "poseidon_commit", public = ["root"] }
/// ]
/// "#).unwrap();
///
/// assert_eq!(ir.meta.name, "toy_balance");
/// assert_eq!(ir.meta.degree_hint, Some(8));
/// assert_eq!(ir.degree_hint, Some(8));
/// assert_eq!(ir.public_inputs[0].ty, PublicTy::Bytes);
/// assert!(matches!(ir.commitments[0].kind, CommitmentKind::PoseidonCommit));
/// assert_eq!(ir.commitments[0].public_inputs, ["root".to_string()]);
/// ```
///
/// Commitment bindings describe wiring for prover commitments but never modify
/// [`AirIr::degree_hint`]; backend selection remains solely a function of the
/// declared degree and trace geometry.
pub struct AirIr {
    pub meta: AirMeta,
    pub columns: AirColumns,
    pub constraints: AirConstraints,
    #[serde(default)]
    pub degree_hint: Option<u32>,
    #[serde(default)]
    pub commitments: Vec<CommitmentBinding>,
    #[serde(default)]
    pub public_inputs: Vec<PublicInput>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
/// Supported commitment gadget families that can be requested from the AIR DSL.
pub enum CommitmentKind {
    Pedersen { curve: String },
    PoseidonCommit,
    KeccakCommit,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// Normalized commitment binding resolved from the AIR DSL.
///
/// Bindings connect commitment gadgets to one or more named public inputs, but
/// they never influence [`AirIr::degree_hint`]; degree calculations only depend
/// on the AIR constraints and metadata supplied by the author.
pub struct CommitmentBinding {
    pub kind: CommitmentKind,
    #[serde(default)]
    pub public_inputs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// Public input declared in the AIR source.
pub struct PublicInput {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: PublicTy,
}

impl From<AirProgram> for AirIr {
    fn from(program: AirProgram) -> Self {
        let AirProgram {
            meta,
            columns,
            constraints,
            public_inputs,
            commitments,
            ..
        } = program;

        let commitments = commitments.map(|c| c.bindings).unwrap_or_default();

        let public_inputs = public_inputs
            .into_iter()
            .map(|pi| PublicInput {
                name: pi.name,
                ty: pi.ty,
            })
            .collect();

        let degree_hint = meta.degree_hint;

        Self {
            meta,
            columns,
            constraints,
            degree_hint,
            commitments,
            public_inputs,
        }
    }
}
