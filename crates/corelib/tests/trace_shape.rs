use zkprov_corelib::air::AirProgram;
use zkprov_corelib::trace::TraceShape;

const TOY_AIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/air/toy.air");

#[test]
fn derive_shape_from_air() {
    let air = AirProgram::load_from_file(TOY_AIR).unwrap();
    let shape = TraceShape::from_air(&air);
    assert_eq!(shape.rows, 65536);
    assert_eq!(shape.cols, 4);
    assert_eq!(shape.const_cols, 1);
    assert_eq!(shape.periodic_cols, 1);
}

#[test]
fn default_rows_when_missing_hint() {
    // same AIR but with hint omitted inline:
    let toml_text = r#"
        [meta]
        name = "toy_nohint"
        field = "Prime254"
        hash = "blake3"

        [columns]
        trace_cols = 2

        [constraints]
        transition_count = 1
        boundary_count = 1
    "#;
    let air: AirProgram = toml::from_str(toml_text).unwrap();
    air.validate().unwrap();
    let shape = TraceShape::from_air(&air);
    assert_eq!(shape.rows, 1 << 16);
}
