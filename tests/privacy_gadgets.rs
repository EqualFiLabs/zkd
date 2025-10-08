use zkprov_bundles::{
    AddUnderCommit, BlindingTracker, PedersenCommit, PedersenCtx, PrivacyError, RangeCheck,
};
use zkprov_corelib::air::AirProgram;
use zkprov_corelib::air_bindings::Bindings;

fn toy_air_text(no_r_reuse: bool) -> String {
    let _ = no_r_reuse;
    r#"
        [meta]
        name = "gadgets_demo"
        field = "Prime254"
        hash = "blake3"
        [columns]
        trace_cols = 2
        [constraints]
        transition_count = 1
        boundary_count = 1
        [commitments]
        pedersen = true
        curve = "placeholder"
        # (no_r_reuse policy surfaced in Bindings; air file defaults false)
    "#
    .to_string()
}

fn ctx_and_tracker() -> (PedersenCtx, BlindingTracker) {
    let air: AirProgram = toml::from_str(&toy_air_text(false)).unwrap();
    air.validate().unwrap();
    let b = Bindings::from_air(&air);
    let ctx = PedersenCtx::from_bindings(&b).unwrap();
    (ctx, BlindingTracker::new())
}

#[test]
fn positive_pedersen_commit_open() {
    let (ctx, mut tracker) = ctx_and_tracker();
    let msg = b"hello";
    let r = b"r-1";
    let PedersenCommit { cx, cy } = ctx.commit(&mut tracker, msg, r).unwrap();
    let ok = ctx.open(msg, r, &cx, &cy).unwrap();
    assert!(ok);
}

#[test]
fn negative_invalid_curve_point_when_mismatch() {
    let (ctx, mut tracker) = ctx_and_tracker();
    let msg = b"hello";
    let r = b"r-1";
    let PedersenCommit { cx, cy } = ctx.commit(&mut tracker, msg, r).unwrap();

    // Tamper with point:
    let mut cy_bad = cy;
    cy_bad[0] ^= 0xFF;

    let err = ctx.open(msg, r, &cx, &cy_bad).unwrap_err();
    assert_eq!(err, PrivacyError::InvalidCurvePoint);
}

#[test]
fn positive_range_check() {
    RangeCheck::check_u64(15, 4).unwrap();
    RangeCheck::check_u64(u64::MAX, 64).unwrap();
}

#[test]
fn negative_range_overflow() {
    let err = RangeCheck::check_u64(16, 4).unwrap_err();
    assert_eq!(err, PrivacyError::RangeCheckOverflow);
}

#[test]
fn positive_add_under_commit() {
    let (ctx, mut tracker) = ctx_and_tracker();
    let m1 = b"7";
    let m2 = b"9";
    let r1 = b"r1";
    let r2 = b"r2";
    let (_csum, _r12) = AddUnderCommit::run(&ctx, &mut tracker, m1, r1, m2, r2).unwrap();
}

#[test]
fn negative_blinding_reuse_policy() {
    // Emulate a program that forbids reuse by flipping policy in bindings.
    let air: AirProgram = toml::from_str(&toy_air_text(false)).unwrap();
    let mut b = Bindings::from_air(&air);
    b.commitments.no_r_reuse = Some(true);

    let ctx = PedersenCtx::from_bindings(&b).unwrap();
    let mut tracker = BlindingTracker::new();

    let _ = ctx.commit(&mut tracker, b"A", b"R").unwrap();
    // Reuse same R with same or different msg should error when policy forbids reuse.
    let err = ctx.commit(&mut tracker, b"B", b"R").unwrap_err();
    assert_eq!(err, PrivacyError::BlindingReuse);
}
