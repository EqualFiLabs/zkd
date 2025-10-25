use std::collections::HashSet;

use anyhow::{anyhow, ensure, Result};

use super::types::{AirIr, CommitmentKind};

pub fn validate_bindings(ir: &AirIr) -> Result<()> {
    let declared: HashSet<&str> = ir.public_inputs.iter().map(|pi| pi.name.as_str()).collect();

    let mut seen: HashSet<(CommitmentKindLabel, String)> = HashSet::new();

    for binding in &ir.commitments {
        let label = CommitmentKindLabel::from(&binding.kind);
        match &binding.kind {
            CommitmentKind::Pedersen { curve } => {
                ensure!(!curve.trim().is_empty(), "CommitmentBindingMissingCurve");
            }
            CommitmentKind::PoseidonCommit | CommitmentKind::KeccakCommit => {}
        }

        for name in &binding.public_inputs {
            if !declared.contains(name.as_str()) {
                return Err(anyhow!(format!(
                    "CommitmentBindingUnknownPublicInput(\"{}\")",
                    name
                )));
            }
            if !seen.insert((label, name.clone())) {
                return Err(anyhow!(format!(
                    "CommitmentBindingDuplicate(\"{}\",\"{}\")",
                    label.as_str(),
                    name
                )));
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum CommitmentKindLabel {
    Pedersen,
    PoseidonCommit,
    KeccakCommit,
}

impl CommitmentKindLabel {
    fn as_str(&self) -> &'static str {
        match self {
            CommitmentKindLabel::Pedersen => "pedersen",
            CommitmentKindLabel::PoseidonCommit => "poseidon_commit",
            CommitmentKindLabel::KeccakCommit => "keccak_commit",
        }
    }
}

impl From<&CommitmentKind> for CommitmentKindLabel {
    fn from(kind: &CommitmentKind) -> Self {
        match kind {
            CommitmentKind::Pedersen { .. } => CommitmentKindLabel::Pedersen,
            CommitmentKind::PoseidonCommit => CommitmentKindLabel::PoseidonCommit,
            CommitmentKind::KeccakCommit => CommitmentKindLabel::KeccakCommit,
        }
    }
}
