//! Winterfell backend adapter (stub).

use zkprov_corelib::backend::{Capabilities, ProverBackend, VerifierBackend};

#[derive(Clone, Debug, serde::Serialize)]
pub struct WinterfellCapabilities {
    pub name: &'static str,
    pub field: &'static str,
    pub hashes: Vec<&'static str>,
    pub commitments: Vec<&'static str>,
    pub recursion: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub blowup: u8,
    pub fri_arity: u8,
    pub queries: u8,
    pub grinding: u8,
}

pub fn capabilities() -> WinterfellCapabilities {
    WinterfellCapabilities {
        name: "winterfell@0.6",
        field: "Prime256",
        hashes: vec!["blake3", "poseidon2", "rescue", "keccak"],
        commitments: vec!["Pedersen(placeholder)", "PoseidonCommit", "KeccakCommit"],
        recursion: false,
    }
}

pub fn profile_map(id: &str) -> Profile {
    match id {
        "fast" | "dev-fast" => Profile {
            blowup: 8,
            fri_arity: 2,
            queries: 24,
            grinding: 16,
        },
        "secure" => Profile {
            blowup: 32,
            fri_arity: 2,
            queries: 50,
            grinding: 20,
        },
        _ => Profile {
            blowup: 16,
            fri_arity: 2,
            queries: 30,
            grinding: 18,
        },
    }
}

#[derive(Debug, Default)]
pub struct WinterfellBackend;

impl ProverBackend for WinterfellBackend {
    fn id(&self) -> &'static str {
        "winterfell@0.6"
    }

    fn capabilities(&self) -> Capabilities {
        let wf_caps = capabilities();
        Capabilities {
            fields: vec![wf_caps.field],
            hashes: wf_caps.hashes.clone(),
            fri_arities: vec![2, 4],
            recursion: if wf_caps.recursion {
                "stark-in-stark"
            } else {
                "none"
            },
            lookups: false,
            curves: vec!["placeholder"],
            pedersen: wf_caps
                .commitments
                .iter()
                .any(|commitment| commitment.starts_with("Pedersen")),
        }
    }
}

impl VerifierBackend for WinterfellBackend {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_capabilities() {
        let caps = capabilities();
        assert_eq!(caps.name, "winterfell@0.6");
        assert_eq!(caps.field, "Prime256");
        assert_eq!(caps.hashes, vec!["blake3", "poseidon2", "rescue", "keccak"]);
        assert!(!caps.recursion);
    }

    #[test]
    fn profiles_have_reasonable_defaults() {
        let fast = profile_map("fast");
        let balanced = profile_map("balanced");
        let secure = profile_map("secure");

        assert!(fast.blowup < balanced.blowup);
        assert!(secure.blowup > balanced.blowup);
        assert_eq!(balanced.fri_arity, 2);
        assert_eq!(profile_map("unknown").blowup, balanced.blowup);
    }
}
