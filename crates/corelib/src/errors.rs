use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("backend with id '{0}' is already registered")]
    DuplicateBackend(String),
    #[error("backend '{0}' not found")]
    BackendNotFound(String),
}

#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("capability mismatch: {0}")]
    Mismatch(String),
    #[error("field '{field}' not supported by backend '{backend_id}'")]
    FieldUnsupported { backend_id: String, field: String },
    #[error("hash '{hash}' not supported by backend '{backend_id}'")]
    HashUnsupported { backend_id: String, hash: String },
    #[error("FRI arity '{fri_arity}' not supported by backend '{backend_id}'")]
    FriArityUnsupported { backend_id: String, fri_arity: u32 },
    #[error("recursion required but backend '{backend_id}' reports none")]
    RecursionUnavailable { backend_id: String },

    #[error("profile '{0}' not found")]
    ProfileNotFound(String),
}
