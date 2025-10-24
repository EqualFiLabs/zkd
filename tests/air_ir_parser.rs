use std::path::Path;

use zkprov_corelib::air::{parse_air_file, parse_air_str, AirIr};

#[test]
#[ignore = "todo"]
fn parse_air_ir_from_str() {
    let _ = parse_air_str as fn(&str) -> anyhow::Result<AirIr>;
    todo!();
}

#[test]
#[ignore = "todo"]
fn parse_air_ir_from_file() {
    let _ = parse_air_file as fn(&Path) -> anyhow::Result<AirIr>;
    todo!();
}
