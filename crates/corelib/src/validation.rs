use std::collections::BTreeMap;

use anyhow::{anyhow, ensure, Result};
use serde::{Deserialize, Serialize};

/// Structured validation report propagated through bindings and CLI.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationReport {
    pub ok: bool,
    pub commit_passed: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub meta: ReportMeta,
}

impl ValidationReport {
    /// Create a new report initialized with the supplied metadata.
    pub fn new(meta: ReportMeta) -> Self {
        Self::new_ok(meta)
    }

    /// Convenience helper that returns a success report with no findings.
    pub fn new_ok(meta: ReportMeta) -> Self {
        Self {
            ok: true,
            commit_passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            meta,
        }
    }

    /// Convenience helper that constructs a failed report with a single error entry.
    pub fn fail(
        meta: ReportMeta,
        code: ValidationErrorCode,
        msg: impl Into<String>,
        context: impl Into<serde_json::Value>,
    ) -> Self {
        let mut report = Self::new_ok(meta);
        report.commit_passed = false;
        report.push_error(ValidationError::new(code, msg, context));
        report
    }

    /// Append an error to the report and update success flags accordingly.
    pub fn push_error(&mut self, error: ValidationError) {
        self.ok = false;
        self.errors.push(error);
    }

    /// Append a warning to the report.
    pub fn push_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Overwrite the commit check result.
    pub fn set_commit_passed(&mut self, passed: bool) {
        self.commit_passed = passed;
        if !passed {
            self.ok = false;
        }
    }

    /// Ensure the manifest hash embedded in the proof matches the expected derivation.
    pub fn verify_manifest_hash(&self, expected_hash: &str) -> Result<()> {
        ensure!(
            self.commit_passed,
            "commitment checks must pass before validating manifest"
        );
        if self.meta.hash_id != expected_hash {
            return Err(anyhow!(
                "determinism manifest mismatch: expected {}, saw {}",
                expected_hash,
                self.meta.hash_id
            ));
        }
        Ok(())
    }

    /// Serialize the report into a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize a report from a JSON string.
    pub fn from_json(data: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMeta {
    pub backend_id: String,
    pub profile_id: String,
    pub hash_id: String,
    pub curve: Option<String>,
    pub time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationErrorCode {
    InvalidCurvePoint,
    BlindingReuse,
    RangeCheckOverflow,
    UnsupportedCurve,
    KeccakNotEnabled,
    PedersenNotEnabled,
    CurveNotAllowed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: ValidationErrorCode,
    pub msg: String,
    pub context: serde_json::Value,
}

impl ValidationError {
    pub fn new(
        code: ValidationErrorCode,
        msg: impl Into<String>,
        context: impl Into<serde_json::Value>,
    ) -> Self {
        Self {
            code,
            msg: msg.into(),
            context: context.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub msg: String,
    pub context: serde_json::Value,
}

impl ValidationWarning {
    pub fn new(code: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            msg: msg.into(),
            context: serde_json::Value::Null,
        }
    }

    pub fn with_context(
        code: impl Into<String>,
        msg: impl Into<String>,
        context: impl Into<serde_json::Value>,
    ) -> Self {
        Self {
            code: code.into(),
            msg: msg.into(),
            context: context.into(),
        }
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
        let meta = ReportMeta {
            backend_id: "native@0.0".to_string(),
            profile_id: "test".to_string(),
            hash_id: "abc123".to_string(),
            curve: Some("bls12-377".to_string()),
            time_ms: 42,
        };
        let report = ValidationReport::new_ok(meta);
        report
            .verify_manifest_hash("abc123")
            .expect("hash should match");
    }

    #[test]
    fn manifest_hash_verification_detects_mismatch() {
        let meta = ReportMeta {
            backend_id: "native@0.0".to_string(),
            profile_id: "test".to_string(),
            hash_id: "abc123".to_string(),
            curve: None,
            time_ms: 99,
        };
        let report = ValidationReport::new_ok(meta);
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

    #[test]
    fn serde_roundtrip() {
        let meta = ReportMeta {
            backend_id: "native@0.0".to_string(),
            profile_id: "profile-a".to_string(),
            hash_id: "deadbeef".to_string(),
            curve: Some("bls12-381".to_string()),
            time_ms: 1200,
        };
        let mut report = ValidationReport::new_ok(meta);
        report.push_warning(ValidationWarning::with_context(
            "Performance",
            "proof generation slower than baseline",
            serde_json::json!({"slowdown": 1.3}),
        ));
        report.push_error(ValidationError::new(
            ValidationErrorCode::RangeCheckOverflow,
            "range check failed",
            serde_json::json!({"witness": 5}),
        ));
        report.set_commit_passed(false);

        let serialized = report.to_json().expect("serialize report");
        let restored = ValidationReport::from_json(&serialized).expect("deserialize report");

        assert!(!restored.ok);
        assert!(!restored.commit_passed);
        assert_eq!(restored.errors.len(), 1);
        assert_eq!(restored.warnings.len(), 1);
        assert_eq!(restored.meta.backend_id, "native@0.0");
    }
}
