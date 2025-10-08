use zkprov_corelib::profile::{load_all_profiles, Profile};

#[test]
fn parse_three_default_profiles() {
    let all = load_all_profiles().expect("profiles load");
    assert!(all.iter().any(|p| p.id == "dev-fast"));
    assert!(all.iter().any(|p| p.id == "balanced"));
    assert!(all.iter().any(|p| p.id == "secure"));
}

#[test]
fn validation_enforces_bounds() {
    let bad = Profile {
        id: "".to_string(),
        lambda_bits: 32,
        fri_blowup: Some(1),
        fri_queries: Some(8),
        grind_bits: Some(100),
        merkle_arity: Some(7),
        const_col_limit: None,
        rows_max: None,
    };
    assert!(bad.validate().is_err());
}
