use serde::{Deserialize, Serialize};

/// User/CLI-selected configuration to be validated against a backend's capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub backend_id: String,     // e.g., "native@0.0"
    pub field: String,          // e.g., "Prime254"
    pub hash: String,           // e.g., "blake3"
    pub fri_arity: u32,         // e.g., 2 or 4
    pub recursion_needed: bool, // true if caller intends to use recursion features
}

impl Config {
    pub fn new<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        backend_id: S1,
        field: S2,
        hash: S3,
        fri_arity: u32,
        recursion_needed: bool,
    ) -> Self {
        Self {
            backend_id: backend_id.into(),
            field: field.into(),
            hash: hash.into(),
            fri_arity,
            recursion_needed,
        }
    }
}
