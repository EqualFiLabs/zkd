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
}
