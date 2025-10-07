//! Global backend registry (thread-safe).
use std::collections::BTreeMap;
use std::sync::{Arc, Once, RwLock};

use crate::backend::{BackendInfo, Capabilities, ProverBackend, VerifierBackend};
use crate::errors::RegistryError;

use zkprov_backend_native::NativeBackend;

pub struct DynBackend {
    pub prover: Box<dyn ProverBackend>,
    pub verifier: Box<dyn VerifierBackend>,
}

static REGISTRY: RwLock<BTreeMap<&'static str, Arc<DynBackend>>> = RwLock::new(BTreeMap::new());
static INIT: Once = Once::new();

pub fn register_backend(
    prover: Box<dyn ProverBackend>,
    verifier: Box<dyn VerifierBackend>,
) -> Result<(), RegistryError> {
    let id = prover.id();
    let mut guard = REGISTRY.write().expect("poisoned backend registry");
    if guard.contains_key(id) {
        return Err(RegistryError::DuplicateBackend(id.to_string()));
    }
    guard.insert(id, Arc::new(DynBackend { prover, verifier }));
    Ok(())
}

pub fn list_backend_infos() -> Vec<BackendInfo> {
    let guard = REGISTRY.read().expect("poisoned backend registry");
    guard
        .iter()
        .map(|(id, dynb)| BackendInfo {
            id,
            recursion: dynb.prover.capabilities().recursion != "none",
        })
        .collect()
}

pub fn get_backend(id: &str) -> Result<Arc<DynBackend>, RegistryError> {
    let guard = REGISTRY.read().expect("poisoned backend registry");
    guard
        .get(id)
        .cloned()
        .ok_or_else(|| RegistryError::BackendNotFound(id.to_string()))
}

/// Helper used by CLI/tests to ensure at least builtins are available.
pub fn ensure_builtins_registered() {
    INIT.call_once(|| {
        let _ = register_native_backend(); // ignore duplicate errors if any
    });
}

fn register_native_backend() -> Result<(), RegistryError> {
    register_backend(
        Box::new(NativeBackend::default()),
        Box::new(NativeBackend::default()),
    )
}

impl ProverBackend for NativeBackend {
    fn id(&self) -> &'static str {
        "native@0.0"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            fields: vec!["Prime254"],
            hashes: vec!["blake3"],
            fri_arities: vec![2, 4],
            recursion: "none",
            lookups: false,
        }
    }
}

impl VerifierBackend for NativeBackend {}
