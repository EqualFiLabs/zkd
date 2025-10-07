//! AIR-IR: minimal, backend-neutral representation + TOML parser.

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
pub struct AirColumns {
    pub trace_cols: u32, // total trace columns
    #[serde(default)]
    pub const_cols: u32, // constant columns
    #[serde(default)]
    pub periodic_cols: u32, // periodic columns
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AirConstraints {
    /// Placeholder: number of transition constraints (to bound degree/shape).
    pub transition_count: u32,
    /// Placeholder: number of boundary constraints.
    pub boundary_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AirProgram {
    pub meta: AirMeta,
    pub columns: AirColumns,
    pub constraints: AirConstraints,
    /// Optional hint for expected row count (power of two). Used to derive TraceShape.
    #[serde(default)]
    pub rows_hint: Option<u32>,
}

impl AirProgram {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let s = fs::read_to_string(&path)
            .with_context(|| format!("reading AIR file {}", path.as_ref().display()))?;
        let prog: AirProgram = toml::from_str(&s)
            .with_context(|| format!("parsing AIR file {}", path.as_ref().display()))?;
        prog.validate()?;
        Ok(prog)
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
            if r < 8 || r > (1 << 22) {
                return Err(anyhow!("rows_hint out of range [2^3 .. 2^22]"));
            }
            if r.count_ones() != 1 {
                return Err(anyhow!("rows_hint must be a power of two"));
            }
        }
        Ok(())
    }
}
