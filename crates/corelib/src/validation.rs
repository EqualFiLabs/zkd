use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::zkprov_bundles::{BlindingTracker, PedersenCtx, PrivacyError, RangeCheck};
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

    pub fn write_pretty<P: AsRef<std::path::Path>>(
        &self,
        dir: P,
    ) -> std::io::Result<std::path::PathBuf> {
        std::fs::create_dir_all(&dir)?;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();
        let fname = format!(
            "validation_{}_{}_{}_{}.json",
            Self::sanitize_component(&self.meta.backend_id),
            Self::sanitize_component(&self.meta.profile_id),
            Self::sanitize_component(&self.meta.hash_id),
            ts
        );
        let path = dir.as_ref().join(fname);
        std::fs::write(&path, serde_json::to_string_pretty(self).unwrap())?;
        Ok(path)
    }

    fn sanitize_component(value: &str) -> String {
        value
            .chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '.' | '_' | '-' => c,
                _ => '_',
            })
            .collect()
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

#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub pedersen_enabled: bool,
    pub allowed_curves: Vec<String>,
    pub keccak_enabled: bool,
    pub no_r_reuse: bool,
    requested_curve: Option<String>,
    requested_hash: Option<String>,
    pedersen_required: bool,
}

impl ValidationConfig {
    fn from_bindings(b: &crate::air::bindings::Bindings) -> Self {
        let requested_curve = b.commitments.curve.clone();
        let requested_hash = b.hash_id_for_commitments.clone();
        let pedersen_required = b.commitments.pedersen;

        let mut allowed_curves = Vec::new();
        if let Some(curve) = &requested_curve {
            allowed_curves.push(curve.clone());
        }

        Self {
            pedersen_enabled: pedersen_required,
            allowed_curves,
            keccak_enabled: true,
            no_r_reuse: b.commitments.no_r_reuse.unwrap_or(false),
            requested_curve,
            requested_hash,
            pedersen_required,
        }
    }

    fn requested_curve(&self) -> Option<&str> {
        self.requested_curve.as_deref()
    }

    fn requested_hash(&self) -> Option<&str> {
        self.requested_hash.as_deref()
    }

    fn pedersen_required(&self) -> bool {
        self.pedersen_required
    }

    fn keccak_requested(&self) -> bool {
        matches!(
            self.requested_hash(),
            Some(hash) if matches_ignore_ascii_case(hash, "keccak")
                || matches_ignore_ascii_case(hash, "keccak256")
        )
    }
}

fn matches_ignore_ascii_case(value: &str, expected: &str) -> bool {
    value.eq_ignore_ascii_case(expected)
}

pub struct Validator<'a> {
    cfg: ValidationConfig,
    ped: Option<PedersenCtx>,
    blinds: BlindingTracker,
    report: ValidationReport,
    clock: Instant,
    _pd: PhantomData<&'a ()>,
}

impl<'a> Validator<'a> {
    pub fn new(b: &crate::air::bindings::Bindings) -> Self {
        let cfg = ValidationConfig::from_bindings(b);
        let meta = ReportMeta {
            backend_id: String::new(),
            profile_id: String::new(),
            hash_id: cfg.requested_hash().unwrap_or("blake3").to_string(),
            curve: cfg.requested_curve().map(|c| c.to_string()),
            time_ms: 0,
        };
        let report = ValidationReport::new_ok(meta);
        let (ped, init_error) = if cfg.pedersen_required() {
            match PedersenCtx::from_bindings(b) {
                Ok(ctx) => (Some(ctx), None),
                Err(err) => (None, Some(err)),
            }
        } else {
            (None, None)
        };

        let mut validator = Self {
            cfg,
            ped,
            blinds: BlindingTracker::new(),
            report,
            clock: Instant::now(),
            _pd: PhantomData,
        };

        if let Some(err) = init_error {
            validator.push_privacy_error(err, serde_json::json!({"operation": "init"}));
        }

        validator
    }

    pub fn config_mut(&mut self) -> &mut ValidationConfig {
        &mut self.cfg
    }

