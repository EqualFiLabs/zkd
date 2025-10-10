//! AIR-IR: minimal, backend-neutral representation + TOML/YAML parser.

pub mod bindings;
mod parser_yaml;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Hash function enum (narrow for now; we’ll extend later)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AirHash {
    Poseidon2,
    Blake3,
    Rescue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AirMeta {
    pub name: String,  // program name (slug)
    pub field: String, // "Goldilocks", "Prime254", etc.
    pub hash: AirHash, // transcript / commitments
    #[serde(default)]
    pub backend: Option<String>, // optional preferred backend id (e.g., "winterfell@0.6")
    #[serde(default)]
    pub profile: Option<String>, // optional suggested profile id
    #[serde(default)]
    pub degree_hint: Option<u32>, // optional upper bound on transition degree
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AirColumns {
    pub trace_cols: u32, // total trace columns
    #[serde(default)]
    pub const_cols: u32, // constant columns
    #[serde(default)]
    pub periodic_cols: u32, // periodic columns
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AirConstraints {
    /// Placeholder: number of transition constraints (to bound degree/shape).
    pub transition_count: u32,
    /// Placeholder: number of boundary constraints.
    pub boundary_count: u32,
}

/// Optional commitments requirements (Phase-0 validation surface)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AirCommitments {
    /// If true, program requires Pedersen (or compatible) commitment gadgets.
    #[serde(default)]
    pub pedersen: bool,
    /// Optional curve name hint (e.g., "placeholder", "bn254")
    #[serde(default)]
    pub curve: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AirProgram {
    pub meta: AirMeta,
    pub columns: AirColumns,
    pub constraints: AirConstraints,
    /// Optional hint for expected row count (power of two). Used to derive TraceShape.
    #[serde(default)]
    pub rows_hint: Option<u32>,
    /// Optional commitments requirements (pedersen/curve hints)
    #[serde(default)]
    pub commitments: Option<AirCommitments>,
}

impl AirProgram {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let ext = path_ref
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        let program = match ext.as_str() {
            "yaml" | "yml" => parser_yaml::load_from_file(path_ref)?,
            _ => {
                let s = fs::read_to_string(path_ref)
                    .with_context(|| format!("reading AIR file {}", path_ref.display()))?;
                let prog: AirProgram = toml::from_str(&s)
                    .with_context(|| format!("parsing AIR file {}", path_ref.display()))?;
                prog.validate()?;
                prog
            }
        };

        Ok(program)
    }

    pub fn validate(&self) -> Result<()> {
        // name: alnum, underscore, dash only; 2..64 chars
        let re = Regex::new(r"^[A-Za-z0-9_\-]{2,64}$").unwrap();
        if !re.is_match(&self.meta.name) {
            return Err(anyhow!("invalid meta.name '{}'", self.meta.name));
        }
        // field basic sanity (we’ll cross-check with backend caps elsewhere)
        if self.meta.field.trim().is_empty() {
            return Err(anyhow!("meta.field cannot be empty"));
        }
        // trace columns sanity
        if self.columns.trace_cols == 0 {
            return Err(anyhow!("columns.trace_cols must be > 0"));
        }
        if self.columns.trace_cols > 2048 {
            return Err(anyhow!(
                "columns.trace_cols too large (>2048) for default limits"
            ));
        }
        // constraints count sanity
        if self.constraints.transition_count == 0 {
            return Err(anyhow!("constraints.transition_count must be > 0"));
        }
        // degree hint sanity
        if let Some(d) = self.meta.degree_hint {
            if d == 0 || d > 64 {
                return Err(anyhow!("degree_hint out of range (1..=64)"));
            }
        }
        // rows_hint sanity (power of two)
        if let Some(r) = self.rows_hint {
            if !(8u32..=(1u32 << 22)).contains(&r) {
                return Err(anyhow!("rows_hint out of range [2^3 .. 2^22]"));
            }
            if r.count_ones() != 1 {
                return Err(anyhow!("rows_hint must be a power of two"));
            }
        }
        if let Some(c) = &self.commitments {
            if let Some(curve) = &c.curve {
                if curve.trim().is_empty() {
                    return Err(anyhow!("commitments.curve cannot be empty when present"));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_yaml() -> &'static str {
        r#"
meta:
  name: toy_balance
  field: Prime254
  hash: poseidon2
columns:
  trace_cols: 8
  const_cols: 2
  periodic_cols: 1
constraints:
  transition_count: 4
  boundary_count: 2
rows_hint: 16
commitments:
  pedersen: true
  curve: bn254
"#
    }

    #[test]
    fn yaml_roundtrip_matches_self() {
        let parsed = parser_yaml::load_from_str(sample_yaml()).expect("yaml parse");
        let serialized = serde_yaml::to_string(&parsed).expect("serialize yaml");
        let reparsed = parser_yaml::load_from_str(&serialized).expect("reparse yaml");
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn load_from_file_handles_yaml_extension() {
        let tmp_path = {
            let mut path = std::env::temp_dir();
            path.push(format!("zkd_yaml_test_{}.yml", std::process::id()));
            path
        };
        fs::write(&tmp_path, sample_yaml()).expect("write yaml file");
        let loaded = AirProgram::load_from_file(&tmp_path).expect("load yaml file");
        fs::remove_file(&tmp_path).ok();
        let expected = parser_yaml::load_from_str(sample_yaml()).expect("parse baseline");
        assert_eq!(loaded, expected);
    }
}
