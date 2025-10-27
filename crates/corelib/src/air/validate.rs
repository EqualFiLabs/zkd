use std::collections::HashSet;

use anyhow::{bail, ensure, Result};

use super::types::{AirIr, CommitmentKind};

pub fn validate_bindings(ir: &AirIr) -> Result<()> {
    let declared: HashSet<&str> = ir.public_inputs.iter().map(|pi| pi.name.as_str()).collect();

    let mut seen: HashSet<(CommitmentKindLabel, String)> = HashSet::new();

    for binding in &ir.commitments {
        let label = CommitmentKindLabel::from(&binding.kind);
        match &binding.kind {
            CommitmentKind::Pedersen { curve } => {
                ensure!(
                    !curve.trim().is_empty(),
                    "pedersen commitment requires a curve name"
                );
            }
            CommitmentKind::PoseidonCommit | CommitmentKind::KeccakCommit => {}
        }

        for name in &binding.public_inputs {
            if !declared.contains(name.as_str()) {
                bail!(
                    "unknown public input '{}' referenced by {}",
                    name,
                    label.as_str()
                );
            }
            if !seen.insert((label, name.clone())) {
                bail!(
                    "public input '{}' already bound to {}",
                    name,
                    label.as_str()
                );
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
