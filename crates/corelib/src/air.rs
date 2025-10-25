//! AIR-IR: minimal, backend-neutral representation + TOML/YAML parser.

pub mod bindings;
pub mod parser;
mod parser_yaml;
pub mod types;
pub mod validate;

pub use parser::{parse_air_file, parse_air_str};
pub use types::{AirIr, CommitmentBinding};

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::air::types::{CommitmentBinding as IrCommitmentBinding, CommitmentKind, PublicTy};

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
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Default)]
pub struct AirCommitments {
    /// If true, program requires Pedersen (or compatible) commitment gadgets.
    pub pedersen: bool,
    /// Optional curve name hint (e.g., "placeholder", "bn254")
    pub curve: Option<String>,
    /// Parsed commitment bindings requested by the AIR.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<IrCommitmentBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct AirPublicInput {
    pub name: String,
    #[serde(default, rename = "type")]
    pub ty: PublicTy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyCommitments {
    #[serde(default)]
    pedersen: bool,
    #[serde(default)]
    curve: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CommitmentTable {
    #[serde(flatten)]
    entries: BTreeMap<String, CommitmentInline>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommitmentInline {
    #[serde(default)]
    curve: Option<String>,
    #[serde(default, rename = "public")]
    public_inputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommitmentListEntry {
    kind: String,
    #[serde(default)]
    curve: Option<String>,
    #[serde(default, rename = "public")]
    public_inputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CommitmentsWithBindings {
    #[serde(default)]
    pedersen: bool,
    #[serde(default)]
    curve: Option<String>,
    #[serde(default)]
    bindings: Vec<IrCommitmentBinding>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CommitmentsRaw {
    Legacy(LegacyCommitments),
    Table(CommitmentTable),
    List(Vec<CommitmentListEntry>),
    Full(CommitmentsWithBindings),
}

impl<'de> Deserialize<'de> for AirCommitments {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = CommitmentsRaw::deserialize(deserializer)?;
        build_commitments(raw).map_err(de::Error::custom)
    }
}

fn build_commitments(raw: CommitmentsRaw) -> Result<AirCommitments, String> {
    match raw {
        CommitmentsRaw::Legacy(legacy) => {
            let mut result = AirCommitments::default();
            result.pedersen = legacy.pedersen;
            result.curve = legacy.curve.clone();
            if legacy.pedersen {
                result.bindings.push(IrCommitmentBinding {
                    kind: CommitmentKind::Pedersen {
                        curve: legacy.curve.unwrap_or_default(),
                    },
                    public_inputs: Vec::new(),
                });
            }
            Ok(result)
        }
        CommitmentsRaw::Table(table) => {
            let mut result = AirCommitments::default();
            for (name, entry) in table.entries {
                let binding = build_table_binding(&name, entry)?;
                if matches!(binding.kind, CommitmentKind::Pedersen { .. }) {
                    result.pedersen = true;
                    if result.curve.is_none() {
                        if let CommitmentKind::Pedersen { curve } = &binding.kind {
                            if !curve.is_empty() {
                                result.curve = Some(curve.clone());
                            }
                        }
                    }
                }
                result.bindings.push(binding);
            }
            Ok(result)
        }
        CommitmentsRaw::List(list) => {
            let mut result = AirCommitments::default();
            for entry in list {
                let binding = build_list_binding(&entry)?;
                if matches!(binding.kind, CommitmentKind::Pedersen { .. }) {
                    result.pedersen = true;
                    if result.curve.is_none() {
                        if let CommitmentKind::Pedersen { curve } = &binding.kind {
                            if !curve.is_empty() {
                                result.curve = Some(curve.clone());
                            }
                        }
                    }
                }
                result.bindings.push(binding);
            }
            Ok(result)
        }
        CommitmentsRaw::Full(full) => {
            let mut result = AirCommitments {
                pedersen: full.pedersen,
                curve: full.curve,
                bindings: full.bindings,
            };
            if result.pedersen && result.curve.is_none() {
                if let Some(curve) =
                    result
                        .bindings
                        .iter()
                        .find_map(|binding| match &binding.kind {
                            CommitmentKind::Pedersen { curve } if !curve.is_empty() => {
                                Some(curve.clone())
                            }
                            _ => None,
                        })
                {
                    result.curve = Some(curve);
                }
            }
            Ok(result)
        }
    }
}

fn build_table_binding(name: &str, entry: CommitmentInline) -> Result<IrCommitmentBinding, String> {
    let public_inputs = entry.public_inputs;
    match name {
        "pedersen" => {
            let curve = entry.curve.unwrap_or_default();
            Ok(IrCommitmentBinding {
                kind: CommitmentKind::Pedersen { curve },
                public_inputs,
            })
        }
        "poseidon_commit" => {
            if entry.curve.is_some() {
                return Err("CommitmentBindingUnexpectedCurve".to_string());
            }
            Ok(IrCommitmentBinding {
                kind: CommitmentKind::PoseidonCommit,
                public_inputs,
            })
        }
        "keccak_commit" => {
            if entry.curve.is_some() {
                return Err("CommitmentBindingUnexpectedCurve".to_string());
            }
            Ok(IrCommitmentBinding {
                kind: CommitmentKind::KeccakCommit,
                public_inputs,
            })
        }
        other => Err(format!("unknown commitment kind '{}'", other)),
    }
}

fn build_list_binding(entry: &CommitmentListEntry) -> Result<IrCommitmentBinding, String> {
    let kind_key = normalize_kind(&entry.kind);
    let public_inputs = entry.public_inputs.clone();
    match kind_key.as_str() {
        "pedersen" => Ok(IrCommitmentBinding {
            kind: CommitmentKind::Pedersen {
                curve: entry.curve.clone().unwrap_or_default(),
            },
            public_inputs,
        }),
        "poseidoncommit" => {
            if entry.curve.is_some() {
                return Err("CommitmentBindingUnexpectedCurve".to_string());
            }
            Ok(IrCommitmentBinding {
                kind: CommitmentKind::PoseidonCommit,
                public_inputs,
            })
        }
        "keccakcommit" => {
            if entry.curve.is_some() {
                return Err("CommitmentBindingUnexpectedCurve".to_string());
            }
            Ok(IrCommitmentBinding {
                kind: CommitmentKind::KeccakCommit,
                public_inputs,
            })
        }
        other => Err(format!("unknown commitment kind '{}'", other)),
    }
}

fn normalize_kind(kind: &str) -> String {
    kind.chars()
        .filter(|c| *c != '_')
        .flat_map(|c| c.to_lowercase())
        .collect()
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
    /// Declared public inputs available for bindings.
    #[serde(default)]
    pub public_inputs: Vec<AirPublicInput>,
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
