use zkprov_corelib::air::AirProgram;

const TOY_AIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/air/toy.air");
const TOY_AIR_SRC: &str = include_str!("../../../examples/air/toy.air");

#[test]
fn parse_and_validate_air() {
    let air = AirProgram::load_from_file(TOY_AIR).expect("load");
    assert_eq!(air.meta.name, "toy_merkle");
    assert_eq!(air.columns.trace_cols, 4);
    assert_eq!(air.constraints.transition_count, 3);
    assert_eq!(air.rows_hint, Some(65536));

    let inline: AirProgram = toml::from_str(TOY_AIR_SRC).unwrap();
    assert_eq!(inline.rows_hint, Some(65536));
}
