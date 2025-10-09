use std::collections::BTreeMap;

use anyhow::{anyhow, ensure, Result};
use serde::{Deserialize, Serialize};

/// Determinism manifest metadata emitted alongside proofs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DeterminismVector {
    pub backend: String,
    pub manifest_hash: String,
    #[serde(default)]
    pub compiler_commit: Option<String>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub seed: Option<String>,
}

/// Structured validation report propagated through bindings and CLI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ValidationReport {
    pub commit_passed: bool,
    pub vector_passed: bool,
    #[serde(default)]
    pub determinism: DeterminismVector,
}

impl ValidationReport {
    /// Ensure the manifest hash embedded in the proof matches the expected derivation.
    pub fn verify_manifest_hash(&self, expected_hash: &str) -> Result<()> {
        ensure!(
            self.vector_passed,
            "golden vector parity must pass before validating manifest"
        );
        if self.determinism.manifest_hash != expected_hash {
            return Err(anyhow!(
                "determinism manifest mismatch: expected {}, saw {}",
                expected_hash,
                self.determinism.manifest_hash
            ));
        }
        Ok(())
    }
}

/// Utility to assert that all backend digests within a registry entry match exactly.
pub fn assert_digest_parity(digests: &BTreeMap<String, String>) -> Result<()> {
    if digests.is_empty() {
        return Err(anyhow!("no digests supplied for parity check"));
    }
    let mut iter = digests.iter();
    let (_, first) = iter.next().unwrap();
    for (backend, digest) in iter {
        if digest != first {
            return Err(anyhow!(
                "digest mismatch for backend {}: expected {}",
                backend,
                first
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_hash_verification_passes() {
        let report = ValidationReport {
            commit_passed: true,
            vector_passed: true,
            determinism: DeterminismVector {
                backend: "native@0.0".to_string(),
                manifest_hash: "abc123".to_string(),
                compiler_commit: Some("deadbeef".to_string()),
                system: Some("linux-x86_64".to_string()),
                seed: Some("000102".to_string()),
            },
        };
        report
            .verify_manifest_hash("abc123")
            .expect("hash should match");
    }

    #[test]
    fn manifest_hash_verification_detects_mismatch() {
        let report = ValidationReport {
            commit_passed: true,
            vector_passed: true,
            determinism: DeterminismVector {
                backend: "native@0.0".to_string(),
                manifest_hash: "abc123".to_string(),
                ..Default::default()
            },
        };
        let err = report.verify_manifest_hash("zzz").unwrap_err();
        assert!(err.to_string().contains("determinism manifest mismatch"));
    }

    #[test]
    fn digest_parity_detects_inconsistency() {
        let mut digests = BTreeMap::new();
        digests.insert("native".to_string(), "00ff".to_string());
        digests.insert("winterfell".to_string(), "00aa".to_string());
        assert!(assert_digest_parity(&digests).is_err());
    }

    #[test]
    fn digest_parity_passes_when_equal() {
        let mut digests = BTreeMap::new();
        digests.insert("native".to_string(), "ff".to_string());
        digests.insert("winterfell".to_string(), "ff".to_string());
        assert!(assert_digest_parity(&digests).is_ok());
    }
}
