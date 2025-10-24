use serde::{Deserialize, Serialize};

use super::{AirColumns, AirConstraints, AirMeta, AirProgram};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AirIr {
    pub meta: AirMeta,
    pub columns: AirColumns,
    pub constraints: AirConstraints,
    #[serde(default)]
    pub commitment_bindings: Vec<CommitmentBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CommitmentBinding {
    Pedersen {
        #[serde(default)]
        curve: Option<String>,
    },
    PoseidonCommit,
    KeccakCommit,
}

impl From<AirProgram> for AirIr {
    fn from(program: AirProgram) -> Self {
        let AirProgram {
            meta,
            columns,
            constraints,
            commitments,
            ..
        } = program;

        let commitment_bindings = match commitments {
            Some(c) if c.pedersen => {
                vec![CommitmentBinding::Pedersen { curve: c.curve }]
            }
            _ => Vec::new(),
        };

        Self {
            meta,
            columns,
            constraints,
            commitment_bindings,
        }
    }
}
