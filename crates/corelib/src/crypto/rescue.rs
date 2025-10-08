//! Placeholder Rescue adapter implementing Hash32 via domain-separated BLAKE3.

use crate::crypto::hash::Hash32;
use blake3::Hasher;

pub struct Rescue {
    inner: Hasher,
}

impl Hash32 for Rescue {
    fn new() -> Self {
        let mut inner = Hasher::new();
        inner.update(b"RESCUE");
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
    fn rescue_domain_separator_changes_output() {
        let b = hash_labeled::<Blake3>("LBL", b"abc");
        let r = hash_labeled::<Rescue>("LBL", b"abc");
        assert_ne!(b, r);
    }
}