    pub fn check_commit_point(&mut self, msg: &[u8], r: &[u8]) {
        if !self.cfg.pedersen_enabled {
            self.report.push_error(ValidationError::new(
                ValidationErrorCode::PedersenNotEnabled,
                "pedersen commitments disabled by configuration",
                serde_json::json!({"operation": "check_commit_point"}),
            ));
            return;
        }

        if let Some(curve) = self.cfg.requested_curve() {
            if !self.cfg.allowed_curves.is_empty()
                && !self
                    .cfg
                    .allowed_curves
                    .iter()
                    .any(|allowed| matches_ignore_ascii_case(allowed, curve))
            {
                self.report.push_error(ValidationError::new(
                    ValidationErrorCode::CurveNotAllowed,
                    "curve not allowed by configuration",
                    serde_json::json!({
                        "operation": "check_commit_point",
                        "curve": curve,
                    }),
                ));
                return;
            }
        }

        if self.cfg.keccak_requested() && !self.cfg.keccak_enabled {
            self.report.push_error(ValidationError::new(
                ValidationErrorCode::KeccakNotEnabled,
                "keccak commitments disabled by configuration",
                serde_json::json!({
                    "operation": "check_commit_point",
                    "hash": self.cfg.requested_hash(),
                }),
            ));
            return;
        }

        let Some(ctx) = self.ped.as_ref() else {
            return;
        };

        match ctx.commit(&mut self.blinds, msg, r) {
            Ok(commit) => {
                let (cx, cy) = commit.as_tuple();
                if let Err(err) = ctx.open(msg, r, cx, cy) {
                    self.push_privacy_error(
                        err,
                        serde_json::json!({"operation": "check_commit_point"}),
                    );
                }
            }
            Err(err) => {
                self.push_privacy_error(
                    err,
                    serde_json::json!({"operation": "check_commit_point"}),
                );
            }
        }
    }

    pub fn check_commit_point_with_pair(
        &mut self,
        msg: &[u8],
        r: &[u8],
        cx: &[u8; 32],
        cy: &[u8; 32],
    ) {
        if !self.cfg.pedersen_enabled {
            self.report.push_error(ValidationError::new(
                ValidationErrorCode::PedersenNotEnabled,
                "pedersen commitments disabled by configuration",
                serde_json::json!({"operation": "check_commit_point"}),
            ));
            return;
        }

        if let Some(curve) = self.cfg.requested_curve() {
            if !self.cfg.allowed_curves.is_empty()
                && !self
                    .cfg
                    .allowed_curves
                    .iter()
                    .any(|allowed| matches_ignore_ascii_case(allowed, curve))
            {
                self.report.push_error(ValidationError::new(
                    ValidationErrorCode::CurveNotAllowed,
                    "curve not allowed by configuration",
                    serde_json::json!({
                        "operation": "check_commit_point",
                        "curve": curve,
                    }),
                ));
                return;
            }
        }

        if self.cfg.keccak_requested() && !self.cfg.keccak_enabled {
            self.report.push_error(ValidationError::new(
                ValidationErrorCode::KeccakNotEnabled,
                "keccak commitments disabled by configuration",
                serde_json::json!({
                    "operation": "check_commit_point",
                    "hash": self.cfg.requested_hash(),
                }),
            ));
            return;
        }

        let Some(ctx) = self.ped.as_ref() else {
            return;
        };

        if let Err(err) = ctx.open(msg, r, cx, cy) {
            self.push_privacy_error(err, serde_json::json!({"operation": "check_commit_point"}));
        }
    }

    pub fn check_r_reuse(&mut self, r: &[u8]) {
        if !self.cfg.pedersen_enabled {
            self.report.push_error(ValidationError::new(
                ValidationErrorCode::PedersenNotEnabled,
                "pedersen commitments disabled by configuration",
                serde_json::json!({"operation": "check_r_reuse"}),
            ));
            return;
        }

        let Some(ctx) = self.ped.as_ref() else {
            return;
        };

        if let Err(err) = self.blinds.note_and_check(r, ctx.no_reuse()) {
            self.push_privacy_error(err, serde_json::json!({"operation": "check_r_reuse"}));
        }
    }

    pub fn check_range_u64(&mut self, v: u64, k: u32) {
        if let Err(err) = RangeCheck::check_u64(v, k) {
            self.push_privacy_error(
                err,
                serde_json::json!({
                    "operation": "check_range_u64",
                    "value": v,
                    "bits": k,
                }),
            );
        }
    }

    pub fn finalize(mut self) -> ValidationReport {
        let elapsed = self.clock.elapsed().as_millis() as u64;
        self.report.meta.time_ms = elapsed;

        const COMMIT_ERROR_CODES: &[ValidationErrorCode] = &[
            ValidationErrorCode::InvalidCurvePoint,
            ValidationErrorCode::BlindingReuse,
            ValidationErrorCode::RangeCheckOverflow,
            ValidationErrorCode::CurveNotAllowed,
            ValidationErrorCode::PedersenNotEnabled,
            ValidationErrorCode::KeccakNotEnabled,
        ];

        let commit_passed = !self
            .report
            .errors
            .iter()
            .any(|err| COMMIT_ERROR_CODES.contains(&err.code));
        self.report.set_commit_passed(commit_passed);
        self.report
    }

    fn push_privacy_error(&mut self, err: PrivacyError, context: serde_json::Value) {
        let code = Self::map_privacy_error(&err);
        self.report
            .push_error(ValidationError::new(code, err.to_string(), context));
    }

