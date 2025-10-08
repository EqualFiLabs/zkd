//! Keccak-256 (SHA3-256 without padding change) as Hash32.

use crate::crypto::hash::Hash32;
use tiny_keccak::{Hasher as TKHasher, Keccak};

pub struct Keccak256 {
    inner: Keccak,
}

impl Hash32 for Keccak256 {
    fn new() -> Self {
        Self {
            inner: Keccak::v256(),
        }
    }

    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finalize(self) -> [u8; 32] {
        let mut out = [0u8; 32];
        let inner = self.inner;
        inner.finalize(&mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::hash::hash_one_shot;

    // Keccak-256("") =
    // c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470
    #[test]
    fn keccak256_empty() {
        let got = hash_one_shot::<Keccak256>(b"");
        let exp = hex::decode("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470")
            .unwrap();
        assert_eq!(got, exp.as_slice());
    }
}

// lightweight hex for test only
#[cfg(test)]
mod hex {
    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if !s.len().is_multiple_of(2) {
            return Err("len".into());
        }
        let mut out = Vec::with_capacity(s.len() / 2);
        let bytes = s.as_bytes();
        for i in (0..bytes.len()).step_by(2) {
            let hi = val(bytes[i])?;
            let lo = val(bytes[i + 1])?;
            out.push((hi << 4) | lo);
        }
        Ok(out)
    }

    fn val(b: u8) -> Result<u8, String> {
        match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            b'A'..=b'F' => Ok(b - b'A' + 10),
            _ => Err("hex".into()),
        }
    }
}
