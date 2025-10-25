use std::path::Path;

use anyhow::{Context, Result};

use super::types::AirIr;
use super::validate::validate_bindings;
use super::AirProgram;

pub fn parse_air_file(path: &Path) -> Result<AirIr> {
    let program = AirProgram::load_from_file(path)?;
    let ir = AirIr::from(program);
    validate_bindings(&ir)?;
    Ok(ir)
}

pub fn parse_air_str(src: &str) -> Result<AirIr> {
    let program: AirProgram = toml::from_str(src).context("parsing AIR source")?;
    program.validate()?;
    let ir = AirIr::from(program);
    validate_bindings(&ir)?;
    Ok(ir)
}
