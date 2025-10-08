//! Generic hash trait and helpers.

/// A simple streaming hash trait with fixed-size 32-byte digests.
/// Designed to be implemented by BLAKE3, Keccak-256, Poseidon2 (sponge-based wrapper), etc.
pub trait Hash32 {
    /// Create a new hasher.
    fn new() -> Self
    where
        Self: Sized;
    /// Absorb bytes into the state.
    fn update(&mut self, data: &[u8]);
    /// Finalize and produce a 32-byte digest.
    fn finalize(self) -> [u8; 32];
}

/// Compute one-shot hash.
pub fn hash_one_shot<H: Hash32>(data: &[u8]) -> [u8; 32] {
    let mut h = H::new();
    h.update(data);
    h.finalize()
}

/// Domain-separated hashing: H(label || data)
pub fn hash_labeled<H: Hash32>(label: &str, data: &[u8]) -> [u8; 32] {
    let mut h = H::new();
    h.update(label.as_bytes());
    h.update(data);
    h.finalize()
}
