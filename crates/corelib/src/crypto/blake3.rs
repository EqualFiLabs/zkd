//! BLAKE3 implementation of Hash32.

use crate::crypto::hash::Hash32;

pub struct Blake3 {
    inner: blake3::Hasher,
}

impl Hash32 for Blake3 {
    fn new() -> Self {
        Self {
            inner: blake3::Hasher::new(),
        }
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
    use crate::crypto::hash::{hash_labeled, hash_one_shot};

    #[test]
    fn blake3_hashes() {
        let d0 = hash_one_shot::<Blake3>(b"");
        let d1 = hash_one_shot::<Blake3>(b"abc");
        assert_ne!(d0, d1);
        let dl = hash_labeled::<Blake3>("LBL", b"abc");
        assert_ne!(d1, dl);
    }
}
