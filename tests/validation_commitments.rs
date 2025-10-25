use zkprov_corelib::validation::{
    ReportMeta, ValidationErrorCode, ValidationReport,
};

#[test]
fn commitment_failure_sets_flags() {
    let meta = ReportMeta {
        backend_id: "native@0.0".to_string(),
        profile_id: "profile-a".to_string(),
        hash_id: "feedface".to_string(),
        curve: Some("bls12-377".to_string()),
        time_ms: 32,
    };

    let report = ValidationReport::fail(
        meta,
        ValidationErrorCode::InvalidCurvePoint,
        "commitment rejected",
        serde_json::json!({"witness": 12}),
    );

    assert!(!report.ok);
    assert!(!report.commit_passed);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.errors[0].code, ValidationErrorCode::InvalidCurvePoint);
    assert_eq!(report.errors[0].msg, "commitment rejected");
    assert_eq!(report.errors[0].context["witness"], 12);
    assert!(report.warnings.is_empty());
}
