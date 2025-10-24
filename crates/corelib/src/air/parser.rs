use std::path::Path;

use anyhow::{Context, Result};

use super::types::AirIr;
use super::AirProgram;

pub fn parse_air_file(path: &Path) -> Result<AirIr> {
    let program = AirProgram::load_from_file(path)?;
    Ok(AirIr::from(program))
}

pub fn parse_air_str(src: &str) -> Result<AirIr> {
    let program: AirProgram = toml::from_str(src).context("parsing AIR source")?;
    program.validate()?;
    Ok(AirIr::from(program))
}
