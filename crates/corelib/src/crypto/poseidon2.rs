//! Placeholder Poseidon2 adapter that conforms to Hash32.
//! Until real field-friendly permutation lands, we domain-separate BLAKE3.

use crate::crypto::hash::Hash32;
use blake3::Hasher;

pub struct Poseidon2 {
    inner: Hasher,
}

impl Hash32 for Poseidon2 {
    fn new() -> Self {
        let mut inner = Hasher::new();
        inner.update(b"POSEIDON2");
        Self { inner }
    }

    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finalize(self) -> [u8; 32] {
        *self.inner.finalize().as_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::blake3::Blake3;
    use crate::crypto::hash::hash_labeled;

    #[test]
    fn poseidon2_domain_separator_changes_output() {
        let b = hash_labeled::<Blake3>("LBL", b"abc");
        let p = hash_labeled::<Poseidon2>("LBL", b"abc");
        assert_ne!(b, p);
    }
}
