use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub id: String,
    pub lambda_bits: u32,
    #[serde(default)]
    pub fri_blowup: Option<u32>,
    #[serde(default)]
    pub fri_queries: Option<u32>,
    #[serde(default)]
    pub grind_bits: Option<u32>,
    #[serde(default)]
    pub merkle_arity: Option<u32>,
    #[serde(default)]
    pub const_col_limit: Option<u32>,
    #[serde(default)]
    pub rows_max: Option<u32>,
}

impl Profile {
    pub fn validate(&self) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(anyhow!("profile id cannot be empty"));
        }
        if !(64..=256).contains(&self.lambda_bits) {
            return Err(anyhow!(
                "lambda_bits {} out of allowed range [64..256]",
                self.lambda_bits
            ));
        }
        if let Some(arity) = self.merkle_arity {
            if ![2u32, 4, 8].contains(&arity) {
                return Err(anyhow!("merkle_arity {} must be 2,4,or 8", arity));
            }
        }
        if let Some(b) = self.fri_blowup {
            if b < 2 {
                return Err(anyhow!("fri_blowup {} must be >= 2", b));
            }
        }
        if let Some(q) = self.fri_queries {
            if q < 16 {
                return Err(anyhow!("fri_queries {} must be >= 16", q));
            }
        }
        if let Some(g) = self.grind_bits {
            if g > 64 {
                return Err(anyhow!("grind_bits {} too large (>64)", g));
            }
        }
        Ok(())
    }
}

fn profiles_dir() -> PathBuf {
    PathBuf::from("profiles")
}

fn read_one(path: &Path) -> Result<Profile> {
    let s =
        fs::read_to_string(path).with_context(|| format!("reading profile {}", path.display()))?;
    let p: Profile =
        toml::from_str(&s).with_context(|| format!("parsing profile {}", path.display()))?;
    p.validate()?;
    Ok(p)
}

fn builtin_profiles() -> Vec<Profile> {
    let mut profiles = vec![
        Profile {
            id: "balanced".to_string(),
            lambda_bits: 100,
            fri_blowup: Some(16),
            fri_queries: Some(30),
            grind_bits: Some(18),
            merkle_arity: Some(2),
            const_col_limit: None,
            rows_max: None,
        },
        Profile {
            id: "dev-fast".to_string(),
            lambda_bits: 80,
            fri_blowup: Some(8),
            fri_queries: Some(24),
            grind_bits: Some(16),
            merkle_arity: Some(2),
            const_col_limit: None,
            rows_max: None,
        },
        Profile {
            id: "secure".to_string(),
            lambda_bits: 120,
            fri_blowup: Some(32),
            fri_queries: Some(50),
            grind_bits: Some(20),
            merkle_arity: Some(2),
            const_col_limit: None,
            rows_max: None,
        },
    ];
    profiles.sort_by(|a, b| a.id.cmp(&b.id));
    profiles
}

/// Load all TOML profiles from /profiles, sorted by id (stable order).
pub fn load_all_profiles() -> Result<Vec<Profile>> {
    let dir = profiles_dir();
    let mut out = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(&dir).with_context(|| format!("listing {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "toml").unwrap_or(false) {
                let p = read_one(&path)?;
                out.push(p);
            }
        }
    }
    if out.is_empty() {
        out = builtin_profiles();
    } else {
        out.sort_by(|a, b| a.id.cmp(&b.id));
    }
    Ok(out)
}

pub fn load_all_profiles_or_default() -> Vec<Profile> {
    load_all_profiles().unwrap_or_else(|e| {
        eprintln!("WARN: failed to load profiles: {e}");
        builtin_profiles()
    })
}
