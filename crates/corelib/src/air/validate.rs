use std::collections::HashSet;

use anyhow::{anyhow, Result};

use super::types::{AirIr, CommitmentKind};

pub fn validate_bindings(ir: &AirIr) -> Result<()> {
    let declared: HashSet<&str> = ir.public_inputs.iter().map(|pi| pi.name.as_str()).collect();

    let mut seen = HashSet::new();

    for binding in &ir.commitments {
        match &binding.kind {
            CommitmentKind::Pedersen { curve } => {
                if curve.trim().is_empty() {
                    return Err(anyhow!("CommitmentBindingMissingCurve"));
                }
            }
            CommitmentKind::PoseidonCommit | CommitmentKind::KeccakCommit => {}
        }

        let kind_label = commitment_kind_label(&binding.kind);
        for name in &binding.public_inputs {
            if !declared.contains(name.as_str()) {
                return Err(anyhow!(format!(
                    "CommitmentBindingUnknownPublicInput(\"{}\")",
                    name
                )));
            }
            let key = format!("{}:{}", kind_label, name);
            if !seen.insert(key) {
                return Err(anyhow!(format!(
                    "CommitmentBindingDuplicate(\"{}\",\"{}\")",
                    kind_label, name
                )));
            }
        }
    }

    Ok(())
}

fn commitment_kind_label(kind: &CommitmentKind) -> &'static str {
    match kind {
        CommitmentKind::Pedersen { .. } => "pedersen",
        CommitmentKind::PoseidonCommit => "poseidon_commit",
        CommitmentKind::KeccakCommit => "keccak_commit",
    }
}
