//! Winterfell backend adapter (stub).

use zkprov_corelib::backend::{Capabilities, ProverBackend, VerifierBackend};

#[derive(Debug, Default)]
pub struct WinterfellBackend;

impl ProverBackend for WinterfellBackend {
    fn id(&self) -> &'static str {
        "winterfell@0.0"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            fields: vec![],
            hashes: vec![],
            fri_arities: vec![],
            recursion: "none",
            lookups: false,
            curves: vec![],
            pedersen: false,
        }
    }
}

impl VerifierBackend for WinterfellBackend {}
