use zkprov_corelib::{
    air::AirProgram,
    air_bindings::Bindings,
    validation::{ReportMeta, ValidationErrorCode, ValidationReport, Validator},
    zkprov_bundles::{BlindingTracker, PedersenCtx},
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

#[test]
fn invalid_curve_point_sets_code() {
    let air: AirProgram = toml::from_str(TOY_AIR).expect("toy AIR parses");
    air.validate().expect("toy AIR validates");

    let mut bindings = Bindings::from_air(&air);
    bindings.commitments.no_r_reuse = Some(false);

    let ped_ctx = PedersenCtx::from_bindings(&bindings).expect("pedersen ctx");
    let mut tracker = BlindingTracker::new();
    let commit = ped_ctx
        .commit(&mut tracker, b"hello", b"blind")
        .expect("commit succeeds");
    let (cx, cy) = commit.as_tuple();
    let mut bad_cy = *cy;
    bad_cy[0] ^= 1;

    let mut validator = Validator::new(&bindings);
    validator.check_commit_point_with_pair(b"hello", b"blind", cx, &bad_cy);

    let report = validator.finalize();
    assert!(!report.ok);
    assert!(!report.commit_passed);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(
        report.errors[0].code,
        ValidationErrorCode::InvalidCurvePoint
    );
}

#[test]
fn blinding_reuse_sets_code() {
    let air: AirProgram = toml::from_str(TOY_AIR).expect("toy AIR parses");
    air.validate().expect("toy AIR validates");

    let mut bindings = Bindings::from_air(&air);
    bindings.commitments.no_r_reuse = Some(true);

    let mut validator = Validator::new(&bindings);
    validator.check_r_reuse(b"reuse-me");
    validator.check_r_reuse(b"reuse-me");

    let report = validator.finalize();
    assert!(!report.ok);
    assert!(!report.commit_passed);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.errors[0].code, ValidationErrorCode::BlindingReuse);
}

#[test]
fn range_overflow_sets_code() {
    let air: AirProgram = toml::from_str(TOY_AIR).expect("toy AIR parses");
    air.validate().expect("toy AIR validates");

    let bindings = Bindings::from_air(&air);

    let mut validator = Validator::new(&bindings);
    validator.check_range_u64(16, 4);

    let report = validator.finalize();
    assert!(!report.ok);
    assert!(!report.commit_passed);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(
        report.errors[0].code,
        ValidationErrorCode::RangeCheckOverflow
    );
}

#[test]
fn curve_not_allowed_sets_code() {
    let air: AirProgram = toml::from_str(TOY_AIR).expect("toy AIR parses");
    air.validate().expect("toy AIR validates");

    let mut bindings = Bindings::from_air(&air);
    bindings.commitments.curve = Some("placeholder".to_string());

    let mut validator = Validator::new(&bindings);
    validator.config_mut().allowed_curves = vec!["other-curve".to_string()];
    validator.check_commit_point(b"hello", b"blind");

    let report = validator.finalize();
    assert!(!report.ok);
    assert!(!report.commit_passed);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.errors[0].code, ValidationErrorCode::CurveNotAllowed);
}

#[test]
fn pedersen_disabled_sets_code() {
    let air: AirProgram = toml::from_str(TOY_AIR).expect("toy AIR parses");
    air.validate().expect("toy AIR validates");

    let bindings = Bindings::from_air(&air);

    let mut validator = Validator::new(&bindings);
    validator.config_mut().pedersen_enabled = false;
    validator.check_commit_point(b"hello", b"blind");

    let report = validator.finalize();
    assert!(!report.ok);
    assert!(!report.commit_passed);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(
        report.errors[0].code,
        ValidationErrorCode::PedersenNotEnabled
    );
}