    fn map_privacy_error(err: &PrivacyError) -> ValidationErrorCode {
        match err {
            PrivacyError::InvalidCurvePoint => ValidationErrorCode::InvalidCurvePoint,
            PrivacyError::BlindingReuse => ValidationErrorCode::BlindingReuse,
            PrivacyError::RangeCheckOverflow => ValidationErrorCode::RangeCheckOverflow,
            PrivacyError::UnsupportedCurve => ValidationErrorCode::CurveNotAllowed,
            PrivacyError::Internal(_) => ValidationErrorCode::UnsupportedCurve,
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
    use crate::air::bindings::{Bindings, CommitmentsPolicy};
    use crate::zkprov_bundles::PrivacyError;
    use serde_json::Value;
    use std::fs;
    use tempfile::tempdir;

    fn bindings_with_pedersen() -> Bindings {
        Bindings {
            commitments: CommitmentsPolicy {
                pedersen: true,
                curve: Some("placeholder".to_string()),
                no_r_reuse: Some(false),
            },
            hash_id_for_commitments: Some("blake3".to_string()),
        }
    }

    #[test]
    fn privacy_error_mapping_matches_codes() {
        assert_eq!(
            Validator::map_privacy_error(&PrivacyError::InvalidCurvePoint),
            ValidationErrorCode::InvalidCurvePoint
        );
        assert_eq!(
            Validator::map_privacy_error(&PrivacyError::BlindingReuse),
            ValidationErrorCode::BlindingReuse
        );
        assert_eq!(
            Validator::map_privacy_error(&PrivacyError::RangeCheckOverflow),
            ValidationErrorCode::RangeCheckOverflow
        );
        assert_eq!(
            Validator::map_privacy_error(&PrivacyError::UnsupportedCurve),
            ValidationErrorCode::CurveNotAllowed
        );
        assert_eq!(
            Validator::map_privacy_error(&PrivacyError::Internal("oops".into())),
            ValidationErrorCode::UnsupportedCurve
        );
    }

    #[test]
    fn pedersen_disabled_records_error() {
        let bindings = bindings_with_pedersen();
        let mut validator = Validator::new(&bindings);
        validator.config_mut().pedersen_enabled = false;
        validator.check_commit_point(b"msg", b"r");
        assert_eq!(validator.report.errors.len(), 1);
        assert_eq!(
            validator.report.errors[0].code,
            ValidationErrorCode::PedersenNotEnabled
        );
    }

    #[test]
    fn blinding_reuse_detected() {
        let mut bindings = bindings_with_pedersen();
        bindings.commitments.no_r_reuse = Some(true);
        let mut validator = Validator::new(&bindings);
        validator.check_r_reuse(b"r1");
        validator.check_r_reuse(b"r1");
        assert!(validator
            .report
            .errors
            .iter()
            .any(|e| e.code == ValidationErrorCode::BlindingReuse));
    }

    #[test]
    fn range_check_overflow_detected() {
        let bindings = bindings_with_pedersen();
        let mut validator = Validator::new(&bindings);
        validator.check_range_u64(16, 4);
        assert!(validator
            .report
            .errors
            .iter()
            .any(|e| e.code == ValidationErrorCode::RangeCheckOverflow));
    }

    #[test]
    fn keccak_disabled_emits_error() {
        let mut bindings = bindings_with_pedersen();
        bindings.hash_id_for_commitments = Some("keccak256".to_string());
        let mut validator = Validator::new(&bindings);
        validator.config_mut().keccak_enabled = false;
        validator.check_commit_point(b"msg", b"r");
        assert!(validator
            .report
            .errors
            .iter()
            .any(|e| e.code == ValidationErrorCode::KeccakNotEnabled));
    }

    #[test]
    fn curve_not_allowed_emits_error() {
        let bindings = bindings_with_pedersen();
        let mut validator = Validator::new(&bindings);
        validator.config_mut().allowed_curves = vec!["bls12-381".to_string()];
        validator.check_commit_point(b"msg", b"r");
        assert!(validator
            .report
            .errors
            .iter()
            .any(|e| e.code == ValidationErrorCode::CurveNotAllowed));
    }

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
    fn write_pretty_persists_report() {
        let report = ValidationReport {
            ok: true,
            commit_passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            meta: ReportMeta {
                backend_id: "backend with spaces".into(),
                profile_id: "profile/@#".into(),
                hash_id: "hash$%^".into(),
                curve: Some("curve25519".into()),
                time_ms: 42,
            },
        };

        let temp = tempdir().unwrap();
        let path = report.write_pretty(temp.path()).unwrap();
        assert!(path.exists());

        let contents = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&contents).unwrap();
        let expected = serde_json::to_value(&report).unwrap();
        assert_eq!(parsed, expected);

        let filename = path.file_name().unwrap().to_string_lossy();
        assert!(filename.starts_with("validation_"));
        assert!(filename.ends_with(".json"));
        assert!(filename
            .chars()
            .all(|c| matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '.' | '_' | '-')));
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
