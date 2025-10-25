use zkprov_corelib::{
    air::AirProgram,
    air_bindings::Bindings,
    validation::{ReportMeta, ValidationErrorCode, ValidationReport, Validator},
};

const TOY_AIR: &str = include_str!("../examples/air/toy.air");

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
    assert_eq!(
        report.errors[0].code,
        ValidationErrorCode::InvalidCurvePoint
    );
    assert_eq!(report.errors[0].msg, "commitment rejected");
    assert_eq!(report.errors[0].context["witness"], 12);
    assert!(report.warnings.is_empty());
}

#[test]
fn positive_commit_validation() {
    let air: AirProgram = toml::from_str(TOY_AIR).expect("toy AIR parses");
    air.validate().expect("toy AIR validates");

    let mut bindings = Bindings::from_air(&air);
    bindings.commitments.no_r_reuse = Some(false);

    let mut validator = Validator::new(&bindings);
    validator.check_commit_point(b"hello", b"r1");
    validator.check_range_u64(15, 4);
    validator.check_r_reuse(b"r2");

    let report = validator.finalize();
    assert!(report.ok);
    assert!(report.commit_passed);
    assert!(report.errors.is_empty());
}
