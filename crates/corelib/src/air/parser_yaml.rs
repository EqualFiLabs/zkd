use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use super::AirProgram;

pub fn load_from_str(input: &str) -> Result<AirProgram> {
    let program: AirProgram = serde_yaml::from_str(input).context("parsing AIR YAML")?;
    program.validate()?;
    Ok(program)
}

pub fn load_from_file(path: &Path) -> Result<AirProgram> {
    let contents =
        fs::read_to_string(path).with_context(|| format!("reading AIR YAML {}", path.display()))?;
    load_from_str(&contents)
}
