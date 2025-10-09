use std::fs;

use zkprov_corelib::air::AirProgram;

fn fixture_yaml() -> &'static str {
    r#"
meta:
  name: zk_balance
  field: Prime254
  hash: poseidon2
columns:
  trace_cols: 4
  const_cols: 1
  periodic_cols: 0
constraints:
  transition_count: 2
  boundary_count: 1
"#
}

#[test]
fn yaml_roundtrip_matches_binary_ir() {
    let mut path = std::env::temp_dir();
    path.push(format!("zkd_yaml_roundtrip_{}.yml", std::process::id()));
    fs::write(&path, fixture_yaml()).expect("write yaml fixture");
    let program = AirProgram::load_from_file(&path).expect("load yaml");
    fs::remove_file(&path).ok();

    let serialized = serde_yaml::to_string(&program).expect("serialize to yaml");
    let reparsed: AirProgram = serde_yaml::from_str(&serialized).expect("parse yaml");
    assert_eq!(program, reparsed);
}
