use serde::{Deserialize, Serialize};

use super::{AirColumns, AirConstraints, AirMeta, AirProgram};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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
pub enum CommitmentKind {
    Pedersen { curve: String },
    PoseidonCommit,
    KeccakCommit,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CommitmentBinding {
    pub kind: CommitmentKind,
    #[serde(default)]
    pub public_inputs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct PublicInput {
    pub name: String,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub binding: Option<String>,
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
                r#type: pi.r#type,
                binding: pi.binding,
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
